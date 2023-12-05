extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IShortsTracker {
        function init(address gov, address vault, address vault_utils, address positions_manager) external;

        function setGov(address gov) external;

        function setHandler(address handler, bool is_active) external;

        function setIsGlobalShortDataReady(bool value) external;

        function updateGlobalShortData(address account, address collateral_token, address index_token, bool is_long, uint256 size_delta, uint256 mark_price, bool is_increase) external;

        function getGlobalShortDelta(address token) external view returns (bool, uint256);

        function setInitData(address[] memory tokens, uint256[] memory average_prices) external;

        function getNextGlobalShortData(address account, address collateral_token, address index_token, uint256 next_price, uint256 size_delta, bool is_increase) external view returns (uint256, uint256);

        function getRealisedPnl(address account, address collateral_token, address index_token, uint256 size_delta, bool is_increase) external view returns (int256);
    }
}

sol! {
    event GlobalShortDataUpdated(address indexed token, uint256 global_short_size, uint256 global_short_average_price);

    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error AlreadyMigrated();
    error Overflow();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShortsTrackerError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    AlreadyMigrated,
    Overflow,
}

impl From<ShortsTrackerError> for Vec<u8> {
    fn from(err: ShortsTrackerError) -> Vec<u8> {
        use ShortsTrackerError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::AlreadyMigrated => AlreadyMigrated {}.encode(),
            E::Overflow => Overflow {}.encode(),
        }
    }
}
