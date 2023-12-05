extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IYieldTracker {
        function init(address gov, address yield_token) external;

        function setGov(address gov) external;

        function setDistributor(address distributor) external;

        function claim(address account, address receiver) external returns (uint256);

        function getTokensPerInterval() external view returns (uint256);

        function claimable(address account) external view returns (uint256);

        function updateRewards(address account) external;
    }
}

sol! {
    event Claim(address receiver, uint256 amount);

    error Forbidden ();
    error AlreadyInitialized();
    error NotInitialized();
    error SumOverflow ();
    error SubOverflow ();
    error MulOverflow ();
    error DivByZero ();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum YieldTrackerError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    SumOverflow,
    SubOverflow,
    MulOverflow,
    DivByZero,
}

impl From<YieldTrackerError> for Vec<u8> {
    fn from(err: YieldTrackerError) -> Vec<u8> {
        use YieldTrackerError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::SumOverflow => SumOverflow {}.encode(),
            E::SubOverflow => SubOverflow {}.encode(),
            E::MulOverflow => MulOverflow {}.encode(),
            E::DivByZero => DivByZero {}.encode(),
        }
    }
}
