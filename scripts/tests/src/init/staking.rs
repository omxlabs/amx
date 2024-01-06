use std::sync::Arc;

use paste::paste;

use crate::{
    contracts::{
        olp_manager::{OlpManager, OlpManagerInitArgs},
        olp_manager_utils::{OlpManagerUtils, OlpManagerUtilsInitArgs},
        bonus_distributor::{BonusDistributor, BonusDistributorInitArgs},
        reward_distributor::{RewardDistributor, RewardDistributorInitArgs},
        reward_router::{RewardRouter, RewardRouterInitArgs},
        reward_tracker::{RewardTracker, RewardTrackerInitArgs},
        reward_tracker_staking::{RewardTrackerStaking, RewardTrackerStakingInitArgs},
        shorts_tracker::{ShortsTracker, ShortsTrackerInitArgs},
        ContractAddresses,
    },
    stylus_testing::provider::TestClient,
};

/// Router contracts init helper
#[derive(Clone, Debug)]
pub struct StakingContractsInitArgs {}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct StakingContracts {
    pub reward_router: RewardRouter<TestClient>,
    pub olp_manager: OlpManager<TestClient>,
    pub olp_manager_utils: OlpManagerUtils<TestClient>,
    pub shorts_tracker: ShortsTracker<TestClient>,

    pub staked_omx_tracker: RewardTracker<TestClient>,
    pub staked_omx_tracker_staking: RewardTrackerStaking<TestClient>,
    pub staked_omx_distributor: RewardDistributor<TestClient>,
    pub bonus_omx_tracker: RewardTracker<TestClient>,
    pub bonus_omx_tracker_staking: RewardTrackerStaking<TestClient>,
    pub bonus_omx_distributor: BonusDistributor<TestClient>,
    pub fee_omx_tracker: RewardTracker<TestClient>,
    pub fee_omx_tracker_staking: RewardTrackerStaking<TestClient>,
    pub fee_omx_distributor: RewardDistributor<TestClient>,
    pub fee_olp_tracker: RewardTracker<TestClient>,
    pub fee_olp_tracker_staking: RewardTrackerStaking<TestClient>,
    pub fee_olp_distributor: RewardDistributor<TestClient>,
    pub staked_olp_tracker: RewardTracker<TestClient>,
    pub staked_olp_tracker_staking: RewardTrackerStaking<TestClient>,
    pub staked_olp_distributor: RewardDistributor<TestClient>,
}

