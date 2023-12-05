use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    PositionsManagerUtils,
    r#"[
        function init(address positions_manager, address positions_decrease_manager, address vault, address fee_manager, address vault_utils) external
        function reduceCollateral(address account, address collateral_token, address index_token, uint256 collateral_delta, uint256 size_delta, bool is_long) external returns (uint256, uint256)
        function validateLiquidation(address account, address collateral_token, address index_token, bool is_long, bool raise) external view returns (uint256, uint256)
        function getNextAveragePrice(address index_token, uint256 size, uint256 average_price, bool is_long, uint256 next_price, uint256 size_delta, uint256 last_increased_time) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct PositionsManagerUtilsInitArgs {
    pub positions_manager: Address,
    pub positions_decrease_manager: Address,
    pub vault: Address,
    pub fee_manager: Address,
    pub vault_utils: Address,
}

impl PositionsManagerUtilsInitArgs {
    pub async fn init(
        self,
        ctx: &DeployContext,
        addr: Address,
    ) -> PositionsManagerUtils<LiveClient> {
        let positions_manager_utils = PositionsManagerUtils::new(addr, ctx.client.clone());

        send(positions_manager_utils.init(
            self.positions_manager,
            self.positions_decrease_manager,
            self.vault,
            self.fee_manager,
            self.vault_utils,
        ))
        .await
        .unwrap();

        positions_manager_utils
    }
}
