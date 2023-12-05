use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use ethers::{
    abi::Abi,
    middleware::SignerMiddleware,
    providers::Provider,
    signers::LocalWallet,
    types::{
        Address, Bytes, NameOrAddress, Transaction, TransactionReceipt, TransactionRequest, H256,
        U256, U64,
    },
};
use ethers_providers::{HttpClientError, JsonRpcClient, Middleware};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{
    constants::CHAIN_ID,
    contract::{ContractCall, ContractState},
    transaction::TransactionKey,
};

pub type TestOuterProvider = Provider<TestInnerProvider>;
pub type TestClient = SignerMiddleware<TestOuterProvider, LocalWallet>;

#[derive(Debug, Clone)]
pub struct TestInnerProvider {
    last_transaction: Arc<Mutex<TransactionKey>>,
    contracts: Arc<Mutex<HashMap<Address, Arc<Mutex<ContractState>>>>>,
    balances: Arc<Mutex<HashMap<Address, U256>>>,
    transactions: Arc<Mutex<HashMap<U256, Transaction>>>,
    labels: Arc<Mutex<HashMap<Address, String>>>,
    block_number: Arc<Mutex<u64>>,
    block_timestamp: Arc<Mutex<u64>>,
}

impl TestInnerProvider {
    pub fn new() -> Self {
        Self {
            last_transaction: Arc::default(),
            contracts: Arc::default(),
            balances: Arc::default(),
            transactions: Arc::default(),
            block_number: Arc::default(),
            block_timestamp: Arc::default(),
            labels: Arc::default(),
        }
    }

    fn new_transaction(&self) -> TransactionKey {
        let mut last_transaction = self.last_transaction.lock().unwrap();

        let result = *last_transaction;

        **last_transaction += 1;

        result
    }

