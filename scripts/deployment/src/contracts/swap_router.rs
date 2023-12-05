use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    SwapRouter,
    r#"[
        function init(address weth, address usdo, address vault, address swap_manager, address positions_router) external
        function directPoolDeposit(address token, uint256 amount) external
        function swapForPosition(address[] memory path, uint256 min_out, address receiver) external returns (uint256)
        function swap(address[] memory path, uint256 amount_in, uint256 min_out, address receiver) external
        function swapEthToTokens(address[] memory path, uint256 min_out, address receiver) external payable
        function swapToEth(address token_in, uint256 amount_in, uint256 min_out, address receiver) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct SwapRouterInitArgs {
    pub weth: Address,
    pub usdo: Address,
    pub vault: Address,
    pub swap_manager: Address,
    pub positions_router: Address,
}

impl SwapRouterInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> SwapRouter<LiveClient> {
        let swap_router = SwapRouter::new(addr, ctx.client.clone());

        send(swap_router.init(
            self.weth,
            self.usdo,
            self.vault,
            self.swap_manager,
            self.positions_router,
        ))
        .await
        .unwrap();

        swap_router
    }
}
