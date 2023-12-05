use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    OrderbookIncrease,
    r#"[
        function init(address gov, address increase_router) external
        function getCurrentIndex(address account) external view returns (uint256)
        function createIncreaseOrder(uint256 collateral_amount, address collateral_token, address index_token, uint256 size_delta, bool is_long, uint256 trigger_price, bool trigger_above_threshold, uint256 execution_fee) external payable
        function cancelIncreaseOrder(uint256 order_index) external
        function getIncreaseOrder(address account, uint256 order_index) external view returns (address, uint256, address, address, uint256, bool, uint256, bool, uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct OrderbookIncreaseInitArgs {
    pub gov: Address,
    pub swap_router: Address,
}

impl OrderbookIncreaseInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> OrderbookIncrease<LiveClient> {
        let contract = OrderbookIncrease::new(addr, ctx.client.clone());

        send(contract.init(self.gov, self.swap_router))
            .await
            .unwrap();

        contract
    }
}