    fn commit_transaction(&self, transaction_key: TransactionKey) {
        self.contracts.lock().unwrap().iter().for_each(|(_, c)| {
            let mut c = c.lock().unwrap();

            c.commit_transaction(transaction_key);
        });
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for TestInnerProvider {
    type Error = HttpClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, HttpClientError> {
        log::debug!("method: {} -> {}", method, std::any::type_name::<R>());

        match method {
            "eth_call" => {
                let params = serde_json::to_string(&params).unwrap();
                log::debug!("\t| raw_params: {}", params);

                let params = serde_json::from_str::<(EthCallParams, String)>(&params).unwrap();

                self.reset_reentrant_counter();

                let value = params.0.value;
                let data = hex::decode(&params.0.data[2..]).unwrap();
                let contract_address = params.0.to;
                let sender_address = params.0.from;

                let sender_label = self.label(sender_address);
                let contract_label = self.label(contract_address);

                let transaction_key = self.new_transaction();

                log::debug!(
                    "\t| tx: {transaction_key}, from: {sender_label}, to: {contract_label}"
                );

                let contract_state = self
                    .contract(contract_address)
                    .expect(&format!("Contract {contract_label} not found"));

                let mut contract = ContractCall::new(
                    self.clone(),
                    contract_address,
                    contract_state,
                    transaction_key,
                )
                .with_sender(sender_address)
                .with_value(value);

                log::debug!(
                    "\t| input: {:?} {:?}",
                    contract.get_signature(&data),
                    contract.parse_input(&data)
                );

                let res: Bytes = contract.entry_point(&data)?.into();

                log::debug!("\t└ commit transaction: {transaction_key}");
                self.commit_transaction(transaction_key);

                let res = serde_json::to_string(&res).unwrap();

                log::debug!("res: {}", res);

                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_chainId" => {
                let res = Bytes::from(CHAIN_ID.to_be_bytes());

                let res = serde_json::to_string(&res).unwrap();

                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_getTransactionCount" => {
                let transaction_count = U256::zero();

                let res = serde_json::to_string(&transaction_count).unwrap();

                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_gasPrice" => {
                let gas_price = U256::zero();

                let res = serde_json::to_string(&gas_price).unwrap();

                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_estimateGas" => {
                let gas = U256::zero();

                let res = serde_json::to_string(&gas).unwrap();

                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_sendRawTransaction" => {
                let params = serde_json::to_string(&params).unwrap();
                log::debug!("raw_params: {}", params);

                let (tx_data,) = serde_json::from_str::<(Bytes,)>(&params).unwrap();

                let data = rlp::Rlp::new(tx_data.as_ref());

                let tx = TransactionRequest::decode_unsigned_rlp(&data).unwrap();
                log::debug!("tx: {:?}", tx);

                if tx.data.is_some() {
                    unimplemented!("Data is not supported yet {tx:?}");
                }

                let to = tx.to.clone().map(|v| match v {
                    NameOrAddress::Address(address) => address,
                    _ => unimplemented!("Name not implemented yet {:?}", tx),
                });

                if let Some(value) = tx.value {
                    match (tx.from, to) {
                        (Some(from), Some(to)) => {
                            self.send_eth(from, to, value);
                        }
                        (None, Some(to)) => {
                            self.mint_eth(to, value);
                        }
                        _ => unimplemented!("Unknown {:?}", tx),
                    };
                }

                let tx_hash_data = rand::random::<[u8; 32]>();
                let tx_hash = H256::from_slice(&tx_hash_data);

                {
                    let mut transactions = self.transactions.lock().unwrap();

                    let mut result_tx = Transaction::default();
                    result_tx.hash = tx_hash;
                    result_tx.from = tx.from.unwrap_or_default();
                    result_tx.value = tx.value.unwrap_or_default();
                    result_tx.to = to;
                    result_tx.block_number = Some(self.block_number());

                    let tx_hash = U256::from_big_endian(&tx_hash_data);
                    transactions.insert(tx_hash, result_tx);
                }

                let res = serde_json::to_string(&tx_hash).unwrap();
                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_getTransactionByHash" => {
                let params = serde_json::to_string(&params).unwrap();

                let (tx_hash,) = serde_json::from_str::<(U256,)>(&params).unwrap();

                let tx = self.transactions.lock().unwrap().get(&tx_hash).cloned();

                let res = serde_json::to_string(&tx).unwrap();
                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_getTransactionReceipt" => {
                let params = serde_json::to_string(&params).unwrap();

                let (tx_hash,) = serde_json::from_str::<(U256,)>(&params).unwrap();

                let mut hash_data = vec![0u8; 32];
                tx_hash.to_big_endian(&mut hash_data);

                let mut res = TransactionReceipt::default();

                res.transaction_hash = H256::from_slice(&hash_data);
                res.block_number = Some(self.block_number());

                let res = serde_json::to_string(&res).unwrap();
                return Ok(serde_json::from_str(&res).unwrap());
            }
            "eth_getBalance" => {
                let params = serde_json::to_string(&params).unwrap();

                let (addr, time_stamp) =
                    serde_json::from_str::<(Address, String)>(&params).unwrap();

                if time_stamp != "latest" {
                    unimplemented!("time_stamp: {}", time_stamp);
                }

                let balance = self.balance(addr);

                let res = serde_json::to_string(&balance).unwrap();
                return Ok(serde_json::from_str(&res).unwrap());
            }
            method => unimplemented!("Method \"{method}\""),
        };
    }
}

pub trait TestProvider {
    fn contract(&self, address: Address) -> Option<Arc<Mutex<ContractState>>>;

    fn deploy_contract(
        &self,
        bytes: &[u8],
        abi: ethers::core::abi::Abi,
        label: impl Display,
    ) -> Address;

    fn mint_eth(&self, to: Address, amount: U256);

    fn send_eth(&self, from: Address, to: Address, amount: U256);

    fn balance(&self, address: Address) -> U256;

    fn label(&self, address: Address) -> String;

    fn set_label(&self, address: Address, label: String);

    fn block_number(&self) -> U64;

    fn mine_block(&self);

    fn block_timestamp(&self) -> U64;

    fn advance_block_timestamp(&self, seconds: u64);

    fn reset_reentrant_counter(&self);
}

impl TestProvider for TestInnerProvider {
    fn block_number(&self) -> U64 {
        let block_number = self.block_number.lock().unwrap();

        U64::from(*block_number)
    }

    fn mine_block(&self) {
        let mut block_number = self.block_number.lock().unwrap();

        *block_number += 1;
    }

    fn block_timestamp(&self) -> U64 {
        let block_timestamp = self.block_timestamp.lock().unwrap();

        U64::from(*block_timestamp)
    }

    fn reset_reentrant_counter(&self) {
        self.contracts.lock().unwrap().iter().for_each(|(_, c)| {
            let mut c = c.lock().unwrap();

            c.reset_reentrant_counter();
        });
    }

    fn advance_block_timestamp(&self, seconds: u64) {
        let mut block_timestamp = self.block_timestamp.lock().unwrap();

        *block_timestamp += seconds;
    }

    fn contract(&self, address: Address) -> Option<Arc<Mutex<ContractState>>> {
        let contracts: std::sync::MutexGuard<
            '_,
            HashMap<ethers::types::H160, Arc<Mutex<ContractState>>>,
        > = self.contracts.lock().unwrap();

        contracts.get(&address).cloned()
    }

    fn deploy_contract(&self, bytes: &[u8], abi: Abi, label: impl Display) -> Address {
        let address = Address::random();

        let state = Arc::new(Mutex::new(ContractState::new(
            bytes,
            abi,
            label.to_string(),
        )));
        self.set_label(address, label.to_string());

        let mut contracts = self.contracts.lock().unwrap();
        contracts.insert(address, state.clone());

        address
    }

    fn mint_eth(&self, to: Address, amount: U256) {
        let mut balances = self.balances.lock().unwrap();

        let balance = balances.entry(to).or_insert(U256::zero());
        *balance += amount;

        log::debug!("mint_eth: {} {}", self.label(to), amount);
        log::debug!("\t└ balance: {}", balance);
    }

    // TODO add error handling
    fn send_eth(&self, from: Address, to: Address, amount: U256) {
        let mut balances = self.balances.lock().unwrap();

        let from_balance = balances.entry(from).or_insert(U256::zero());

        let from_label = self.label(from);
        let to_label = self.label(to);
        log::debug!("send_eth: {from_label} -> {to_label} {amount}",);
        log::debug!("\t└ sender_balance: {}", from_balance);

        if *from_balance < amount {
            panic!("Insufficient funds, {from_label} tried to send {amount} to {to_label} but only had {from_balance}", );
        }

        *from_balance -= amount;

        let to_balance = balances.entry(to).or_insert(U256::zero());
        *to_balance += amount;
    }

    fn balance(&self, address: Address) -> U256 {
        let balances = self.balances.lock().unwrap();

        balances.get(&address).cloned().unwrap_or_default()
    }

    fn label(&self, address: Address) -> String {
        let labels = self.labels.lock().unwrap();

        let label = labels.get(&address).cloned().unwrap_or(address.to_string());

        label
    }

    fn set_label(&self, address: Address, label: String) {
        let mut labels = self.labels.lock().unwrap();

        labels.insert(address, label);
    }
}

impl TestProvider for Arc<TestClient> {
    fn block_number(&self) -> U64 {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.block_number()
    }

    fn mine_block(&self) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.mine_block()
    }

    fn block_timestamp(&self) -> U64 {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.block_timestamp()
    }

    fn reset_reentrant_counter(&self) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.reset_reentrant_counter();
    }

