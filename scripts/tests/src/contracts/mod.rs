use std::{fmt::Display, sync::Arc};

use ethers::{
    abi::Abi, middleware::SignerMiddleware, providers::Provider, signers::LocalWallet,
    types::Address,
};
use serde::Serialize;

use crate::{
    constants::ARTIFACTS_DIR,
    stylus_testing::provider::{TestClient, TestProvider},
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
    pub dai: Address,
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
    pub yield_tracker: Address,
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

    pub staking: StakingAddresses,
    pub router: RouterAddresses,
    pub vault: VaultAddresses,
    pub tokens: TokensAddresses,
    pub orderbook: OrderbookAddresses,
}

fn get_contract_path(contract: impl Display) -> String {
    format!("../../{}/omx_{}.wasm", ARTIFACTS_DIR, contract)
}

impl ContractAddresses {
    pub async fn deploy_contracts(client: Arc<TestClient>) -> Self {
        let deploy = |name: &str, abi: Abi, label: Option<&str>| -> Address {
            let path = get_contract_path(name);
            let bytes = std::fs::read(&path).expect(&format!("file should exist {}", path));

            let label = if let Some(label) = label {
                format!("{}__{}", name, label)
            } else {
                name.to_string()
            };
            client.deploy_contract(&bytes, abi, label)
        };

        Self {
            vault_price_feed: deploy(
                "vault_price_feed",
                vault_price_feed::VAULTPRICEFEED_ABI.clone(),
                None,
            ),

            staking: StakingAddresses {
                reward_router: deploy(
                    "reward_router",
                    reward_router::REWARDROUTER_ABI.clone(),
                    None,
                ),
                shorts_tracker: deploy(
                    "shorts_tracker",
                    shorts_tracker::SHORTSTRACKER_ABI.clone(),
                    None,
                ),
                olp_manager: deploy("olp_manager", olp_manager::OLPMANAGER_ABI.clone(), None),
                olp_manager_utils: deploy(
                    "olp_manager_utils",
                    olp_manager_utils::OLPMANAGERUTILS_ABI.clone(),
                    None,
                ),
                bonus_omx_distributor: deploy(
                    "bonus_distributor",
                    bonus_distributor::BONUSDISTRIBUTOR_ABI.clone(),
                    Some("bonus_omx_distributor"),
                ),
                bonus_omx_tracker: deploy(
                    "reward_tracker",
                    reward_tracker::REWARDTRACKER_ABI.clone(),
                    Some("bonus_omx_tracker"),
                ),
                bonus_omx_tracker_staking: deploy(
                    "reward_tracker_staking",
                    reward_tracker_staking::REWARDTRACKERSTAKING_ABI.clone(),
                    Some("bonus_omx_tracker_staking"),
                ),
                fee_olp_distributor: deploy(
                    "reward_distributor",
                    reward_distributor::REWARDDISTRIBUTOR_ABI.clone(),
                    Some("fee_olp_distributor"),
                ),
                fee_olp_tracker: deploy(
                    "reward_tracker",
                    reward_tracker::REWARDTRACKER_ABI.clone(),
                    Some("fee_olp_tracker"),
                ),
                fee_olp_tracker_staking: deploy(
                    "reward_tracker_staking",
                    reward_tracker_staking::REWARDTRACKERSTAKING_ABI.clone(),
                    Some("fee_olp_tracker_staking"),
                ),
                fee_omx_distributor: deploy(
                    "reward_distributor",
                    reward_distributor::REWARDDISTRIBUTOR_ABI.clone(),
                    Some("fee_omx_distributor"),
                ),
                fee_omx_tracker: deploy(
                    "reward_tracker",
                    reward_tracker::REWARDTRACKER_ABI.clone(),
                    Some("fee_omx_tracker"),
                ),
                fee_omx_tracker_staking: deploy(
                    "reward_tracker_staking",
                    reward_tracker_staking::REWARDTRACKERSTAKING_ABI.clone(),
                    Some("fee_omx_tracker_staking"),
                ),
                staked_olp_distributor: deploy(
                    "reward_distributor",
                    reward_distributor::REWARDDISTRIBUTOR_ABI.clone(),
                    Some("staked_olp_distributor"),
                ),
                staked_olp_tracker: deploy(
                    "reward_tracker",
                    reward_tracker::REWARDTRACKER_ABI.clone(),
                    Some("staked_olp_tracker"),
                ),
                staked_olp_tracker_staking: deploy(
                    "reward_tracker_staking",
                    reward_tracker_staking::REWARDTRACKERSTAKING_ABI.clone(),
                    Some("staked_olp_tracker_staking"),
                ),
                staked_omx_distributor: deploy(
                    "reward_distributor",
                    reward_distributor::REWARDDISTRIBUTOR_ABI.clone(),
                    Some("staked_omx_distributor"),
                ),
                staked_omx_tracker: deploy(
                    "reward_tracker",
                    reward_tracker::REWARDTRACKER_ABI.clone(),
                    Some("staked_omx_tracker"),
                ),
                staked_omx_tracker_staking: deploy(
                    "reward_tracker_staking",
                    reward_tracker_staking::REWARDTRACKERSTAKING_ABI.clone(),
                    Some("staked_omx_tracker_staking"),
                ),
            },

            vault: VaultAddresses {
                fee_manager: deploy("fee_manager", fee_manager::FEEMANAGER_ABI.clone(), None),
                funding_rate_manager: deploy(
                    "funding_rate_manager",
                    funding_rate_manager::FUNDINGRATEMANAGER_ABI.clone(),
                    None,
                ),
                positions_decrease_manager: deploy(
                    "positions_decrease_manager",
                    positions_decrease_manager::POSITIONSDECREASEMANAGER_ABI.clone(),
                    None,
                ),
                positions_increase_manager: deploy(
                    "positions_increase_manager",
                    positions_increase_manager::POSITIONSINCREASEMANAGER_ABI.clone(),
                    None,
                ),
                positions_liquidation_manager: deploy(
                    "positions_liquidation_manager",
                    positions_liquidation_manager::POSITIONSLIQUIDATIONMANAGER_ABI.clone(),
                    None,
                ),
                positions_manager: deploy(
                    "positions_manager",
                    positions_manager::POSITIONSMANAGER_ABI.clone(),
                    None,
                ),
                positions_manager_utils: deploy(
                    "positions_manager_utils",
                    positions_manager_utils::POSITIONSMANAGERUTILS_ABI.clone(),
                    None,
                ),
                swap_manager: deploy("swap_manager", swap_manager::SWAPMANAGER_ABI.clone(), None),
                vault: deploy("vault", vault::VAULT_ABI.clone(), None),
                vault_utils: deploy("vault_utils", vault_utils::VAULTUTILS_ABI.clone(), None),
            },

            router: RouterAddresses {
                swap: deploy("swap_router", swap_router::SWAPROUTER_ABI.clone(), None),
                positions_decrease: deploy(
                    "positions_decrease_router",
                    positions_decrease_router::POSITIONSDECREASEROUTER_ABI.clone(),
                    None,
                ),
                positions_increase: deploy(
                    "positions_increase_router",
                    positions_increase_router::POSITIONSINCREASEROUTER_ABI.clone(),
                    None,
                ),
            },

            tokens: TokensAddresses {
                usdo: deploy(
                    "yield_token",
                    yield_token::YIELDTOKEN_ABI.clone(),
                    Some("usdo"),
                ),
                olp: deploy("base_token", base_token::BASETOKEN_ABI.clone(), Some("olp")),
                omx: deploy("base_token", base_token::BASETOKEN_ABI.clone(), Some("omx")),
                es_omx: deploy(
                    "base_token",
                    base_token::BASETOKEN_ABI.clone(),
                    Some("es_omx"),
                ),
                bn_omx: deploy(
                    "base_token",
                    base_token::BASETOKEN_ABI.clone(),
                    Some("bn_omx"),
                ),
                weth: deploy("weth", weth::WETH_ABI.clone(), Some("weth")),
                dai: deploy("erc20", erc20::ERC20_ABI.clone(), Some("dai")),
                btc: deploy("erc20", erc20::ERC20_ABI.clone(), Some("btc")),
                bnb: deploy("erc20", erc20::ERC20_ABI.clone(), Some("bnb")),
                usdt: deploy("erc20", erc20::ERC20_ABI.clone(), Some("usdt")),
                usdc: deploy("erc20", erc20::ERC20_ABI.clone(), Some("usdc")),
                atom: deploy("erc20", erc20::ERC20_ABI.clone(), Some("atom")),
                osmo: deploy("erc20", erc20::ERC20_ABI.clone(), Some("osmo")),
                distributor: deploy(
                    "time_distributor",
                    distributor::DISTRIBUTOR_ABI.clone(),
                    None,
                ),
                yield_tracker: deploy(
                    "yield_tracker",
                    yield_tracker::YIELDTRACKER_ABI.clone(),
                    None,
                ),
            },

            orderbook: OrderbookAddresses {
                swap: deploy(
                    "orderbook_swap",
                    orderbook_swap::ORDERBOOKSWAP_ABI.clone(),
                    None,
                ),
                increase: deploy(
                    "orderbook_increase",
                    orderbook_increase::ORDERBOOKINCREASE_ABI.clone(),
                    None,
                ),
            },
        }
    }
}
