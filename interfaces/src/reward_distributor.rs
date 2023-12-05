extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event Distribute(uint256 amount);
    event TokensPerIntervalChange(uint256 amount);

    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error InvalidSender();
    error ZeroLastDistributionTime();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RewardDistributorError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    InvalidSender,
    ZeroLastDistributionTime,
}

impl From<RewardDistributorError> for Vec<u8> {
    fn from(err: RewardDistributorError) -> Vec<u8> {
        use RewardDistributorError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::InvalidSender => InvalidSender {}.encode(),
            E::ZeroLastDistributionTime => ZeroLastDistributionTime {}.encode(),
        }
    }
}

sol_interface! {
    interface IRewardDistributor {
        function init(address gov, address reward_token, address reward_tracker, address reward_tracker_staking) external;

        function setGov(address gov) external;

        function setAdmin(address admin) external;

        function updateLastDistributionTime() external;

        function setTokensPerInterval(uint256 amount) external;

        function pendingRewards() external view returns (uint256);

        function distribute() external returns (uint256);

        function rewardToken() external view returns (address);

        function tokensPerInterval() external view returns (uint256);
    }
}
