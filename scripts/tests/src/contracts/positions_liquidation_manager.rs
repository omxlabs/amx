use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    PositionsLiquidationManager,
    r#"[
        function init(address gov, address vault, address vault_utils, address fee_manager, address funding_rate_manager, address positions_manager, address positions_manager_utils, address positions_decrease_manager) external
        function setGov(address gov) external
        function setLiquidator(address liquidator, bool is_active) external
        function liquidatePosition(address account, address collateral_token, address index_token, bool is_long, address fee_receiver) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsLiquidationManagerInitArgs {
    pub gov: Address,
    pub vault: Address,
    pub vault_utils: Address,
    pub fee_manager: Address,
    pub funding_rate_manager: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
    pub positions_decrease_manager: Address,
}

impl PositionsLiquidationManagerInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> PositionsLiquidationManager<TestClient> {
        let contract = PositionsLiquidationManager::new(addr, gov.clone());

        contract
            .init(
                self.gov,
                self.vault,
                self.vault_utils,
                self.fee_manager,
                self.funding_rate_manager,
                self.positions_manager,
                self.positions_manager_utils,
                self.positions_decrease_manager,
            )
            .await
            .unwrap();

        contract
    }
}
