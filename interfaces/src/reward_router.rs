extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event StakeOmx(address account, uint256 amount);
    event UnstakeOmx(address account, uint256 amount);

    event StakeOlp(address account, uint256 amount);
    event UnstakeOlp(address account, uint256 amount);

    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error InvalidAmount();
    error InvalidValue();
    error InvalidOlpAmount();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RewardRouterError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    InvalidAmount,
    InvalidValue,
    InvalidOlpAmount,
}

impl From<RewardRouterError> for Vec<u8> {
    fn from(err: RewardRouterError) -> Vec<u8> {
        use RewardRouterError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::InvalidAmount => InvalidAmount {}.encode(),
            E::InvalidValue => InvalidValue {}.encode(),
            E::InvalidOlpAmount => InvalidOlpAmount {}.encode(),
        }
    }
}

sol_interface! {
    interface IRewardRouter {
        function init(address gov, address omx, address es_omx, address bn_omx, address olp, address staked_omx_tracker, address bonus_omx_tracker, address staked_omx_tracker_staking, address bonus_omx_tracker_staking, address fee_omx_tracker_staking, address fee_olp_tracker_staking, address staked_olp_tracker_staking, address olp_manager) external;

        function setGov(address gov) external;

        function batchStakeOmxForAccount(address[] memory accounts, uint256[] memory amounts) external;

        function stakeOmxForAccount(address account, uint256 amount) external;

        function stakeOmx(uint256 amount) external;

        function stakeEsOmx(uint256 amount) external;

        function unstakeOmx(uint256 amount) external;

        function unstakeEsOmx(uint256 amount) external;

        function mintAndStakeOlp(address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256);

        function unstakeAndRedeemOlp(address token_out, uint256 olp_amount, uint256 min_out, address receiver) external returns (uint256);

        function claim() external;

        function claimEsOmx() external;

        function claimFees() external;

        function compound() external;

        function compoundForAccount(address account) external;

        function batchCompoundForAccounts(address[] memory accounts) external;
    }
}
