use std::sync::Arc;

use ethers::{
    prelude::abigen,
    types::{Address, U256},
};

use crate::stylus_testing::provider::TestClient;

abigen!(
    VaultPriceFeed,
    r#"[
        function init(address gov, uint256 max_strict_price_deviation) external
        function setGov(address gov) external
        function setAdjustment(address token, bool is_additive, uint256 adjustment_bps) external
        function setPrice(address token, uint256 price) external
        function setSpreadBasisPoints(address token, uint256 spread_basis_points) external
        function setMaxStrictPriceDeviation(uint256 max_strict_price_deviation) external
        function setTokenConfig(address token, bool is_strict_stable) external
        function getPrice(address token, bool maximize) external view returns (uint256)
        function getPriceV1(address token, bool maximize) external view returns (uint256)
        function getPrimaryPrice(address token, bool _maximize) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct VaultPriceFeedInitArgs {
    pub gov: Address,
    pub max_strict_price_deviation: U256,
}

impl VaultPriceFeedInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> VaultPriceFeed<TestClient> {
        let contract = VaultPriceFeed::new(addr, gov.clone());

        contract
            .init(self.gov, self.max_strict_price_deviation)
            .await
            .unwrap();

        contract
    }
}
