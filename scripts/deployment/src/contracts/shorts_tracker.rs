use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    ShortsTracker,
    r#"[
        function init(address gov, address vault, address vault_utils, address positions_manager) external
        function setGov(address gov) external
        function setHandler(address handler, bool is_active) external
        function setIsGlobalShortDataReady(bool value) external
        function updateGlobalShortData(address account, address collateral_token, address index_token, bool is_long, uint256 size_delta, uint256 mark_price, bool is_increase) external
        function getGlobalShortDelta(address token) external view returns (bool, uint256)
        function setInitData(address[] memory tokens, uint256[] memory average_prices) external
        function getNextGlobalShortData(address account, address collateral_token, address index_token, uint256 next_price, uint256 size_delta, bool is_increase) external view returns (uint256, uint256)
        function getRealisedPnl(address account, address collateral_token, address index_token, uint256 size_delta, bool is_increase) external view returns (int256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct ShortsTrackerInitArgs {
    pub gov: Address,
    pub vault: Address,
    pub vault_utils: Address,
    pub positions_manager: Address,
}

impl ShortsTrackerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> ShortsTracker<LiveClient> {
        let contract = ShortsTracker::new(addr, ctx.client());

        send(contract.init(
            self.gov,
            self.vault,
            self.vault_utils,
            self.positions_manager,
        ))
        .await
        .unwrap();

        contract
    }
}
