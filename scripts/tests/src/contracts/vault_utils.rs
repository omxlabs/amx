use std::sync::Arc;

use ethers::{
    prelude::abigen,
    types::{Address, U256},
};

use crate::stylus_testing::provider::TestClient;

abigen!(
    VaultUtils,
    r#"[
        function init(address vault, uint256 min_profit_time) external
        function getUtilization(address token) external view returns (uint256)
        function validateTokens(address collateral_token, address index_token, bool is_long) external view
        function tokenToUsd(address token, uint256 token_amount) external view returns (uint256)
        function usdToToken(address token, uint256 usd_amount) external view returns (uint256)
        function getDelta(address index_token, uint256 size, uint256 average_price, bool is_long, uint256 last_increased_time) external view returns (bool, uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct VaultUtilsInitArgs {
    pub vault: Address,
    pub min_profit_time: U256,
}

impl VaultUtilsInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> VaultUtils<TestClient> {
        let contract = VaultUtils::new(addr, gov.clone());

        contract
            .init(self.vault, self.min_profit_time)
            .await
            .unwrap();

        contract
    }
}
