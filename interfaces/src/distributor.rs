extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IDistributor {
        function distribute() external returns (uint256);
        function getRewardToken(address receiver) external view returns (address);
        function getDistributionAmount(address receiver) external view returns (uint256);
        function tokensPerInterval(address receiver) external view returns (uint256);
    }
}

sol! {
    event Distribute(address receiver, uint256 amount);
    event DistributionChange(address receiver, uint256 amount, address reward_token);
    event TokensPerIntervalChange(address receiver, uint256 amount);

    error Forbidden();
    error NotInitialized();
    error AlreadyInitialized();
    error PendingDistribution();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistributorError {
    Forbidden,
    NotInitialized,
    AlreadyInitialized,
    PendingDistribution,
}

impl From<DistributorError> for Vec<u8> {
    fn from(err: DistributorError) -> Vec<u8> {
        use DistributorError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::PendingDistribution => PendingDistribution {}.encode(),
        }
    }
}
