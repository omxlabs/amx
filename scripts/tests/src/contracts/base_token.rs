use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    BaseToken,
    r#"[
        error Forbidden()
        error AlreadyInitialized()
        error AccountAlreadyMarked(address account)
        error AccountNotMarked(address account)
        error TransferAmountExceedsAllowance(uint256 amount, uint256 allowance, address owner, address spender)
        error TransferAmountExceedsBalance(uint256 amount, uint256 balance, address account)
        error TransferFromZeroAddress()
        error TransferToZeroAddress()
        error MintToZeroAddress()
        error BurnFromZeroAddress()
        error BurnAmountExceedsBalance(uint256 amount, uint256 balance, address account)
        error SenderNotWhitelisted()
        error ApproveFromZeroAddress()
        error ApproveToZeroAddress()
        error BalanceOverflow()
        error BalanceUnderflow()
        error NonStakingSupplyOverflow()
        error NonStakingSupplyUnderflow()
        error TotalSupplyOverflow()
        error TotalSupplyUnderflow()
        function init(address gov, string calldata name, string calldata symbol) external
        function setGov(address gov) external
        function setMinter(address minter, bool is_active) external
        function mint(address account, uint256 amount) external
        function burn(address account, uint256 amount) external
        function setInfo(string calldata name, string calldata symbol) external
        function setYieldTrackers(address[] memory yield_trackers) external
        function addAdmin(address account) external
        function removeAdmin(address account) external
        function setInPrivateTransferMode(bool in_private_transfer_mode) external
        function setHandler(address handler, bool is_active) external
        function addNonStakingAccount(address account) external
        function removeNonStakingAccount(address account) external
        function recoverClaim(address account, address receiver) external
        function claim(address receiver) external
        function totalSupply() external view returns (uint256)
        function totalStaked() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function stakedBalance(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
    ]"#
);

#[derive(Clone, Debug)]
pub struct BaseTokenInitArgs {
    pub gov: Address,
    pub name: String,
    pub symbol: String,
}

impl BaseTokenInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> BaseToken<TestClient> {
        let contract = BaseToken::new(addr, gov.clone());

        contract
            .init(self.gov, self.name, self.symbol)
            .await
            .unwrap();

        contract
    }
}
