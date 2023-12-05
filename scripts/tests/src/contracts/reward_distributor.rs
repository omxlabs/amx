use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    RewardDistributor,
    r#"[
        event Distribute(uint256 amount)
        event TokensPerIntervalChange(uint256 amount)
        error Forbidden()
        error AlreadyInitialized()
        error NotInitialized()
        error InvalidSender()
        error ZeroLastDistributionTime()
        function init(address gov, address reward_token, address reward_tracker, address reward_tracker_staking) external
        function setGov(address gov) external
        function setAdmin(address admin) external
        function updateLastDistributionTime() external
        function setTokensPerInterval(uint256 amount) external
        function pendingRewards() external view returns (uint256)
        function distribute() external returns (uint256)
        function rewardToken() external view returns (address)
        function tokensPerInterval() external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct RewardDistributorInitArgs {
    pub reward_token: Address,
    pub reward_tracker: Address,
    pub reward_tracker_staking: Address,
}

impl RewardDistributorInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> RewardDistributor<TestClient> {
        let contract = RewardDistributor::new(addr, gov.clone());

        contract
            .init(
                gov.address(),
                self.reward_token,
                self.reward_tracker,
                self.reward_tracker_staking,
            )
            .await
            .unwrap();

        contract
    }
}
