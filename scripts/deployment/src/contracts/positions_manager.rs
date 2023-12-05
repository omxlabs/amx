use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    PositionsManager,
    r#"[
        function init(address gov, address vault_utils, address fee_manager, address positions_decrease_manager, address positions_increase_manager, address positions_liquidation_manager, address positions_manager_utils) external
        function updateGuaranteedUsd(address token, int256 value) external
        function afterShortIncrease(address index_token, uint256 price, uint256 size_delta) external
        function decreaseGlobalShortSize(address token, uint256 amount) external
        function setGov(address gov) external
        function position(address account, address collateral_token, address index_token, bool is_long) external view returns (uint256, uint256, uint256, uint256, uint256, int256, uint256)
        function positionUpdate(address account, address collateral_token, address index_token, bool is_long, uint256 size, uint256 collateral, uint256 average_price, uint256 entry_funding_rate, uint256 reserve_amount, int256 realised_pnl, uint256 last_increased_time) external
        function getNextGlobalShortAveragePrice(address index_token, uint256 next_price, uint256 size_delta) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsManagerInitArgs {
    pub gov: Address,
    pub vault_utils: Address,
    pub fee_manager: Address,
    pub positions_decrease_manager: Address,
    pub positions_increase_manager: Address,
    pub positions_liquidation_manager: Address,
    pub positions_manager_utils: Address,
}

impl PositionsManagerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> PositionsManager<LiveClient> {
        let positions_manager = PositionsManager::new(addr, ctx.client.clone());

        send(positions_manager.init(
            self.gov,
            self.vault_utils,
            self.fee_manager,
            self.positions_decrease_manager,
            self.positions_increase_manager,
            self.positions_liquidation_manager,
            self.positions_manager_utils,
        ))
        .await
        .unwrap();

        positions_manager
    }
}
