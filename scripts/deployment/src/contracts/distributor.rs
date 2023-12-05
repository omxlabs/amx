use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    Distributor,
    r#"[
        function init() external
        function setGov(address gov) external
        function setTokensPerInterval(address receiver, uint256 amount) external
        function updateLastDistributionTime(address receiver) external
        function setDistribution(address[] memory receivers, uint256[] memory amounts, address[] memory reward_tokens) external
        function distribute() external returns (uint256)
        function getRewardToken(address receiver) external view returns (address)
        function getDistributionAmount(address receiver) external view returns (uint256)
        function getIntervals(address receiver) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct DistributorInitArgs {}

impl DistributorInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> Distributor<LiveClient> {
        let contract = Distributor::new(addr, ctx.client());

        send(contract.init()).await.unwrap();

        contract
    }
}
