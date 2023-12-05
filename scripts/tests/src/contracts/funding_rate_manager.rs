use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    FundingRateManager,
    r#"[
        function init(address gov, address vault) external
        function setGov(address gov) external
        function cumulativeFundingRate(address token) external view returns (uint256)
        function getNextFundingRate(address token) external view returns (uint256)
        function updateCumulativeFundingRate(address collateral_token) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct FundingRateManagerInitArgs {
    pub gov: Address,
    pub vault: Address,
}

impl FundingRateManagerInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> FundingRateManager<TestClient> {
        let contract = FundingRateManager::new(addr, gov.clone());

        contract.init(self.gov, self.vault).await.unwrap();

        contract
    }
}
