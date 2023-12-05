use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    RewardTracker,
    r#"[
        event Claim(address receiver, uint256 amount)
        error Forbidden()
        error AlreadyInitialized()
        error NotInitialized()
        error MintToZeroAddress()
        error BurnFromZeroAddress()
        error TransferFromZeroAddress()
        error TransferToZeroAddress()
        error ApproveFromZeroAddress()
        error ApproveToZeroAddress()
        error InvalidZeroAmount()
        error InvalidDepositToken()
        error ActionNotEnabled()
        error AmountExceedsStakedAmount(uint256 staked_amount, uint256 amount)
        error AmountExceedsDepositBalance(uint256 deposit_balance, uint256 amount)
        error TransferAmountExceedsAllowance(uint256 allowance, uint256 amount)
        function init(address gov, address distributor, address reward_tracker_staking, string calldata name, string calldata symbol) external
        function totalSupply() external view returns (uint256)
        function setGov(address gov) external
        function setInPrivateTransferMode(bool in_private_transfer_mode) external
        function setHandler(address handler, bool is_active) external
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        function tokensPerInterval() external view returns (uint256)
        function rewardToken() external view returns (address)
        function mintInternal(address account, uint256 amount) external
        function burnInternal(address account, uint256 amount) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct RewardTrackerInitArgs {
    pub distributor: Address,
    pub reward_tracker_staking: Address,
    pub name: String,
    pub symbol: String,
}

impl RewardTrackerInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> RewardTracker<TestClient> {
        let contract = RewardTracker::new(addr, gov.clone());

        contract
            .init(
                gov.address(),
                self.distributor,
                self.reward_tracker_staking,
                self.name,
                self.symbol,
            )
            .await
            .unwrap();

        contract
    }
}