    fn advance_block_timestamp(&self, seconds: u64) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.advance_block_timestamp(seconds)
    }

    fn contract(&self, address: Address) -> Option<Arc<Mutex<ContractState>>> {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.contract(address)
    }

    fn deploy_contract(&self, bytes: &[u8], abi: Abi, label: impl Display) -> Address {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.deploy_contract(bytes, abi, label)
    }

    fn mint_eth(&self, to: Address, amount: U256) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.mint_eth(to, amount)
    }

    fn send_eth(&self, from: Address, to: Address, amount: U256) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.send_eth(from, to, amount)
    }

    fn balance(&self, address: Address) -> U256 {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.balance(address)
    }

    fn label(&self, address: Address) -> String {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.label(address)
    }

    fn set_label(&self, address: Address, label: String) {
        let p: TestInnerProvider = self.provider().as_ref().clone();
        p.set_label(address, label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthCallParams {
    #[serde(rename = "accessList")]
    access_list: Vec<()>,
    data: String,
    from: Address,
    to: Address,
    #[serde(rename = "type")]
    tx_type: String,
    #[serde(default)]
    value: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimateRequestParams {
    #[serde(rename = "accessList")]
    access_list: Vec<()>,
    data: String,
    from: Address,
    to: Address,
    #[serde(rename = "type")]
    tx_type: String,
    #[serde(rename = "maxFeePerGas")]
    max_fee_per_gas: U64,
    #[serde(rename = "maxPriorityFeePerGas")]
    max_priority_fee_per_gas: U64,
    nonce: U256,
}

pub trait FromContractResult {
    fn from_contract_result(result: &[u8]) -> Self;
}

impl FromContractResult for String {
    fn from_contract_result(result: &[u8]) -> Self {
        hex::encode(result)
    }
}

impl FromContractResult for U256 {
    fn from_contract_result(result: &[u8]) -> Self {
        U256::from_big_endian(result)
    }
}
