use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    OlpManagerUtils,
    r#"[
        function init(address vault, address positions_manager, address shorts_tracker, address olp) external
        function setShortsTrackerAveragePriceWeight(uint256 weight) external
        function getPrice() external view returns (uint256)
        function getAumInUsdo() external view returns (uint256)
        function getAum() external view returns (uint256)
        function getGlobalShortDelta(address token, uint256 price, uint256 size) external view returns (uint256, bool)
        function getGlobalShortAveragePrice(address token) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct OlpManagerUtilsInitArgs {
    pub gov: Address,
    pub vault: Address,
    pub positions_manager: Address,
    pub shorts_tracker: Address,
    pub olp: Address,
}

impl OlpManagerUtilsInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> OlpManagerUtils<LiveClient> {
        let contract = OlpManagerUtils::new(addr, ctx.client());

        send(contract.init(
            self.vault,
            self.positions_manager,
            self.shorts_tracker,
            self.olp,
        ))
        .await
        .unwrap();

        contract
    }
}
