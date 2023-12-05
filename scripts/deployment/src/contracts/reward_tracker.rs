use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    RewardTracker,
    r#"[
        function init(address gov, address distributor, address reward_tracker_staking, string calldata name, string calldata symbol) external
        function totalSupply() external view returns (uint256)
        function setGov(address gov) external
        function setInPrivateTransferMode(bool in_private_transfer_mode) external
        function setHandler(address handler, bool is_active) external
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        function tokensPerInterval() external view returns (uint256)
        function rewardToken() external view returns (address)
        function mintInternal(address account, uint256 amount) external
        function burnInternal(address account, uint256 amount) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct RewardTrackerInitArgs {
    pub gov: Address,
    pub distributor: Address,
    pub reward_tracker_staking: Address,
    pub name: String,
    pub symbol: String,
}

impl RewardTrackerInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> RewardTracker<LiveClient> {
        let contract = RewardTracker::new(addr, ctx.client());

        send(contract.init(
            self.gov,
            self.distributor,
            self.reward_tracker_staking,
            self.name,
            self.symbol,
        ))
        .await
        .unwrap();

        contract
    }
}
