use ethers::{prelude::abigen, types::Address};

use crate::utils::contract_call_helper::send;

use super::{DeployContext, LiveClient};

abigen!(
    Vault,
    r#"[
        function init(address gov, address swap_manager, address positions_manager, address positions_increase_manager, address positions_decrease_manager, address positions_liquidation_manager, address positions_manager_utils, address price_feed) external
        function isStable(address token) external view returns (bool)
        function isShortable(address token) external view returns (bool)
        function isWhitelisted(address token) external view returns (bool)
        function getTokenDecimals(address token) external view returns (uint8)
        function tokenWeight(address token) external view returns (uint256)
        function totalTokenWeights() external view returns (uint256)
        function reservedAmount(address token) external view returns (uint256)
        function minProfitBasisPoint(address token) external view returns (uint256)
        function allWhitelistedTokensLength() external view returns (uint256)
        function poolAmount(address token) external view returns (uint256)
        function transferIn(address token) external returns (uint256)
        function transferOut(address token, uint256 amount, address receiver) external
        function decreasePoolAmount(address token, uint256 amount) external
        function increasePoolAmount(address token, uint256 amount) external
        function updateTokenBalance(address token) external
        function getPrice(address token) external view returns (uint256)
        function setTokenConfig(address token, uint8 token_decimals, uint256 token_weight, uint256 min_profit_basis_points, bool is_stable, bool is_shortable) external
        function clearTokenConfig(address token) external
        function directPoolDeposit(address token) external
        function increaseReservedAmount(address token, uint256 amount) external
        function decreaseReservedAmount(address token, uint256 amount) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct VaultInitArgs {
    pub gov: Address,
    pub swap_manager: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
    pub positions_increase_manager: Address,
    pub positions_decrease_manager: Address,
    pub positions_liquidation_manager: Address,
    pub price_feed: Address,
}

impl VaultInitArgs {
    pub async fn init(self, ctx: &DeployContext, addr: Address) -> Vault<LiveClient> {
        let vault = Vault::new(addr, ctx.client.clone());

        send(vault.init(
            self.gov,
            self.swap_manager,
            self.positions_manager,
            self.positions_increase_manager,
            self.positions_decrease_manager,
            self.positions_liquidation_manager,
            self.positions_manager_utils,
            self.price_feed,
        ))
        .await
        .unwrap();

        vault
    }
}
