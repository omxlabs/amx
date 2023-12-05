use ethers::{
    abi::{Abi, Function as ContractFunction, Token},
    types::{Address, Bytes, U256},
};
use ethers_providers::{HttpClientError, JsonRpcError};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use thiserror::Error as ThisError;
use wasmer::{
    imports, AsStoreRef, Function, FunctionEnv, Instance, Memory, MemoryView, Module, RuntimeError,
    Store, Value,
};

use crate::stylus_testing::{
    provider::{TestInnerProvider, TestProvider},
    vm_hooks,
};

use super::transaction::TransactionKey;

#[derive(Debug, ThisError, Clone)]
pub enum ContractCallError {
    #[error("{0}")]
    Message(String),

    #[error("Runtime error: {0}")]
    RuntimeError(#[from] RuntimeError),

    #[error("Revert: {0}")]
    Revert(Bytes),
}

impl From<ContractCallError> for HttpClientError {
    fn from(err: ContractCallError) -> Self {
        use ContractCallError as E;
        match err {
            E::Message(msg) => JsonRpcError {
                code: 0,
                data: None,
                message: msg,
            }
            .into(),
            E::RuntimeError(err) => JsonRpcError {
                code: 0,
                data: None,
                message: err.message(),
            }
            .into(),

            E::Revert(data) => JsonRpcError {
                code: 0,
                data: Some(hex::encode(data.to_vec()).into()),
                message: "revert".to_string(),
            }
            .into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractState {
    /// Counter for reentrant calls
    reentrant_counter: u32,
    /// Contract binary
    binary: Vec<u8>,
    /// Contract storage. Before each call this storage is copied to temporary
    /// storage and after the call the temporary storage merged back to the
    /// main storage
    storage_bytes32: HashMap<U256, U256>,
    /// This storage is used to store the state of the contract during the
    /// transaction execution. After the transaction is finished the storage
    /// is merged to the main storage. If the transaction is reverted the
    /// storage is discarded
    transactions_storages: HashMap<TransactionKey, HashMap<U256, U256>>,
    /// Execution result code
    result: Vec<u8>,
    /// Execution return data
    return_data: Vec<u8>,
    /// Contract ABI
    abi: Abi,
    /// Contract label
    label: String,
}

impl ContractState {
    pub fn new(binary: &[u8], abi: Abi, label: String) -> Self {
        Self {
            transactions_storages: HashMap::new(),
            abi,
            binary: binary.to_vec(),
            reentrant_counter: 0,
            storage_bytes32: HashMap::new(),
            result: Vec::new(),
            return_data: Vec::new(),
            label,
        }
    }

    pub fn reset_reentrant_counter(&mut self) {
        self.reentrant_counter = 0;
    }

    pub fn inc_reentrant_counter(&mut self) {
        self.reentrant_counter += 1;
    }

    pub fn reset_result(&mut self) {
        self.result = Vec::new();
    }

    /// Initialize temp transaction storage if it is not initialized yet
    pub fn init_transaction(&mut self, transaction_key: TransactionKey) {
        let storage = self.storage_bytes32.clone();

        self.transactions_storages
            .entry(transaction_key)
            .or_insert(storage);
    }

    /// Merge temp transaction storage to the main storage
    pub fn commit_transaction(&mut self, transaction_key: TransactionKey) {
        if let Some(transactions_storages) = self.transactions_storages.remove(&transaction_key) {
            for (key, value) in transactions_storages.iter() {
                self.storage_bytes32.insert(*key, *value);
            }
            log::debug!(
                "transaction {transaction_key} committed for contract {contract}",
                contract = self.label
            );
        }
    }

    pub fn merge_to_transaction_storage(
        &mut self,
        transaction_key: TransactionKey,
        storage: HashMap<U256, U256>,
    ) {
        let temp_storage = self
            .transactions_storages
            .get_mut(&transaction_key)
            .expect(&format!("transaction {:?} should exists", transaction_key));

        for (key, value) in storage.iter() {
            temp_storage.insert(*key, *value);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Env {
    state: Arc<Mutex<ContractState>>,
    provider: TestInnerProvider,
    value: U256,
    entrypoint_data: Vec<u8>,
    memory: Option<Memory>,
    sender: Address,
    address: Address,
    transaction_key: TransactionKey,
}

pub type ContractCallResult<T> = Result<T, ContractCallError>;

#[derive(Debug)]
pub struct ContractCall {
    env: FunctionEnv<Env>,
    store: Store,
    instance: Instance,
}

impl ContractCall {
    pub fn new(
        provider: TestInnerProvider,
        address: Address,
        state: Arc<Mutex<ContractState>>,
        transaction_key: TransactionKey,
    ) -> Self {
        let mut store = Store::default();

        let bytes = {
            let mut state = state.lock().unwrap();

            state.init_transaction(transaction_key);

            state.binary.clone()
        };

        let module = Module::new(&store, bytes).unwrap();

        let env = FunctionEnv::new(
            &mut store,
            Env {
                transaction_key,
                state,
                sender: Address::zero(),
                value: U256::zero(),
                entrypoint_data: Vec::new(),
                provider,
                address,
                memory: None,
            },
        );

        let import_object = imports! {
            "vm_hooks" => {
                "msg_reentrant" => Function::new_typed_with_env(&mut store, &env, vm_hooks::msg_reentrant),
                "read_args" => Function::new_typed_with_env(&mut store, &env, vm_hooks::read_args),
                "storage_store_bytes32" => Function::new_typed_with_env(&mut store, &env, vm_hooks::storage_store_bytes32),
                "write_result" => Function::new_typed_with_env(&mut store, &env, vm_hooks::write_result),
                "native_keccak256" => Function::new_typed_with_env(&mut store, &env, vm_hooks::native_keccak256),
                "storage_load_bytes32" => Function::new_typed_with_env(&mut store, &env, vm_hooks::storage_load_bytes32),
                "msg_value" => Function::new_typed_with_env(&mut store, &env, vm_hooks::msg_value),
                "emit_log" => Function::new_typed_with_env(&mut store, &env, vm_hooks::emit_log),
                "memory_grow" => Function::new_typed_with_env(&mut store, &env, vm_hooks::memory_grow),
                "msg_sender" => Function::new_typed_with_env(&mut store, &env, vm_hooks::msg_sender),
                "block_timestamp" => Function::new_typed_with_env(&mut store, &env, vm_hooks::block_timestamp),
                "call_contract" => Function::new_typed_with_env(&mut store, &env, vm_hooks::call_contract),
                "delegate_call_contract" => Function::new_typed_with_env(&mut store, &env, vm_hooks::delegate_call_contract),
                "static_call_contract" => Function::new_typed_with_env(&mut store, &env, vm_hooks::static_call_contract),
                "read_return_data" => Function::new_typed_with_env(&mut store, &env, vm_hooks::read_return_data),
                "contract_address" => Function::new_typed_with_env(&mut store, &env, vm_hooks::contract_address),
            },
            "console" => {
                "log_txt" => Function::new_typed_with_env(&mut store, &env, vm_hooks::log_txt),
            }
        };

        // Compile our webassembly into an `Instance`.
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();

        let memory = instance.exports.get_memory("memory").unwrap().clone();

        env.as_mut(&mut store).memory = Some(memory);

        Self {
            instance,
            env,
            store,
        }
    }

    pub fn address(&self) -> Address {
        self.env.as_ref(&self.store).address
    }

    pub fn abi(&self) -> Abi {
        self.env
            .as_ref(&self.store)
            .state
            .lock()
            .unwrap()
            .abi
            .clone()
    }

    pub fn with_value(mut self, value: U256) -> Self {
        self.env.as_mut(&mut self.store).value = value;

        self
    }

    pub fn with_sender(mut self, sender: Address) -> Self {
        self.env.as_mut(&mut self.store).sender = sender;

        self
    }

    /// Set contract entry point data, process value, reset previous call
    /// result and increment reentrant counter
    fn prepare_call(&mut self, data: &[u8]) {
        let env = self.env.as_mut(&mut self.store);

        env.entrypoint_data = data.to_vec();

        let mut state = env.state.lock().unwrap();
        state.reset_result();
        state.inc_reentrant_counter();

        if env.value > U256::zero() {
            env.provider.send_eth(env.sender, env.address, env.value);
        }
    }

    fn process_data(&self, result: Box<[Value]>) -> ContractCallResult<Vec<u8>> {
        let results = result.to_vec();
        let result = results[0].i32().unwrap();

        let result_data = {
            self.env
                .as_ref(&self.store)
                .state
                .lock()
                .unwrap()
                .result
                .clone()
        };

        log::debug!(
            "{} -> result: {}",
            self.env.as_ref(&self.store).label(self.address()),
            result
        );

        if result != 0 {
            return Err(ContractCallError::Revert(result_data.into()));
        }

        return Ok(result_data);
    }

    pub fn get_function(&self, data: &[u8]) -> Option<ContractFunction> {
        self.abi()
            .functions()
            .find(|f| data[0..4].to_vec() == f.short_signature().to_vec())
            .cloned()
    }

    pub fn get_signature(&self, data: &[u8]) -> Option<String> {
        Some(self.get_function(data)?.signature())
    }

    pub fn parse_input(&self, data: &[u8]) -> Option<Vec<Token>> {
        let function = self.get_function(data)?;

        Some(function.decode_input(&data[4..]).expect("decode input"))
    }

    pub fn entry_point(&mut self, data: &[u8]) -> ContractCallResult<Vec<u8>> {
        self.prepare_call(data);

        let entrypoint = self
            .instance
            .exports
            .get_function("user_entrypoint")
            .unwrap();

        let result = entrypoint.call(&mut self.store, &[Value::I32(data.len() as i32)])?;

        let result_data = self.process_data(result)?;

        Ok(result_data)
    }

    pub fn read_mem(&self, ptr: u64, len: usize) -> Vec<u8> {
        let memory = self
            .env
            .as_ref(&self.store)
            .memory
            .as_ref()
            .expect("memory should be initialized");

        let view = memory.view(&self.store);

        let mut data = vec![0; len];

        view.read(ptr, &mut data).unwrap();

        data
    }

    pub fn env(&self) -> &Env {
        self.env.as_ref(&self.store)
    }

    pub fn block_number(&self) -> u64 {
        self.env
            .as_ref(&self.store)
            .provider
            .block_number()
            .as_u64()
    }

    pub fn commit_transaction(&self) {
        self.env.as_ref(&self.store).commit_transaction();
    }
}

impl Env {
    pub fn memory_mut(&mut self) -> &mut Memory {
        self.memory.as_mut().expect("memory should be initialized")
    }

    pub fn memory(&self) -> &Memory {
        self.memory.as_ref().expect("memory should be initialized")
    }

    pub fn sender(&self) -> Address {
        self.sender
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn provider(&self) -> TestInnerProvider {
        self.provider.clone()
    }

    pub fn label(&self, address: Address) -> String {
        self.provider.label(address)
    }

    pub fn value(&self) -> U256 {
        self.value
    }

    pub fn set_value(&mut self, value: U256) {
        self.value = value;
    }

    pub fn set_result(&mut self, result: Vec<u8>) {
        let mut state = self.state.lock().unwrap();

        state.result = result;
    }

    pub fn storage_bytes32_get(&self, key: U256) -> U256 {
        let state = self.state.lock().unwrap();

        state
            .transactions_storages
            .get(&self.transaction_key)
            .expect(&format!(
                "transaction {} should exists",
                self.transaction_key
            ))
            .get(&key)
            .cloned()
            .unwrap_or_default()
            .clone()
    }

    pub fn storage_bytes32_insert(&mut self, key: U256, value: U256) {
        let mut state = self.state.lock().unwrap();

        state
            .transactions_storages
            .get_mut(&self.transaction_key)
            .expect(&format!(
                "transaction {} should exists",
                self.transaction_key
            ))
            .insert(key, value);
    }

    pub fn commit_transaction(&self) {
        let mut state = self.state.lock().unwrap();

        state.commit_transaction(self.transaction_key);
    }

    pub fn return_data(&self) -> Vec<u8> {
        self.state.lock().unwrap().return_data.clone()
    }

    pub fn set_return_data(&mut self, return_data: Vec<u8>) {
        self.state.lock().unwrap().return_data = return_data;
    }

    pub fn entrypoint_data(&self) -> Vec<u8> {
        self.entrypoint_data.clone()
    }

    pub fn block_timestamp(&self) -> u64 {
        self.provider.block_timestamp().as_u64()
    }

    pub fn set_entrypoint_data(&mut self, entrypoint_data: Vec<u8>) {
        self.entrypoint_data = entrypoint_data;
    }

    pub fn reentrant_counter(&self) -> u32 {
        self.state.lock().unwrap().reentrant_counter
    }

    pub fn inc_reentrant_counter(&mut self) {
        let mut state = self.state.lock().unwrap();

        state.reentrant_counter += 1;
    }

    pub fn reset_reentrant_counter(&mut self) {
        let mut state = self.state.lock().unwrap();

        state.reset_reentrant_counter()
    }

    pub fn view(&self, store: &impl AsStoreRef) -> MemoryView {
        let memory = self.memory();
        memory.view(store)
    }

    pub fn transaction(&self) -> TransactionKey {
        self.transaction_key
    }
}
