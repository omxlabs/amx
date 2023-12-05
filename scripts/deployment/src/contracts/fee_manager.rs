use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    FeeManager,
    r#"[
        function init(address gov, address usdo, address vault, address funding_rate_manager, address swap_manager, address positions_manager, address positions_manager_utils, address positions_increase_manager, address positions_decrease_manager, address positions_liquidation_manager) external
        function setGov(address gov) external
        function increaseFeeReserves(address token, uint256 amount) external
        function getFeeReserve(address token) external view returns (uint256)
        function collectSwapFees(address token, uint256 amount, uint256 fee_basis_points) external returns (uint256)
        function getPositionFee(uint256 size_delta) external view returns (uint256)
        function getFundingFee(address collateral_token, uint256 size, uint256 entry_funding_rate) external view returns (uint256)
        function collectMarginFees(address collateral_token, uint256 size_delta, uint256 size, uint256 entry_funding_rate) external returns (uint256)
        function withdrawFees(address token, address receiver) external returns (uint256)
        function getSwapFeeBasisPoints(address token_in, address token_out) external view returns (uint256)
        function getTargetUsdoAmount(address token) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct FeeManagerInitArgs {
    pub gov: Address,
    pub usdo: Address,
    pub vault: Address,
    pub funding_rate_manager: Address,
    pub swap_manager: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
    pub positions_increase_manager: Address,
    pub positions_decrease_manager: Address,
    pub positions_liquidation_manager: Address,
}

impl FeeManagerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> FeeManager<LiveClient> {
        let fee_manager = FeeManager::new(addr, ctx.client.clone());

        send(fee_manager.init(
            self.gov,
            self.usdo,
            self.vault,
            self.funding_rate_manager,
            self.swap_manager,
            self.positions_manager,
            self.positions_manager_utils,
            self.positions_increase_manager,
            self.positions_decrease_manager,
            self.positions_liquidation_manager,
        ))
        .await
        .unwrap();

        fee_manager
    }
}
