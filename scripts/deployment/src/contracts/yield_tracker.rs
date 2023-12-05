use ethers::{
    prelude::abigen,
    types::{Address, U256},
};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    YieldTracker,
    r#"[
        function init(address gov, address yield_token) external
        function setGov(address gov) external
        function setDistributor(address distributor) external
        function claim(address account, address receiver) external returns (uint256)
        function getTokensPerInterval() external view returns (uint256)
        function claimable(address account) external view returns (uint256)
        function updateRewards(address account) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct YieldTrackerInitArgs {
    pub gov: Address,
    pub yield_token: Address,
}

impl YieldTrackerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> YieldTracker<LiveClient> {
        let contract = YieldTracker::new(addr, ctx.client());

        send(contract.init(self.gov, self.yield_token))
            .await
            .unwrap();

        contract
    }
}
