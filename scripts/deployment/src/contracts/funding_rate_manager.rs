use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    FundingRateManager,
    r#"[
        function init(address gov, address vault) external
        function setGov(address gov) external
        function cumulativeFundingRate(address token) external view returns (uint256)
        function getNextFundingRate(address token) external view returns (uint256)
        function updateCumulativeFundingRate(address collateral_token) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct FundingRateManagerInitArgs {
    pub gov: Address,
    pub vault: Address,
}

impl FundingRateManagerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> FundingRateManager<LiveClient> {
        let funding_rate_manager = FundingRateManager::new(addr, ctx.client.clone());

        send(funding_rate_manager.init(self.gov, self.vault))
            .await
            .unwrap();

        funding_rate_manager
    }

}
