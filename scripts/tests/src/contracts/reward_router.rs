use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    RewardRouter,
    r#"[
        event StakeOmx(address account, uint256 amount)
        event UnstakeOmx(address account, uint256 amount)
        event StakeOlp(address account, uint256 amount)
        event UnstakeOlp(address account, uint256 amount)
        error Forbidden()
        error AlreadyInitialized()
        error NotInitialized()
        error InvalidAmount()
        error InvalidValue()
        error InvalidOlpAmount()
        function init(address gov, address omx, address es_omx, address bn_omx, address olp, address staked_omx_tracker, address bonus_omx_tracker, address staked_omx_tracker_staking, address bonus_omx_tracker_staking, address fee_omx_tracker_staking, address fee_olp_tracker_staking, address staked_olp_tracker_staking, address olp_manager) external
        function setGov(address gov) external
        function batchStakeOmxForAccount(address[] memory accounts, uint256[] memory amounts) external
        function stakeOmxForAccount(address account, uint256 amount) external
        function stakeOmx(uint256 amount) external
        function stakeEsOmx(uint256 amount) external
        function unstakeOmx(uint256 amount) external
        function unstakeEsOmx(uint256 amount) external
        function mintAndStakeOlp(address token, uint256 amount, uint256 min_usdo, uint256 min_olp) external returns (uint256)
        function unstakeAndRedeemOlp(address token_out, uint256 olp_amount, uint256 min_out, address receiver) external returns (uint256)
        function claim() external
        function claimEsOmx() external
        function claimFees() external
        function compound() external
        function compoundForAccount(address account) external
        function batchCompoundForAccounts(address[] memory accounts) external
    ]"#
);

#[derive(Clone, Debug)]
pub struct RewardRouterInitArgs {
    pub omx: Address,
    pub es_omx: Address,
    pub bn_omx: Address,
    pub olp: Address,
    pub staked_omx_tracker: Address,
    pub bonus_omx_tracker: Address,
    pub staked_omx_tracker_staking: Address,
    pub bonus_omx_tracker_staking: Address,
    pub fee_omx_tracker_staking: Address,
    pub fee_olp_tracker_staking: Address,
    pub staked_olp_tracker_staking: Address,
    pub olp_manager: Address,
}

impl RewardRouterInitArgs {
    pub async fn init(self, gov: Arc<TestClient>, addr: Address) -> RewardRouter<TestClient> {
        let contract = RewardRouter::new(addr, gov.clone());

        contract
            .init(
                gov.address(),
                self.omx,
                self.es_omx,
                self.bn_omx,
                self.olp,
                self.staked_omx_tracker,
                self.bonus_omx_tracker,
                self.staked_omx_tracker_staking,
                self.bonus_omx_tracker_staking,
                self.fee_omx_tracker_staking,
                self.fee_olp_tracker_staking,
                self.staked_olp_tracker_staking,
                self.olp_manager,
            )
            .await
            .unwrap();

        contract
    }
}
