use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    Erc20,
    r#"[
        function init(address gov, string calldata name, string calldata symbol, uint8 decimals) external
        function setMinter(address minter, bool is_active) external
        function setGov(address gov) external
        function mint(address account, uint256 amount) external
        function burn(uint256 amount) external
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
pub struct Erc20InitArgs {
    pub name: String,
    pub symbol: String,
    pub gov: Address,
    pub decimals: u8,
}

impl Erc20InitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> Erc20<TestClient> {
        let contract = Erc20::new(addr, gov.clone());

        contract
            .init(self.gov, self.name, self.symbol, self.decimals)
            .await
            .unwrap();

        contract
    }
}
