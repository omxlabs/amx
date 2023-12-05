use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    PositionsDecreaseRouter,
    r#"[
        function init(address weth, address vault, address positions_decrease_manager, address swap_router) external
        function decreasePosition(address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver, uint256 price) external returns (uint256)
        function decreasePositionEth(address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long, address receiver, uint256 price) external returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsDecreaseRouterInitArgs {
    pub weth: Address,
    pub vault: Address,
    pub positions_decrease_manager: Address,
    pub swap_router: Address,
}

impl PositionsDecreaseRouterInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> PositionsDecreaseRouter<TestClient> {
        let contract = PositionsDecreaseRouter::new(addr, gov.clone());

        contract
            .init(
                self.weth,
                self.vault,
                self.positions_decrease_manager,
                self.swap_router,
            )
            .await
            .unwrap();

        contract
    }
}
