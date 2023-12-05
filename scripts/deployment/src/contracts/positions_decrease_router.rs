use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

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
        ctx: &DeployContext,
        addr: Address,
    ) -> PositionsDecreaseRouter<LiveClient> {
        let positions_decrease_router = PositionsDecreaseRouter::new(addr, ctx.client.clone());

        send(positions_decrease_router.init(
            self.weth,
            self.vault,
            self.positions_decrease_manager,
            self.swap_router,
        ))
        .await
        .unwrap();

        positions_decrease_router
    }
}
