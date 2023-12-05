use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    OrderbookSwap,
    r#"[
        function init(address gov, address swap_router) external
        function getCurrentIndex(address account) external view returns (uint256)
        function createSwapOrder(address token_in, address token_out, uint256 amount_in, uint256 min_out, uint256 trigger_ratio, bool trigger_above_threshold, uint256 execution_fee) external payable
        function cancelSwapOrder(uint256 order_index) external
        function getSwapOrder(address account, uint256 order_index) external view returns (address, address, address, uint256, uint256, uint256, bool, uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct OrderbookSwapInitArgs {
    pub gov: Address,
    pub swap_router: Address,
}

impl OrderbookSwapInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> OrderbookSwap<TestClient> {
        let contract = OrderbookSwap::new(addr, gov.clone());

        contract.init(self.gov, self.swap_router).await.unwrap();

        contract
    }
}
