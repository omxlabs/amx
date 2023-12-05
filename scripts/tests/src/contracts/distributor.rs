use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    Distributor,
    r#"[
        function init() external
        function setGov(address gov) external
        function setTokensPerInterval(address receiver, uint256 amount) external
        function updateLastDistributionTime(address receiver) external
        function setDistribution(address[] memory receivers, uint256[] memory amounts, address[] memory reward_tokens) external
        function distribute() external returns (uint256)
        function getRewardToken(address receiver) external view returns (address)
        function getDistributionAmount(address receiver) external view returns (uint256)
        function getIntervals(address receiver) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct DistributorInitArgs {}

impl DistributorInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> Distributor<TestClient> {
        let contract = Distributor::new(addr, gov.clone());

        contract.init().await.unwrap();

        contract
    }
}
