use std::sync::Arc;

use ethers::types::{Address, U256};

use crate::{
    contracts::{erc20::Erc20, vault_price_feed::VaultPriceFeed, ContractAddresses},
    stylus_testing::provider::TestClient,
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
    pub vault_price_feed: VaultPriceFeed<TestClient>,

    pub orderbook: OrderbookContracts,
    pub router: RouterContracts,
    pub vault: VaultContracts,
    pub tokens: TokensContracts,
    pub staking: StakingContracts,
}

impl Contracts {
    /// Helper to set price for token on OMX
    pub async fn set_price(&self, token: Address, price: U256) {
        self.vault_price_feed.set_price(token, price).await.unwrap();
    }

    pub async fn balance(&self, token: Address, account: Address) -> U256 {
        let token = Erc20::new(token, self.vault.vault.client());

        token.balance_of(account).await.unwrap()
    }

    /// Mint tokens and deposit directly to the vault
    pub async fn deposit_to_vault(&self, token_addr: Address, amount: U256) {
        let token = Erc20::new(token_addr, self.vault.vault.client());
        token
            .mint(self.vault.vault.address(), amount)
            .await
            .unwrap();
        self.vault
            .vault
            .direct_pool_deposit(token_addr)
            .await
            .unwrap();
    }

    pub async fn validate_vault_balance(&self, token_addr: Address, offset: U256) {
        let pool_amount = self.vault.vault.pool_amount(token_addr).await.unwrap();

        let fee_reserve = self
            .vault
            .fee_manager
            .get_fee_reserve(token_addr)
            .await
            .unwrap();

        let balance = self.balance(token_addr, self.vault.vault.address()).await;

        assert!(balance > U256::zero());
        assert_eq!(pool_amount + fee_reserve + offset, balance);
    }
}

#[derive(Clone, Debug)]
pub struct ContractsInitArgs {
    pub gov: Address,
    pub min_profit_time: U256,
}

impl ContractsInitArgs {
    pub async fn init(self, client: Arc<TestClient>, contracts: &ContractAddresses) -> Contracts {
        let contracts = Contracts {
            vault_price_feed: init_vault_price_feed(client.clone(), contracts).await,
            orderbook: OrderbookInitArgs {
                gov: self.gov,
                swap_router: contracts.router.swap,
            }
            .init(client.clone(), contracts)
            .await,
            router: RouterContractsInitArgs {}
                .init(client.clone(), contracts)
                .await,
            tokens: TokensContractsInitArgs { gov: self.gov }
                .init(client.clone(), contracts)
                .await,
            vault: VaultContractsInitArgs {
                gov: self.gov,
                min_profit_time: self.min_profit_time,
            }
            .init(client.clone(), contracts)
            .await,
            staking: StakingContractsInitArgs {}
                .init(client.clone(), contracts)
                .await,
        };

        log::debug!("Update last distribution time...");
        contracts
            .staking
            .fee_olp_distributor
            .update_last_distribution_time()
            .await
            .unwrap();
        contracts
            .staking
            .fee_omx_distributor
            .update_last_distribution_time()
            .await
            .unwrap();
        contracts
            .staking
            .bonus_omx_distributor
            .update_last_distribution_time()
            .await
            .unwrap();
        contracts
            .staking
            .staked_olp_distributor
            .update_last_distribution_time()
            .await
            .unwrap();
        contracts
            .staking
            .staked_omx_distributor
            .update_last_distribution_time()
            .await
            .unwrap();

        log::debug!("Configure staking modes...");
        contracts
            .staking
            .olp_manager
            .set_in_private_mode(true)
            .await
            .unwrap();

        contracts
            .staking
            .staked_omx_tracker
            .set_in_private_transfer_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .staked_omx_tracker_staking
            .set_in_private_staking_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .bonus_omx_tracker
            .set_in_private_transfer_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .bonus_omx_tracker_staking
            .set_in_private_staking_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .bonus_omx_tracker_staking
            .set_in_private_claiming_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .fee_omx_tracker
            .set_in_private_transfer_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .fee_omx_tracker_staking
            .set_in_private_staking_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .fee_olp_tracker
            .set_in_private_transfer_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .fee_olp_tracker_staking
            .set_in_private_staking_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .staked_olp_tracker
            .set_in_private_transfer_mode(true)
            .await
            .unwrap();
        contracts
            .staking
            .staked_olp_tracker_staking
            .set_in_private_staking_mode(true)
            .await
            .unwrap();

        log::debug!("Configure staking handlers...");
        // allow reward_router to stake in staked_omx_tracker
        contracts
            .staking
            .staked_omx_tracker
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow bonus_omx_tracker to stake stakedomxTracker
        contracts
            .staking
            .staked_omx_tracker
            .set_handler(contracts.staking.bonus_omx_tracker_staking.address(), true)
            .await
            .unwrap();
        // allow reward_router to stake in bonus_omx_tracker
        contracts
            .staking
            .bonus_omx_tracker
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow bonus_omx_tracker to stake fee_omx_tracker
        contracts
            .staking
            .bonus_omx_tracker
            .set_handler(contracts.staking.fee_omx_tracker_staking.address(), true)
            .await
            .unwrap();
        contracts
            .staking
            .bonus_omx_distributor
            .set_bonus_multiplier(U256::from(10000))
            .await
            .unwrap();
        // allow reward_router to stake in fee_omx_tracker
        contracts
            .staking
            .fee_omx_tracker
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow fee_omx_tracker to stake bnomx
        contracts
            .tokens
            .bn_omx
            .set_handler(contracts.staking.fee_omx_tracker_staking.address(), true)
            .await
            .unwrap();

        contracts
            .tokens
            .bn_omx
            .set_minter(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();

        // allow reward_router to mint in olp_manager
        contracts
            .staking
            .olp_manager
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow reward_router to stake in fee_olp_tracker
        contracts
            .staking
            .fee_olp_tracker
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow staked_olp_tracker to stake fee_olp_tracker
        contracts
            .staking
            .fee_olp_tracker
            .set_handler(contracts.staking.staked_olp_tracker_staking.address(), true)
            .await
            .unwrap();
        // allow reward_router to sake in staked_olp_tracker
        contracts
            .staking
            .staked_olp_tracker
            .set_handler(contracts.staking.reward_router.address(), true)
            .await
            .unwrap();
        // allow fee_olp_tracker to stake olp
        contracts
            .staking
            .fee_olp_tracker
            .set_handler(contracts.tokens.olp.address(), true)
            .await
            .unwrap();
        contracts
            .tokens
            .olp
            .set_handler(contracts.staking.fee_olp_tracker.address(), true)
            .await
            .unwrap();

        contracts
    }
}
