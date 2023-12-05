extern crate alloc;

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    event Claim(address receiver, uint256 amount);

    error Forbidden();
    error AlreadyInitialized();
    error NotInitialized();
    error MintToZeroAddress();
    error BurnFromZeroAddress();
    error TransferFromZeroAddress();
    error TransferToZeroAddress();
    error ApproveFromZeroAddress();
    error ApproveToZeroAddress();
    error InvalidZeroAmount();
    error InvalidDepositToken();
    error ActionNotEnabled();
    error AmountExceedsStakedAmount(uint256 staked_amount, uint256 amount);
    error AmountExceedsDepositBalance(uint256 deposit_balance, uint256 amount);
    error TransferAmountExceedsAllowance(uint256 allowance, uint256 amount);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RewardTrackerError {
    Forbidden,
    AlreadyInitialized,
    NotInitialized,
    MintToZeroAddress,
    BurnFromZeroAddress,
    TransferFromZeroAddress,
    TransferToZeroAddress,
    ApproveFromZeroAddress,
    ApproveToZeroAddress,
    InvalidZeroAmount,
    InvalidDepositToken,
    ActionNotEnabled,
    AmountExceedsStakedAmount { staked_amount: U256, amount: U256 },
    AmountExceedsDepositBalance { deposit_balance: U256, amount: U256 },
    TransferAmountExceedsAllowance { allowance: U256, amount: U256 },
}

impl From<RewardTrackerError> for Vec<u8> {
    fn from(err: RewardTrackerError) -> Vec<u8> {
        use RewardTrackerError as E;
        match err {
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::MintToZeroAddress => MintToZeroAddress {}.encode(),
            E::BurnFromZeroAddress => BurnFromZeroAddress {}.encode(),
            E::TransferFromZeroAddress => TransferFromZeroAddress {}.encode(),
            E::TransferToZeroAddress => TransferToZeroAddress {}.encode(),
            E::ApproveFromZeroAddress => ApproveFromZeroAddress {}.encode(),
            E::ApproveToZeroAddress => ApproveToZeroAddress {}.encode(),
            E::InvalidZeroAmount => InvalidZeroAmount {}.encode(),
            E::InvalidDepositToken => InvalidDepositToken {}.encode(),
            E::ActionNotEnabled => ActionNotEnabled {}.encode(),
            E::AmountExceedsStakedAmount {
                staked_amount,
                amount,
            } => AmountExceedsStakedAmount {
                staked_amount,
                amount,
            }
            .encode(),
            E::AmountExceedsDepositBalance {
                deposit_balance,
                amount,
            } => AmountExceedsDepositBalance {
                deposit_balance,
                amount,
            }
            .encode(),
            E::TransferAmountExceedsAllowance { allowance, amount } => {
                TransferAmountExceedsAllowance { allowance, amount }.encode()
            }
        }
    }
}

sol_interface! {
    interface IRewardTracker {
        function init(address gov, address distributor, address reward_tracker_staking, string calldata name, string calldata symbol) external;

        function totalSupply() external view returns (uint256);

        function isHandler(address account) external view returns (bool);

        function setGov(address gov) external;

        function setInPrivateTransferMode(bool in_private_transfer_mode) external;

        function setHandler(address handler, bool is_active) external;

        function balanceOf(address account) external view returns (uint256);

        function transfer(address recipient, uint256 amount) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function tokensPerInterval() external view returns (uint256);

        function rewardToken() external view returns (address);

        function mintInternal(address account, uint256 amount) external;

        function burnInternal(address account, uint256 amount) external;
    }
}

sol_interface! {
    interface IRewardTrackerStaking {
        function init(address gov, address reward_tracker, address distributor, address[] memory deposit_tokens) external;

        function setGov(address gov) external;

        function setDepositToken(address deposit_token, bool is_deposit_token) external;

        function depositBalance(address account, address deposit_token) external view returns (uint256);

        function stake(address deposit_token, uint256 amount) external;

        function setInPrivateClaimingMode(bool in_private_claiming_mode) external;

        function setInPrivateStakingMode(bool in_private_staking_mode) external;

        function stakeForAccount(address funding_account, address account, address deposit_token, uint256 amount) external;

        function unstake(address deposit_token, uint256 amount) external;

        function unstakeForAccount(address account, address deposit_token, uint256 amount, address receiver) external;

        function updateRewards() external;

        function claim(address receiver) external returns (uint256);

        function claimForAccount(address account, address receiver) external returns (uint256);

        function claimable(address account) external view returns (uint256);

        function stakedAmount(address account) external view returns (uint256);

        function cumulativeReward(address account) external view returns (uint256);
    }
}
