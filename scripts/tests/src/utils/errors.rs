use ethers_contract::{ContractError, EthError};

use crate::stylus_testing::provider::TestClient;

#[track_caller]
pub fn assert_revert<T>(err: ContractError<TestClient>, expected: T)
where
    T: EthError,
{
    let err = err.as_revert().cloned().expect("Expected revert error");

    let err = err.to_vec();

    let expected = expected.encode();

    assert_eq!(err, expected);
}

#[track_caller]
pub fn assert_revert_str(err: ContractError<TestClient>, expected: impl ToString) {
    let err = err.to_string();

    let expected = format!(
        "Contract call reverted with data: 0x{}",
        hex::encode(expected.to_string())
    );

    assert_eq!(err, expected);
}

pub trait ContractRevertExt {
    fn assert_revert<T>(self, expected: T)
    where
        T: EthError;

    fn assert_revert_str(self, expected: impl ToString);
}

impl<R> ContractRevertExt for Result<R, ContractError<TestClient>>
where
    R: std::fmt::Debug,
{
    #[track_caller]
    fn assert_revert<T>(self, expected: T)
    where
        T: EthError,
    {
        assert_revert(self.unwrap_err(), expected);
    }

    #[track_caller]
    fn assert_revert_str(self, expected: impl ToString) {
        assert_revert_str(self.unwrap_err(), expected);
    }
}
