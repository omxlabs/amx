use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    OlpManager,
    r#"[
        function init(address gov, address olp_manager_utils, address vault, address swap_manager, address positions_manager, address shorts_tracker, address usdo, address olp) external
        function isHandler(address account) external view returns (bool)
        function setGov(address gov) external
        function setInPrivateMode(bool in_private_mode) external
        function setHandler(address handler, bool is_active) external
        function addLiquidity(address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256)
        function addLiquidityForAccount(address funding_account, address account, address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256)
        function removeLiquidity(address token_out, uint256 olp_amount, uint256 min_amount, address recipient) external returns (uint256)
        function removeLiquidityForAccount(address account, address token_out, uint256 olp_amount, uint256 min_amount, address recipient) external returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct OlpManagerInitArgs {
    pub olp_manager_utils: Address,
    pub vault: Address,
    pub swap_manager: Address,
    pub positions_manager: Address,
    pub shorts_tracker: Address,
    pub usdo: Address,
    pub olp: Address,
}

impl OlpManagerInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> OlpManager<TestClient> {
        let contract = OlpManager::new(addr, gov.clone());

        contract
            .init(
                gov.address(),
                self.olp_manager_utils,
                self.vault,
                self.swap_manager,
                self.positions_manager,
                self.shorts_tracker,
                self.usdo,
                self.olp,
            )
            .await
            .unwrap();

        contract
    }
}
