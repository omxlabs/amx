#![cfg(not(feature = "export-abi"))]
extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::{console, msg};

sol! {
    error ReentrantForbidden();
}

/// Validates that the current message is not reentrant.
pub fn no_reentrant() -> Result<(), Vec<u8>> {
    if msg::reentrant() {
        console!("Reentrant call forbidden");
        Err(ReentrantForbidden {}.encode())
    } else {
        Ok(())
    }
}
