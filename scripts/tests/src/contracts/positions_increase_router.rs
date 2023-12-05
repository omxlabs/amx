use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    PositionsIncreaseRouter,
    r#"[
        function init(address weth, address vault, address positions_increase_manager, address swap_router) external
        function increasePosition(address[] memory path, address index_token, uint256 amount_in, uint256 min_out, uint256 size_delta, bool is_long, uint256 price) external
        function increasePositionEth(address[] memory path, address index_token, uint256 min_out, uint256 size_delta, bool is_long, uint256 price) external payable
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsIncreaseRouterInitArgs {
    pub weth: Address,
    pub vault: Address,
    pub positions_increase_manager: Address,
    pub swap_router: Address,
}

impl PositionsIncreaseRouterInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> PositionsIncreaseRouter<TestClient> {
        let contract = PositionsIncreaseRouter::new(addr, gov.clone());

        contract
            .init(
                self.weth,
                self.vault,
                self.positions_increase_manager,
                self.swap_router,
            )
            .await
            .unwrap();

        contract
    }
}
