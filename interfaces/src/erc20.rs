extern crate alloc;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{
    call::{Error as CallError, NonPayableCallContext},
    stylus_proc::sol_interface,
};

sol! {
    /// Emitted when `value` tokens are moved from one account (`from`) to
    /// another (`to`).
    ///
    /// Note that `value` may be zero.
    event Transfer(address indexed from, address indexed to, uint256 value);

    /// Emitted when the allowance of a `spender` for an `owner` is set by
    /// a call to [approve]. `value` is the new allowance.
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error SafeTransferFailed();
    error SafeTransferFromFailed();

    error AlreadyInitialized();
    error BurnFromZeroAddress();
    error MintToZeroAddress();
    error ApproveFromZeroAddress();
    error ApproveToZeroAddress();
    error TransferFromZeroAddress();
    error TransferToZeroAddress();
    error InsufficientBalance();
    error InsufficientAllowance();
    error AllowanceBelowZero();
    error Forbidden();
}

sol_interface! {
    interface IErc20 {

        function init(address gov, string calldata name, string calldata symbol, uint8 decimals) external;

        function setMinter(address minter, bool is_active) external;

        function setGov(address gov) external;

        function mint(address account, uint256 amount) external;

        function burn(uint256 amount) external;

        function name() external view returns (string memory);

        function symbol() external view returns (string memory);

        function decimals() external view returns (uint8);

        function totalSupply() external view returns (uint256);

        function balanceOf(address account) external view returns (uint256);

        function transfer(address recipient, uint256 amount) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);

        function increaseAllowance(address spender, uint256 added_value) external returns (bool);

        function decreaseAllowance(address spender, uint256 subtracted_value) external returns (bool);
    }
}

impl From<Address> for IErc20 {
    fn from(addr: Address) -> Self {
        Self::new(addr)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Erc20Error {
    AlreadyInitialized,
    BurnFromZeroAddress,
    MintToZeroAddress,
    ApproveFromZeroAddress,
    ApproveToZeroAddress,
    TransferFromZeroAddress,
    TransferToZeroAddress,
    InsufficientBalance,
    InsufficientAllowance,
    AllowanceBelowZero,
    Forbidden,
}

impl From<Erc20Error> for Vec<u8> {
    fn from(err: Erc20Error) -> Vec<u8> {
        use Erc20Error as E;
        match err {
            E::AlreadyInitialized => AlreadyInitialized {}.encode(),
            E::Forbidden => Forbidden {}.encode(),
            E::TransferFromZeroAddress => TransferFromZeroAddress {}.encode(),
            E::TransferToZeroAddress => TransferToZeroAddress {}.encode(),
            E::InsufficientBalance => InsufficientBalance {}.encode(),
            E::MintToZeroAddress => MintToZeroAddress {}.encode(),
            E::BurnFromZeroAddress => BurnFromZeroAddress {}.encode(),
            E::ApproveFromZeroAddress => ApproveFromZeroAddress {}.encode(),
            E::ApproveToZeroAddress => ApproveToZeroAddress {}.encode(),
            E::InsufficientAllowance => InsufficientAllowance {}.encode(),
            E::AllowanceBelowZero => AllowanceBelowZero {}.encode(),
        }
    }
}

#[derive(Debug)]
pub enum SafeTransferError {
    TransferFailed,
    TransferFromFailed,
    CallError(CallError),
}

impl From<CallError> for SafeTransferError {
    fn from(err: CallError) -> Self {
        Self::CallError(err)
    }
}

impl From<SafeTransferError> for Vec<u8> {
    fn from(err: SafeTransferError) -> Vec<u8> {
        use SafeTransferError as E;
        match err {
            E::TransferFailed => SafeTransferFailed {}.encode(),
            E::TransferFromFailed => SafeTransferFromFailed {}.encode(),
            E::CallError(err) => err.into(),
        }
    }
}

pub fn safe_transfer(
    ctx: impl NonPayableCallContext,
    token: impl Into<IErc20>,
    recipient: Address,
    amount: U256,
) -> Result<(), SafeTransferError> {
    let token: IErc20 = token.into();

    if token.transfer(ctx, recipient, amount)? {
        Ok(())
    } else {
        Err(SafeTransferError::TransferFailed)
    }
}

pub fn safe_transfer_from(
    ctx: impl NonPayableCallContext,
    token: impl Into<IErc20>,
    sender: Address,
    recipient: Address,
    amount: U256,
) -> Result<(), SafeTransferError> {
    let token: IErc20 = token.into();

    if token.transfer_from(ctx, sender, recipient, amount)? {
        Ok(())
    } else {
        Err(SafeTransferError::TransferFailed)
    }
}
