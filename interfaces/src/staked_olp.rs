extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StakedOlpError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
}

impl From<StakedOlpError> for Vec<u8> {
    fn from(err: StakedOlpError) -> Vec<u8> {
        use StakedOlpError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
        }
    }
}

sol_interface! {
    interface IStakedOlp {
        function init() external;
    }
}
