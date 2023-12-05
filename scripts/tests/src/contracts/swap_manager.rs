use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    SwapManager,
    r#"[
        function init(address gov, address usdo, address vault, address fee_manager, address funding_rate_manager) external
        function setManager(address account, bool is_manager) external
        function setInManagerMode(bool in_manager_mode) external
        function usdoAmount(address token) external view returns (uint256)
        function buyUsdo(address token, address receiver) external returns (uint256)
        function sellUsdo(address token, address receiver) external returns (uint256)
        function swap(address token_in, address token_out, address receiver) external returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct SwapManagerInitArgs {
    pub gov: Address,
    pub usdo: Address,
    pub vault: Address,
    pub fee_manager: Address,
    pub funding_rate_manager: Address,
}

impl SwapManagerInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> SwapManager<TestClient> {
        let contract = SwapManager::new(addr, gov.clone());

        contract
            .init(
                self.gov,
                self.usdo,
                self.vault,
                self.fee_manager,
                self.funding_rate_manager,
            )
            .await
            .unwrap();

        contract
    }
}
