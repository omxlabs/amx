use std::borrow::Borrow;

use ethers_contract::ContractError;
use ethers_providers::Middleware;

pub fn map_contract_error<M: Middleware>(e: ContractError<M>) -> String {
    match e {
        ContractError::Revert(b) => String::from_utf8_lossy(b.borrow()).to_string(),
        e => panic!("unexpected error: {:?}", e),
    }
}
