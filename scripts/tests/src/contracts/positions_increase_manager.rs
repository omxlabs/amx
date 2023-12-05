use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    PositionsIncreaseManager,
    r#"[
        function init(address gov, address vault, address vault_utils, address fee_manager, address funding_rate_manager, address increase_router, address positions_manager, address positions_manager_utils) external
        function setGov(address gov) external
        function increasePosition(address account, address collateral_token, address index_token, uint256 size_delta, bool is_long) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsIncreaseManagerInitArgs {
    pub gov: Address,
    pub vault: Address,
    pub vault_utils: Address,
    pub fee_manager: Address,
    pub funding_rate_manager: Address,
    pub increase_router: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
}

impl PositionsIncreaseManagerInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> PositionsIncreaseManager<TestClient> {
        let contract = PositionsIncreaseManager::new(addr, gov.clone());

        contract
            .init(
                self.gov,
                self.vault,
                self.vault_utils,
                self.fee_manager,
                self.funding_rate_manager,
                self.increase_router,
                self.positions_manager,
                self.positions_manager_utils,
            )
            .await
            .unwrap();

        contract
    }
}
