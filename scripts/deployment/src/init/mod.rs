use ethers::types::{Address, U256};

use crate::{
    contracts::{
        erc20::Erc20, vault_price_feed::VaultPriceFeed, ContractAddresses, DeployContext,
        LiveClient,
    },
    utils::contract_call_helper::send,
};

use self::{
    orderbook::{OrderbookContracts, OrderbookInitArgs},
    price_feed::init_vault_price_feed,
    router::{RouterContracts, RouterContractsInitArgs},
    staking::{StakingContracts, StakingContractsInitArgs},
    tokens::{TokensContracts, TokensContractsInitArgs},
    vault::{VaultContracts, VaultContractsInitArgs},
};

pub mod orderbook;
pub mod price_feed;
pub mod router;
pub mod staking;
pub mod tokens;
pub mod vault;

#[derive(Clone, Debug)]
pub struct Contracts {
    pub vault_price_feed: VaultPriceFeed<LiveClient>,

    pub orderbook: OrderbookContracts,
    pub router: RouterContracts,
    pub vault: VaultContracts,
    pub tokens: TokensContracts,
    pub staking: StakingContracts,
}

impl Contracts {
    /// Helper to set price for token on OMX
    pub async fn set_price(&self, token: Address, price: U256) {
        send(self.vault_price_feed.set_price(token, price))
            .await
            .unwrap();
    }

    /// Mint tokens and deposit directly to the vault
    pub async fn deposit_to_vault(&self, token_addr: Address, amount: U256) {
        let token = Erc20::new(token_addr, self.vault.vault.client());
        send(token.mint(self.vault.vault.address(), amount))
            .await
            .unwrap();
        send(self.vault.vault.direct_pool_deposit(token_addr))
            .await
            .unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct ContractsInitArgs {
    pub gov: Address,
    pub min_profit_time: U256,
}

impl ContractsInitArgs {
    pub async fn init(self, ctx: &DeployContext, contracts: &ContractAddresses) -> Contracts {
        let contracts = Contracts {
            vault_price_feed: init_vault_price_feed(ctx, contracts, self.gov).await,
            orderbook: OrderbookInitArgs {
                gov: self.gov,
                swap_router: contracts.router.swap,
            }
            .init(ctx, contracts)
            .await,
            router: RouterContractsInitArgs {}.init(ctx, contracts).await,
            tokens: TokensContractsInitArgs { gov: self.gov }
                .init(ctx, contracts)
                .await,
            vault: VaultContractsInitArgs {
                gov: self.gov,
                min_profit_time: self.min_profit_time,
            }
            .init(ctx, contracts)
            .await,
            staking: StakingContractsInitArgs { gov: self.gov }
                .init(ctx, contracts)
                .await,
        };

        println!("Update last distribution time...");
        send(
            contracts
                .staking
                .fee_olp_distributor
                .update_last_distribution_time(),
        )
        .await
        .unwrap();
        send(
            contracts
                .staking
                .fee_omx_distributor
                .update_last_distribution_time(),
        )
        .await
        .unwrap();
        send(
            contracts
                .staking
                .bonus_omx_distributor
                .update_last_distribution_time(),
        )
        .await
        .unwrap();
        send(
            contracts
                .staking
                .staked_olp_distributor
                .update_last_distribution_time(),
        )
        .await
        .unwrap();
        send(
            contracts
                .staking
                .staked_omx_distributor
                .update_last_distribution_time(),
        )
        .await
        .unwrap();

        println!("Configure staking handlers...");
        // allow reward_router to stake in staked_omx_tracker
        send(
            contracts
                .staking
                .staked_omx_tracker
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow bonus_omx_tracker to stake stakedomxTracker
        send(
            contracts
                .staking
                .staked_omx_tracker
                .set_handler(contracts.staking.bonus_omx_tracker_staking.address(), true),
        )
        .await
        .unwrap();
        // allow reward_router to stake in bonus_omx_tracker
        send(
            contracts
                .staking
                .bonus_omx_tracker
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow bonus_omx_tracker to stake fee_omx_tracker
        send(
            contracts
                .staking
                .bonus_omx_tracker
                .set_handler(contracts.staking.fee_omx_tracker_staking.address(), true),
        )
        .await
        .unwrap();
        send(
            contracts
                .staking
                .bonus_omx_distributor
                .set_bonus_multiplier(U256::from(10000)),
        )
        .await
        .unwrap();
        // allow reward_router to stake in fee_omx_tracker
        send(
            contracts
                .staking
                .fee_omx_tracker
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow fee_omx_tracker to stake bnomx
        send(
            contracts
                .tokens
                .bn_omx
                .set_handler(contracts.staking.fee_omx_tracker_staking.address(), true),
        )
        .await
        .unwrap();

        send(
            contracts
                .tokens
                .bn_omx
                .set_minter(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();

        // allow reward_router to mint in olp_manager
        send(
            contracts
                .staking
                .olp_manager
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow reward_router to stake in fee_olp_tracker
        send(
            contracts
                .staking
                .fee_olp_tracker
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow staked_olp_tracker to stake fee_olp_tracker
        send(
            contracts
                .staking
                .fee_olp_tracker
                .set_handler(contracts.staking.staked_olp_tracker_staking.address(), true),
        )
        .await
        .unwrap();
        // allow reward_router to sake in staked_olp_tracker
        send(
            contracts
                .staking
                .staked_olp_tracker
                .set_handler(contracts.staking.reward_router.address(), true),
        )
        .await
        .unwrap();
        // allow fee_olp_tracker to stake olp
        send(
            contracts
                .staking
                .fee_olp_tracker
                .set_handler(contracts.tokens.olp.address(), true),
        )
        .await
        .unwrap();

        send(
            contracts
                .tokens
                .olp
                .set_handler(contracts.staking.fee_olp_tracker.address(), true),
        )
        .await
        .unwrap();

        contracts
    }
}
