extern crate alloc;

use stylus_sdk::stylus_proc::sol_interface;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WethError {
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

impl From<WethError> for Vec<u8> {
    fn from(err: WethError) -> Vec<u8> {
        use WethError as E;
        let err = match err {
            E::AlreadyInitialized => "already initialized",
            E::Forbidden => "forbidden",
            E::TransferFromZeroAddress => "transfer from zero address",
            E::TransferToZeroAddress => "transfer to zero address",
            E::InsufficientBalance => "insufficient balance",
            E::MintToZeroAddress => "mint to zero address",
            E::BurnFromZeroAddress => "burn from zero address",
            E::ApproveFromZeroAddress => "approve from zero address",
            E::ApproveToZeroAddress => "approve to zero address",
            E::InsufficientAllowance => "insufficient allowance",
            E::AllowanceBelowZero => "allowance below zero",
        };

        format!("Weth: {err}").into()
    }
}

sol_interface! {
    interface IWeth {
        function init(string calldata name, string calldata symbol) external;

        function deposit(address to) external payable;

        function depositApprove(address to) external payable;

        function withdraw(address to, uint256 amount) external;

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
