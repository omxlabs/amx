extern crate alloc;

use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    interface IYieldToken {
        function init(string calldata name, string calldata symbol, address minter, uint256 initial_supply) external;

        function setGov(address gov) external;

        function setInfo(string calldata name, string calldata symbol) external;

        function setYieldTrackers(address[] memory yield_trackers) external;

        function addAdmin(address account) external;

        function removeAdmin(address account) external;

        function setInWhitelistMode(bool in_whitelist_mode) external;

        function setWhitelistedHandler(address handler, bool is_whitelisted) external;

        function addNonStakingAccount(address account) external;

        function removeNonStakingAccount(address account) external;

        function recoverClaim(address account, address receiver) external;

        function claim(address receiver) external;

        function totalStaked() external view returns (uint256);

        function balanceOf(address account) external view returns (uint256);

        function stakedBalance(address account) external view returns (uint256);

        function transfer(address recipient, uint256 amount) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function setMinter(address minter, bool is_active) external;

        function mint(address account, uint256 amount) external;

        function burn(address account, uint256 amount) external;
    }
}

sol! {
    error AllowanceOverflow();
    error AllowanceUnderflow();
    error TotalSupplyOverflow();
    error TotalSupplyUnderflow();
    error BalanceOverflow();
    error BalanceUnderflow();
    error NonStakingSupplyOverflow();
    error NonStakingSupplyUnderflow();
    error Forbidden();
    error NotInitialized();
    error AlreadyInitialized();
    error AccountAlreadyMarked();
    error AccountNotMarked();
    error TransferAmountExceedsAllowance();
    error MintToZeroAddress();
    error BurnFromZeroAddress();
    error BurnAmountExceedsBalance();
    error TransferFromZeroAddress();
    error TransferToZeroAddress();
    error SenderNotWhitelisted();
    error TransferAmountExceedsBalance();
    error ApproveFromZeroAddress();
    error ApproveToZeroAddress();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum YieldTokenError {
    AllowanceOverflow,
    AllowanceUnderflow,
    TotalSupplyOverflow,
    TotalSupplyUnderflow,
    BalanceOverflow,
    BalanceUnderflow,
    NonStakingSupplyOverflow,
    NonStakingSupplyUnderflow,

    Forbidden,
    NotInitialized,
    AlreadyInitialized,
    AccountAlreadyMarked,
    AccountNotMarked,
    TransferAmountExceedsAllowance,
    MintToZeroAddress,
    BurnFromZeroAddress,
    BurnAmountExceedsBalance,
    TransferFromZeroAddress,
    TransferToZeroAddress,
    SenderNotWhitelisted,
    TransferAmountExceedsBalance,
    ApproveFromZeroAddress,
    ApproveToZeroAddress,
}

impl From<YieldTokenError> for Vec<u8> {
    fn from(err: YieldTokenError) -> Vec<u8> {
        use YieldTokenError as E;
        match err {
            E::AccountAlreadyMarked => AccountAlreadyMarked {}.encode(),
            E::AccountNotMarked => AccountNotMarked {}.encode(),
            E::TransferAmountExceedsAllowance => TransferAmountExceedsAllowance {}.encode(),
            E::TransferAmountExceedsBalance => TransferAmountExceedsBalance {}.encode(),
            E::TransferFromZeroAddress => TransferFromZeroAddress {}.encode(),
            E::TransferToZeroAddress => TransferToZeroAddress {}.encode(),
            E::MintToZeroAddress => MintToZeroAddress {}.encode(),
            E::BurnFromZeroAddress => BurnFromZeroAddress {}.encode(),
            E::BurnAmountExceedsBalance => BurnAmountExceedsBalance {}.encode(),
            E::ApproveFromZeroAddress => ApproveFromZeroAddress {}.encode(),
            E::ApproveToZeroAddress => ApproveToZeroAddress {}.encode(),
            E::BalanceOverflow => BalanceOverflow {}.encode(),
            E::BalanceUnderflow => BalanceUnderflow {}.encode(),
            E::NonStakingSupplyOverflow => NonStakingSupplyOverflow {}.encode(),
            E::NonStakingSupplyUnderflow => NonStakingSupplyUnderflow {}.encode(),
            E::TotalSupplyOverflow => TotalSupplyOverflow {}.encode(),
            E::TotalSupplyUnderflow => TotalSupplyUnderflow {}.encode(),
            E::Forbidden => Forbidden {}.encode(),
            E::NotInitialized => NotInitialized {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::SenderNotWhitelisted => SenderNotWhitelisted {}.encode(),
            E::AllowanceOverflow => AllowanceOverflow {}.encode(),
            E::AllowanceUnderflow => AllowanceUnderflow {}.encode(),
        }
    }
}
