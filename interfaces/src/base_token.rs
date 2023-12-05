extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::stylus_proc::sol_interface;

sol! {
    error Forbidden();
    error AlreadyInitialized();
    error AccountAlreadyMarked(address account);
    error AccountNotMarked(address account);
    error TransferAmountExceedsAllowance(uint256 amount, uint256 allowance, address owner, address spender);
    error TransferAmountExceedsBalance(uint256 amount, uint256 balance, address account);
    error TransferFromZeroAddress();
    error TransferToZeroAddress();
    error MintToZeroAddress();
    error BurnFromZeroAddress();
    error BurnAmountExceedsBalance(uint256 amount, uint256 balance, address account);
    error SenderNotWhitelisted();
    error ApproveFromZeroAddress();
    error ApproveToZeroAddress();
    error BalanceOverflow();
    error BalanceUnderflow();
    error NonStakingSupplyOverflow();
    error NonStakingSupplyUnderflow();
    error TotalSupplyOverflow();
    error TotalSupplyUnderflow();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BaseTokenError {
    Forbidden,
    AlreadyInitialized,
    AccountAlreadyMarked {
        account: Address,
    },
    AccountNotMarked {
        account: Address,
    },
    TransferAmountExceedsAllowance {
        amount: U256,
        allowance: U256,
        owner: Address,
        spender: Address,
    },
    TransferAmountExceedsBalance {
        amount: U256,
        balance: U256,
        account: Address,
    },
    TransferFromZeroAddress,
    TransferToZeroAddress,
    MintToZeroAddress,
    BurnFromZeroAddress,
    BurnAmountExceedsBalance {
        amount: U256,
        balance: U256,
        account: Address,
    },
    SenderNotWhitelisted,
    ApproveFromZeroAddress,
    ApproveToZeroAddress,
    BalanceOverflow,
    BalanceUnderflow,
    NonStakingSupplyOverflow,
    NonStakingSupplyUnderflow,
    TotalSupplyOverflow,
    TotalSupplyUnderflow,
}

impl From<BaseTokenError> for Vec<u8> {
    fn from(err: BaseTokenError) -> Vec<u8> {
        use BaseTokenError as E;
        match err {
            E::AccountAlreadyMarked { account } => AccountAlreadyMarked { account }.encode(),
            E::AccountNotMarked { account } => AccountNotMarked { account }.encode(),
            E::TransferAmountExceedsAllowance {
                amount,
                allowance,
                owner,
                spender,
            } => TransferAmountExceedsAllowance {
                amount,
                allowance,
                owner,
                spender,
            }
            .encode(),
            E::TransferAmountExceedsBalance {
                amount,
                balance,
                account,
            } => TransferAmountExceedsBalance {
                amount,
                balance,
                account,
            }
            .encode(),
            E::SenderNotWhitelisted => SenderNotWhitelisted {}.encode(),
            E::BurnAmountExceedsBalance {
                amount,
                balance,
                account,
            } => BurnAmountExceedsBalance {
                amount,
                balance,
                account,
            }
            .encode(),
            E::ApproveFromZeroAddress => ApproveFromZeroAddress {}.encode(),
            E::ApproveToZeroAddress => ApproveToZeroAddress {}.encode(),
            E::BalanceOverflow => BalanceOverflow {}.encode(),
            E::BalanceUnderflow => BalanceUnderflow {}.encode(),
            E::NonStakingSupplyOverflow => NonStakingSupplyOverflow {}.encode(),
            E::NonStakingSupplyUnderflow => NonStakingSupplyUnderflow {}.encode(),
            E::TotalSupplyOverflow => TotalSupplyOverflow {}.encode(),
            E::TotalSupplyUnderflow => TotalSupplyUnderflow {}.encode(),
            E::TransferFromZeroAddress => TransferFromZeroAddress {}.encode(),
            E::TransferToZeroAddress => TransferToZeroAddress {}.encode(),
            E::MintToZeroAddress => MintToZeroAddress {}.encode(),
            E::BurnFromZeroAddress => BurnFromZeroAddress {}.encode(),
            E::Forbidden => Forbidden {}.encode(),
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
        }
    }
}

sol_interface! {
    interface IBaseToken {
        function init(address gov, string calldata name, string calldata symbol) external;

        function setGov(address gov) external;

        function setMinter(address minter, bool is_active) external;

        function mint(address account, uint256 amount) external;

        function burn(address account, uint256 amount) external;

        function setInfo(string calldata name, string calldata symbol) external;

        function setYieldTrackers(address[] memory yield_trackers) external;

        function addAdmin(address account) external;

        function removeAdmin(address account) external;

        function setInPrivateTransferMode(bool in_private_transfer_mode) external;

        function setHandler(address handler, bool is_active) external;

        function addNonStakingAccount(address account) external;

        function removeNonStakingAccount(address account) external;

        function recoverClaim(address account, address receiver) external;

        function claim(address receiver) external;

        function totalSupply() external view returns (uint256);

        function totalStaked() external view returns (uint256);

        function balanceOf(address account) external view returns (uint256);

        function stakedBalance(address account) external view returns (uint256);

        function transfer(address recipient, uint256 amount) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
    }
}
