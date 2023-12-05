extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VesterError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
}

impl From<VesterError> for Vec<u8> {
    fn from(err: VesterError) -> Vec<u8> {
        use VesterError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
        }
    }
}

sol_interface! {
    interface IVester {
        function rewardTracker() external view returns (address);

        function claimForAccount(address account, address receiver) external returns (uint256);

        function claimable(address account) external view returns (uint256);
        function cumulativeClaimAmounts(address account) external view returns (uint256);
        function claimedAmounts(address account) external view returns (uint256);
        function pairAmounts(address account) external view returns (uint256);
        function getVestedAmount(address account) external view returns (uint256);
        function transferredAverageStakedAmounts(address account) external view returns (uint256);
        function transferredCumulativeRewards(address account) external view returns (uint256);
        function cumulativeRewardDeductions(address account) external view returns (uint256);
        function bonusRewards(address account) external view returns (uint256);

        function transferStakeValues(address sender, address receiver) external;
        function setTransferredAverageStakedAmounts(address account, uint256 amount) external;
        function setTransferredCumulativeRewards(address account, uint256 amount) external;
        function setCumulativeRewardDeductions(address account, uint256 amount) external;
        function setBonusRewards(address account, uint256 amount) external;

        function getMaxVestableAmount(address account) external view returns (uint256);
        function getCombinedAverageStakedAmount(address account) external view returns (uint256);
    }
}
