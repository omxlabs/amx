use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    PositionsDecreaseManager,
    r#"[
        function init(address gov, address vault, address funding_rate_manager, address decrease_router, address positions_manager, address positions_liquidation_manager, address positions_manager_utils) external
        function setGov(address gov) external
        function decreasePosition(address account, address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver) external returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsDecreaseManagerInitArgs {
    pub gov: Address,
    pub vault: Address,
    pub funding_rate_manager: Address,
    pub decrease_router: Address,
    pub positions_manager: Address,
    pub positions_liquidation_manager: Address,
    pub positions_manager_utils: Address,
}

impl PositionsDecreaseManagerInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> PositionsDecreaseManager<TestClient> {
        let contract = PositionsDecreaseManager::new(addr, gov.clone());

        contract
            .init(
                self.gov,
                self.vault,
                self.funding_rate_manager,
                self.decrease_router,
                self.positions_manager,
                self.positions_liquidation_manager,
                self.positions_manager_utils,
            )
            .await
            .unwrap();

        contract
    }
}
