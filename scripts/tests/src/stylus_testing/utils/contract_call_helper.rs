use std::borrow::Borrow;

use ethers::abi::Detokenize;
use ethers_contract::{ContractError, FunctionCall};
use ethers_providers::Middleware;

use crate::stylus_testing::provider::TestClient;

pub fn map_contract_error<M: Middleware>(e: ContractError<M>) -> String {
    match e {
        ContractError::Revert(b) => String::from_utf8_lossy(b.borrow()).to_string(),
        e => panic!("unexpected error: {:?}", e),
    }
}

pub async fn send<B, D>(call: FunctionCall<B, TestClient, D>) -> Result<(), String>
where
    B: Borrow<TestClient>,
    D: Detokenize,
{
    call.send()
        .await
        .map_err(map_contract_error)?
        .await
        .unwrap();

    Ok(())
}
