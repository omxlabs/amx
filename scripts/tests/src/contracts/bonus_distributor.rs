use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    BonusDistributor,
    r#"[
        function init(address gov, address reward_token, address reward_tracker, address reward_tracker_staking) external
        function setGov(address gov) external
        function setAdmin(address admin) external
        function updateLastDistributionTime() external
        function setBonusMultiplier(uint256 bonus_multiplier_basis_points) external
        function tokensPerInterval() external view returns (uint256)
        function pendingRewards() external view returns (uint256)
        function distribute() external returns (uint256)
    ]"#,
);

#[derive(Clone, Debug)]
pub struct BonusDistributorInitArgs {
    pub reward_token: Address,
    pub reward_tracker: Address,
    pub reward_tracker_staking: Address,
}

impl BonusDistributorInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> BonusDistributor<TestClient> {
        let contract = BonusDistributor::new(addr, gov.clone());

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
