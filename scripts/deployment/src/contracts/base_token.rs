use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    BaseToken,
    r#"[
        function init(address gov, string calldata name, string calldata symbol) external
        function setGov(address gov) external
        function setMinter(address minter, bool is_active) external
        function mint(address account, uint256 amount) external
        function burn(address account, uint256 amount) external
        function setInfo(string calldata name, string calldata symbol) external
        function setYieldTrackers(address[] memory yield_trackers) external
        function addAdmin(address account) external
        function removeAdmin(address account) external
        function setInPrivateTransferMode(bool in_private_transfer_mode) external
        function setHandler(address handler, bool is_active) external
        function addNonStakingAccount(address account) external
        function removeNonStakingAccount(address account) external
        function recoverClaim(address account, address receiver) external
        function claim(address receiver) external
        function totalSupply() external view returns (uint256)
        function totalStaked() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function stakedBalance(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
    ]"#
);

#[derive(Clone, Debug)]
pub struct BaseTokenInitArgs {
    pub gov: Address,
    pub name: String,
    pub symbol: String,
}

impl BaseTokenInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> BaseToken<LiveClient> {
        let base_token = BaseToken::new(addr, ctx.client.clone());

        send(base_token.init(self.gov, self.name, self.symbol))
            .await
            .unwrap();

        base_token
    }
}
