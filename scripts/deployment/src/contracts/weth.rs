use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    Weth,
    r#"[
        function init(string calldata name, string calldata symbol) external
        function deposit(address to) external payable
        function depositApprove(address to) external payable
        function withdraw(address to, uint256 amount) external
        function name() external view returns (string memory)
        function symbol() external view returns (string memory)
        function decimals() external view returns (uint8)
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        function increaseAllowance(address spender, uint256 added_value) external returns (bool)
        function decreaseAllowance(address spender, uint256 subtracted_value) external returns (bool)
    ]"#
);

#[derive(Clone, Debug)]
pub struct WethInitArgs {
    pub name: String,
    pub symbol: String,
}

impl WethInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> Weth<LiveClient> {
        let weth = Weth::new(addr, ctx.client.clone());

        send(weth.init(self.name, self.symbol)).await.unwrap();

        weth
    }
}