impl StakingContractsInitArgs {
    pub async fn init(
        self,
        client: Arc<TestClient>,
        contracts: &ContractAddresses,
    ) -> StakingContracts {
        macro_rules! distributor {
            ($name:ident, $distributor_type:ident, $reward_token:ident, $deposit_tokens:expr, $tracker_name:expr, $tracker_symbol:expr) => {
                paste! {
                    (
                        [<$distributor_type DistributorInitArgs>] {
                            reward_token: contracts.tokens.$reward_token,
                            reward_tracker: contracts.staking.[<$name _tracker>],
                            reward_tracker_staking: contracts.staking.[<$name _tracker_staking>],
                        }
                        .init(client.clone(), contracts.staking.[<$name _distributor>])
                        .await,
                        RewardTrackerInitArgs {
                            distributor: contracts.staking.[<$name _distributor>],
                            name: $tracker_name.to_string(),
                            reward_tracker_staking: contracts.staking.[<$name _tracker_staking>],
                            symbol: $tracker_symbol.to_string(),
                        }
                        .init(client.clone(), contracts.staking.[<$name _tracker>])
                        .await,
                        RewardTrackerStakingInitArgs {
                            deposit_tokens: $deposit_tokens,
                            reward_tracker: contracts.staking.[<$name _tracker>],
                            distributor: contracts.staking.[<$name _distributor>],
                        }
                        .init(client.clone(), contracts.staking.[<$name _tracker_staking>])
                        .await
                    )
                }
            };
        }

        let (staked_omx_distributor, staked_omx_tracker, staked_omx_tracker_staking) = distributor!(
            staked_omx,
            Reward,
            es_omx,
            vec![contracts.tokens.omx, contracts.tokens.es_omx],
            "Staked OMX",
            "sOMX"
        );

        let (bonus_omx_distributor, bonus_omx_tracker, bonus_omx_tracker_staking) = distributor!(
            bonus_omx,
            Bonus,
            bn_omx,
            vec![contracts.staking.staked_omx_tracker],
            "Staked + Bonus OMX",
            "sbOMX"
        );

        let (fee_omx_distributor, fee_omx_tracker, fee_omx_tracker_staking) = distributor!(
            fee_omx,
            Reward,
            weth,
            vec![contracts.staking.bonus_omx_tracker, contracts.tokens.bn_omx],
            "Staked + Bonus + Fee OMX",
            "sbfOMX"
        );

        let (fee_olp_distributor, fee_olp_tracker, fee_olp_tracker_staking) = distributor!(
            fee_olp,
            Reward,
            weth,
            vec![contracts.tokens.olp],
            "Fee OLP",
            "fOLP"
        );

        let (staked_olp_distributor, staked_olp_tracker, staked_olp_tracker_staking) = distributor!(
            staked_olp,
            Reward,
            es_omx,
            vec![contracts.staking.fee_olp_tracker],
            "Fee + Staked OLP",
            "fsOLP"
        );

        StakingContracts {
            reward_router: RewardRouterInitArgs {
                olp: contracts.tokens.olp,
                omx: contracts.tokens.omx,
                es_omx: contracts.tokens.es_omx,
                bn_omx: contracts.tokens.bn_omx,
                olp_manager: contracts.staking.olp_manager,
                bonus_omx_tracker: contracts.staking.bonus_omx_tracker,
                staked_omx_tracker: contracts.staking.staked_omx_tracker,
                fee_olp_tracker_staking: contracts.staking.fee_olp_tracker_staking,
                bonus_omx_tracker_staking: contracts.staking.bonus_omx_tracker_staking,
                fee_omx_tracker_staking: contracts.staking.fee_omx_tracker_staking,
                staked_olp_tracker_staking: contracts.staking.staked_olp_tracker_staking,
                staked_omx_tracker_staking: contracts.staking.staked_omx_tracker_staking,
            }
            .init(client.clone(), contracts.staking.reward_router)
            .await,
            olp_manager: OlpManagerInitArgs {
                olp: contracts.tokens.olp,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                olp_manager_utils: contracts.staking.olp_manager_utils,
                positions_manager: contracts.vault.positions_manager,
                shorts_tracker: contracts.staking.shorts_tracker,
                swap_manager: contracts.vault.swap_manager,
            }
            .init(client.clone(), contracts.staking.olp_manager)
            .await,
            olp_manager_utils: OlpManagerUtilsInitArgs {
                olp: contracts.tokens.olp,
                positions_manager: contracts.vault.positions_manager,
                shorts_tracker: contracts.staking.shorts_tracker,
                vault: contracts.vault.vault,
            }
            .init(client.clone(), contracts.staking.olp_manager_utils)
            .await,
            shorts_tracker: ShortsTrackerInitArgs {
                positions_manager: contracts.vault.positions_manager,
                vault: contracts.vault.vault,
                vault_utils: contracts.vault.vault_utils,
            }
            .init(client.clone(), contracts.staking.shorts_tracker)
            .await,

            staked_omx_distributor,
            staked_omx_tracker,
            staked_omx_tracker_staking,
            bonus_omx_distributor,
            bonus_omx_tracker,
            bonus_omx_tracker_staking,
            fee_olp_distributor,
            fee_olp_tracker,
            fee_olp_tracker_staking,
            fee_omx_distributor,
            fee_omx_tracker,
            fee_omx_tracker_staking,
            staked_olp_tracker,
            staked_olp_distributor,
            staked_olp_tracker_staking,
        }
    }
}
