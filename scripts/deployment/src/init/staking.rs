use ethers::abi::Address;

use paste::paste;

use crate::contracts::{
    olp_manager::{OlpManager, OlpManagerInitArgs},
    olp_manager_utils::{OlpManagerUtils, OlpManagerUtilsInitArgs},
    bonus_distributor::{BonusDistributor, BonusDistributorInitArgs},
    reward_distributor::{RewardDistributor, RewardDistributorInitArgs},
    reward_router::{RewardRouter, RewardRouterInitArgs},
    reward_tracker::{RewardTracker, RewardTrackerInitArgs},
    reward_tracker_staking::{RewardTrackerStaking, RewardTrackerStakingInitArgs},
    shorts_tracker::{ShortsTracker, ShortsTrackerInitArgs},
    ContractAddresses, DeployContext, LiveClient,
};

/// Router contracts init helper
#[derive(Clone, Debug)]
pub struct StakingContractsInitArgs {
    pub gov: Address,
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct StakingContracts {
    pub reward_router: RewardRouter<LiveClient>,
    pub olp_manager: OlpManager<LiveClient>,
    pub olp_manager_utils: OlpManagerUtils<LiveClient>,
    pub shorts_tracker: ShortsTracker<LiveClient>,

    pub staked_omx_tracker: RewardTracker<LiveClient>,
    pub staked_omx_tracker_staking: RewardTrackerStaking<LiveClient>,
    pub staked_omx_distributor: RewardDistributor<LiveClient>,
    pub bonus_omx_tracker: RewardTracker<LiveClient>,
    pub bonus_omx_tracker_staking: RewardTrackerStaking<LiveClient>,
    pub bonus_omx_distributor: BonusDistributor<LiveClient>,
    pub fee_omx_tracker: RewardTracker<LiveClient>,
    pub fee_omx_tracker_staking: RewardTrackerStaking<LiveClient>,
    pub fee_omx_distributor: RewardDistributor<LiveClient>,
    pub fee_olp_tracker: RewardTracker<LiveClient>,
    pub fee_olp_tracker_staking: RewardTrackerStaking<LiveClient>,
    pub fee_olp_distributor: RewardDistributor<LiveClient>,
    pub staked_olp_tracker: RewardTracker<LiveClient>,
    pub staked_olp_tracker_staking: RewardTrackerStaking<LiveClient>,
    pub staked_olp_distributor: RewardDistributor<LiveClient>,
}

impl StakingContractsInitArgs {
    pub async fn init(
        self,
        ctx: &DeployContext,
        contracts: &ContractAddresses,
    ) -> StakingContracts {
        macro_rules! distributor {
            ($name:ident, $distributor_type:ident, $reward_token:ident, $deposit_tokens:expr, $tracker_name:expr, $tracker_symbol:expr) => {
                paste! {
                    (
                        [<$distributor_type DistributorInitArgs>] {
                            gov: self.gov,
                            reward_token: contracts.tokens.$reward_token,
                            reward_tracker: contracts.staking.[<$name _tracker>],
                            reward_tracker_staking: contracts.staking.[<$name _tracker_staking>],
                        }
                        .init(&ctx, contracts.staking.[<$name _distributor>])
                        .await,
                        RewardTrackerInitArgs {
                            gov: self.gov,
                            distributor: contracts.staking.[<$name _distributor>],
                            name: $tracker_name.to_string(),
                            reward_tracker_staking: contracts.staking.[<$name _tracker_staking>],
                            symbol: $tracker_symbol.to_string(),
                        }
                        .init(&ctx, contracts.staking.[<$name _tracker>])
                        .await,
                        RewardTrackerStakingInitArgs {
                            gov: self.gov,
                            deposit_tokens: $deposit_tokens,
                            reward_tracker: contracts.staking.[<$name _tracker>],
                            distributor: contracts.staking.[<$name _distributor>],
                        }
                        .init(&ctx, contracts.staking.[<$name _tracker_staking>])
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
                gov: self.gov,
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
            .init(&ctx, contracts.staking.reward_router)
            .await,
            olp_manager: OlpManagerInitArgs {
                gov: self.gov,
                olp: contracts.tokens.olp,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                olp_manager_utils: contracts.staking.olp_manager_utils,
                positions_manager: contracts.vault.positions_manager,
                shorts_tracker: contracts.staking.shorts_tracker,
                swap_manager: contracts.vault.swap_manager,
            }
            .init(&ctx, contracts.staking.olp_manager)
            .await,
            olp_manager_utils: OlpManagerUtilsInitArgs {
                gov: self.gov,
                olp: contracts.tokens.olp,
                positions_manager: contracts.vault.positions_manager,
                shorts_tracker: contracts.staking.shorts_tracker,
                vault: contracts.vault.vault,
            }
            .init(&ctx, contracts.staking.olp_manager_utils)
            .await,
            shorts_tracker: ShortsTrackerInitArgs {
                gov: self.gov,
                positions_manager: contracts.vault.positions_manager,
                vault: contracts.vault.vault,
                vault_utils: contracts.vault.vault_utils,
            }
            .init(&ctx, contracts.staking.shorts_tracker)
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
