use std::{fmt::Display, sync::Arc};

use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::Address,
};
use serde::Serialize;

use crate::{
    constants::ARTIFACTS_DIR, deploy::deploy_contract, utils::client::client_from_str_key,
};

pub mod base_token;
pub mod bonus_distributor;
pub mod distributor;
pub mod erc20;
pub mod fee_manager;
pub mod funding_rate_manager;
pub mod olp_manager;
pub mod olp_manager_utils;
pub mod orderbook_increase;
pub mod orderbook_swap;
pub mod positions_decrease_manager;
pub mod positions_decrease_router;
pub mod positions_increase_manager;
pub mod positions_increase_router;
pub mod positions_liquidation_manager;
pub mod positions_manager;
pub mod positions_manager_utils;
pub mod reward_distributor;
pub mod reward_router;
pub mod reward_tracker;
pub mod reward_tracker_staking;
pub mod shorts_tracker;
pub mod swap_manager;
pub mod swap_router;
pub mod vault;
pub mod vault_price_feed;
pub mod vault_utils;
pub mod weth;
pub mod yield_token;
pub mod yield_tracker;

pub type Client<P> = SignerMiddleware<Provider<P>, LocalWallet>;

pub type LiveClient = SignerMiddleware<Provider<Http>, LocalWallet>;

#[derive(Clone, Debug)]
pub struct DeployContext {
    endpoint: String,
    client: Arc<LiveClient>,
    key_path: String,
}

impl DeployContext {
    pub async fn new(endpoint: impl AsRef<str>, key_path: impl AsRef<str>) -> Self {
        Self {
            endpoint: endpoint.as_ref().to_string(),
            key_path: key_path.as_ref().to_string(),
            client: client_from_str_key(key_path, endpoint).await,
        }
    }

    pub fn client(&self) -> Arc<LiveClient> {
        self.client.clone()
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub fn key_path(&self) -> String {
        self.key_path.clone()
    }
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct OrderbookAddresses {
    pub swap: Address,
    pub increase: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct StakingAddresses {
    pub shorts_tracker: Address,
    pub reward_router: Address,
    pub olp_manager: Address,
    pub olp_manager_utils: Address,

    pub staked_omx_tracker: Address,
    pub staked_omx_tracker_staking: Address,
    pub staked_omx_distributor: Address,
    pub bonus_omx_tracker: Address,
    pub bonus_omx_tracker_staking: Address,
    pub bonus_omx_distributor: Address,
    pub fee_omx_tracker: Address,
    pub fee_omx_tracker_staking: Address,
    pub fee_omx_distributor: Address,
    pub fee_olp_tracker: Address,
    pub fee_olp_tracker_staking: Address,
    pub fee_olp_distributor: Address,
    pub staked_olp_tracker: Address,
    pub staked_olp_tracker_staking: Address,
    pub staked_olp_distributor: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct TokensAddresses {
    pub weth: Address,
    pub btc: Address,
    pub atom: Address,
    pub osmo: Address,
    pub bnb: Address,
    pub usdt: Address,
    pub usdc: Address,
    pub usdo: Address,
    pub es_omx: Address,
    pub bn_omx: Address,
    pub olp: Address,
    pub omx: Address,
    pub distributor: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct VaultAddresses {
    pub fee_manager: Address,
    pub funding_rate_manager: Address,
    pub positions_manager: Address,
    pub positions_manager_utils: Address,
    pub positions_decrease_manager: Address,
    pub positions_increase_manager: Address,
    pub positions_liquidation_manager: Address,
    pub swap_manager: Address,
    pub vault: Address,
    pub vault_utils: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct RouterAddresses {
    pub positions_decrease: Address,
    pub positions_increase: Address,
    pub swap: Address,
}

#[derive(Clone, Debug, Copy, Serialize)]
pub struct ContractAddresses {
    pub vault_price_feed: Address,

    pub router: RouterAddresses,
    pub vault: VaultAddresses,
    pub tokens: TokensAddresses,
    pub orderbook: OrderbookAddresses,
    pub staking: StakingAddresses,
}

fn get_contract_path(contract: impl Display) -> String {
    format!("{}/omx_{}.wasm", ARTIFACTS_DIR, contract)
}

impl ContractAddresses {
    /// Deploy all contracts
    ///
    /// **NOTE**: This function only deploys contracts, you may also need to call `init` functions
    pub async fn deploy_contracts(ctx: &DeployContext) -> Self {
        let deploy = |name: &str| -> Address {
            deploy_contract(get_contract_path(name), &ctx.key_path, &ctx.endpoint)
        };

        Self {
            vault_price_feed: deploy("vault_price_feed"),

            staking: StakingAddresses {
                reward_router: deploy("reward_router"),
                shorts_tracker: deploy("shorts_tracker"),
                olp_manager: deploy("olp_manager"),
                olp_manager_utils: deploy("olp_manager_utils"),
                bonus_omx_distributor: deploy("bonus_distributor"),
                bonus_omx_tracker: deploy("reward_tracker"),
                bonus_omx_tracker_staking: deploy("reward_tracker_staking"),
                fee_olp_distributor: deploy("reward_distributor"),
                fee_olp_tracker: deploy("reward_tracker"),
                fee_olp_tracker_staking: deploy("reward_tracker_staking"),
                fee_omx_distributor: deploy("reward_distributor"),
                fee_omx_tracker: deploy("reward_tracker"),
                fee_omx_tracker_staking: deploy("reward_tracker_staking"),
                staked_olp_distributor: deploy("reward_distributor"),
                staked_olp_tracker: deploy("reward_tracker"),
                staked_olp_tracker_staking: deploy("reward_tracker_staking"),
                staked_omx_distributor: deploy("reward_distributor"),
                staked_omx_tracker: deploy("reward_tracker"),
                staked_omx_tracker_staking: deploy("reward_tracker_staking"),
            },

            vault: VaultAddresses {
                fee_manager: deploy("fee_manager"),
                funding_rate_manager: deploy("funding_rate_manager"),
                positions_decrease_manager: deploy("positions_decrease_manager"),
                positions_increase_manager: deploy("positions_increase_manager"),
                positions_liquidation_manager: deploy("positions_liquidation_manager"),
                positions_manager: deploy("positions_manager"),
                positions_manager_utils: deploy("positions_manager_utils"),
                swap_manager: deploy("swap_manager"),
                vault: deploy("vault"),
                vault_utils: deploy("vault_utils"),
            },

            router: RouterAddresses {
                swap: deploy("swap_router"),
                positions_decrease: deploy("positions_decrease_router"),
                positions_increase: deploy("positions_increase_router"),
            },

            tokens: TokensAddresses {
                usdo: deploy("yield_token"),
                weth: deploy("weth"),
                btc: deploy("erc20"),
                bnb: deploy("erc20"),
                usdt: deploy("erc20"),
                usdc: deploy("erc20"),
                atom: deploy("erc20"),
                osmo: deploy("erc20"),
                olp: deploy("base_token"),
                omx: deploy("base_token"),
                bn_omx: deploy("base_token"),
                es_omx: deploy("base_token"),
                distributor: deploy("time_distributor"),
            },

            orderbook: OrderbookAddresses {
                swap: deploy("orderbook_swap"),
                increase: deploy("orderbook_increase"),
            },
        }
    }
}
