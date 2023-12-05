use std::borrow::Borrow;

use ethers::abi::{AbiEncode, Detokenize};
use ethers_contract::{ContractError, FunctionCall};
use ethers_providers::Middleware;

use crate::contracts::LiveClient;

pub fn map_contract_error<M: Middleware>(e: ContractError<M>) -> String {
    match e {
        ContractError::Revert(b) => format!("{} ({})", hex::encode(b.to_vec()), b.encode_hex(),),
        e => panic!("unexpected error: {:?}", e.to_string()),
    }
}

pub async fn send<B, D>(call: FunctionCall<B, LiveClient, D>) -> Result<(), String>
where
    B: Borrow<LiveClient>,
    D: Detokenize,
{
    call.send()
        .await
        .map_err(map_contract_error)?
        .await
        .unwrap();

    Ok(())
}
