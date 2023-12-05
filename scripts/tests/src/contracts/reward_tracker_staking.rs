use std::sync::Arc;

use ethers::{prelude::abigen, types::Address};

use crate::stylus_testing::provider::TestClient;

abigen!(
    RewardTrackerStaking,
    r#"[
        function init(address gov, address reward_tracker, address distributor, address[] memory deposit_tokens) external
        function setGov(address gov) external
        function setDepositToken(address deposit_token, bool is_deposit_token) external
        function depositBalance(address account, address deposit_token) external view returns (uint256)
        function stake(address deposit_token, uint256 amount) external
        function setInPrivateClaimingMode(bool in_private_claiming_mode) external
        function setInPrivateStakingMode(bool in_private_staking_mode) external
        function stakeForAccount(address funding_account, address account, address deposit_token, uint256 amount) external
        function unstake(address deposit_token, uint256 amount) external
        function unstakeForAccount(address account, address deposit_token, uint256 amount, address receiver) external
        function updateRewards() external
        function claim(address receiver) external returns (uint256)
        function claimForAccount(address account, address receiver) external returns (uint256)
        function claimable(address account) external view returns (uint256)
        function stakedAmount(address account) external view returns (uint256)
        function cumulativeReward(address account) external view returns (uint256)
    ]"#
);

#[derive(Clone, Debug)]
pub struct RewardTrackerStakingInitArgs {
    pub reward_tracker: Address,
    pub distributor: Address,
    pub deposit_tokens: Vec<Address>,
}

impl RewardTrackerStakingInitArgs {
    pub async fn init(
        self,
        gov: Arc<TestClient>,
        addr: Address,
    ) -> RewardTrackerStaking<TestClient> {
        let contract = RewardTrackerStaking::new(addr, gov.clone());

        contract
            .init(
                gov.address(),
                self.reward_tracker,
                self.distributor,
                self.deposit_tokens,
            )
            .await
            .unwrap();

        contract
    }
}
