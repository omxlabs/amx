use std::sync::Arc;

use ethers::types::{Address, U256};

use crate::{
    constants::{
        ATOM_DECIMALS, BNB_DECIMALS, BTC_DECIMALS, DAI_DECIMALS, ETH_DECIMALS, OSMO_DECIMALS,
        USDC_DECIMALS, USDT_DECIMALS,
    },
    contracts::{
        fee_manager::{FeeManager, FeeManagerInitArgs},
        funding_rate_manager::{FundingRateManager, FundingRateManagerInitArgs},
        positions_decrease_manager::{PositionsDecreaseManager, PositionsDecreaseManagerInitArgs},
        positions_increase_manager::{PositionsIncreaseManager, PositionsIncreaseManagerInitArgs},
        positions_liquidation_manager::{
            PositionsLiquidationManager, PositionsLiquidationManagerInitArgs,
        },
        positions_manager::{PositionsManager, PositionsManagerInitArgs},
        positions_manager_utils::{PositionsManagerUtils, PositionsManagerUtilsInitArgs},
        swap_manager::{SwapManager, SwapManagerInitArgs},
        vault::{Vault, VaultInitArgs},
        vault_utils::{VaultUtils, VaultUtilsInitArgs},
        ContractAddresses,
    },
    stylus_testing::provider::TestClient,
};

/// Vault contracts init helper
#[derive(Clone, Debug)]
pub struct VaultContractsInitArgs {
    pub min_profit_time: U256,
    pub gov: Address,
}

impl VaultContracts {
    pub async fn set_token_config(
        &self,
        token: Address,
        token_decimals: u8,
        token_weight: U256,
        min_profit_basis_points: U256,
        is_stable: bool,
        is_shortable: bool,
    ) {
        self.vault
            .set_token_config(
                token,
                token_decimals,
                token_weight,
                min_profit_basis_points,
                is_stable,
                is_shortable,
            )
            .await
            .unwrap();
    }

    pub async fn set_bnb_config(&self, token: Address) {
        self.set_token_config(
            token,
            BNB_DECIMALS,
            U256::from(10000),
            U256::from(75),
            false,
            true,
        )
        .await;
    }

    pub async fn set_dai_config(&self, token: Address) {
        self.set_token_config(
            token,
            DAI_DECIMALS,
            U256::from(10000),
            U256::from(75),
            true,
            false,
        )
        .await;
    }

    pub async fn set_eth_config(&self, token: Address) {
        self.set_token_config(
            token,
            ETH_DECIMALS,
            U256::from(10000),
            U256::from(75),
            true,
            false,
        )
        .await;
    }

    pub async fn set_btc_config(&self, token: Address) {
        self.set_token_config(
            token,
            BTC_DECIMALS,
            U256::from(10000),
            U256::from(75),
            false,
            true,
        )
        .await;
    }

    pub async fn set_atom_config(&self, token: Address) {
        self.set_token_config(
            token,
            ATOM_DECIMALS,
            U256::from(10000),
            U256::from(75),
            false,
            true,
        )
        .await;
    }

    pub async fn set_osmo_config(&self, token: Address) {
        self.set_token_config(
            token,
            OSMO_DECIMALS,
            U256::from(10000),
            U256::from(75),
            false,
            true,
        )
        .await;
    }

    pub async fn set_usdt_config(&self, token: Address) {
        self.set_token_config(
            token,
            USDT_DECIMALS,
            U256::from(10000),
            U256::from(75),
            true,
            false,
        )
        .await;
    }

    pub async fn set_usdc_config(&self, token: Address) {
        self.set_token_config(
            token,
            USDC_DECIMALS,
            U256::from(10000),
            U256::from(75),
            true,
            false,
        )
        .await;
    }
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct VaultContracts {
    pub vault: Vault<TestClient>,
    pub funding_rate_manager: FundingRateManager<TestClient>,
    pub swap_manager: SwapManager<TestClient>,
    pub fee_manager: FeeManager<TestClient>,
    pub positions_manager: PositionsManager<TestClient>,
    pub positions_manager_utils: PositionsManagerUtils<TestClient>,
    pub positions_decrease_manager: PositionsDecreaseManager<TestClient>,
    pub positions_increase_manager: PositionsIncreaseManager<TestClient>,
    pub positions_liquidation_manager: PositionsLiquidationManager<TestClient>,
    pub vault_utils: VaultUtils<TestClient>,
}

impl VaultContractsInitArgs {
    pub async fn init(
        self,
        client: Arc<TestClient>,
        contracts: &ContractAddresses,
    ) -> VaultContracts {
        VaultContracts {
            vault: VaultInitArgs {
                gov: self.gov,
                swap_manager: contracts.vault.swap_manager,
                positions_manager: contracts.vault.positions_manager,
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                price_feed: contracts.vault_price_feed,
                positions_increase_manager: contracts.vault.positions_increase_manager,
                positions_liquidation_manager: contracts.vault.positions_liquidation_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
            }
            .init(client.clone(), contracts.vault.vault)
            .await,

            vault_utils: VaultUtilsInitArgs {
                min_profit_time: self.min_profit_time,
                vault: contracts.vault.vault,
            }
            .init(client.clone(), contracts.vault.vault_utils)
            .await,

            fee_manager: FeeManagerInitArgs {
                gov: self.gov,
                funding_rate_manager: contracts.vault.funding_rate_manager,
                positions_manager: contracts.vault.positions_manager,
                swap_manager: contracts.vault.swap_manager,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                positions_increase_manager: contracts.vault.positions_increase_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                positions_liquidation_manager: contracts.vault.positions_liquidation_manager,
            }
            .init(client.clone(), contracts.vault.fee_manager)
            .await,

            funding_rate_manager: FundingRateManagerInitArgs {
                gov: self.gov,
                vault: contracts.vault.vault,
            }
            .init(client.clone(), contracts.vault.funding_rate_manager)
            .await,

            positions_decrease_manager: PositionsDecreaseManagerInitArgs {
                gov: self.gov,
                vault: contracts.vault.vault,
                funding_rate_manager: contracts.vault.funding_rate_manager,
                decrease_router: contracts.router.positions_decrease,
                positions_manager: contracts.vault.positions_manager,
                positions_liquidation_manager: contracts.vault.positions_liquidation_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
            }
            .init(client.clone(), contracts.vault.positions_decrease_manager)
            .await,

            positions_increase_manager: PositionsIncreaseManagerInitArgs {
                gov: self.gov,
                vault: contracts.vault.vault,
                vault_utils: contracts.vault.vault_utils,
                fee_manager: contracts.vault.fee_manager,
                funding_rate_manager: contracts.vault.funding_rate_manager,
                increase_router: contracts.router.positions_increase,
                positions_manager: contracts.vault.positions_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
            }
            .init(client.clone(), contracts.vault.positions_increase_manager)
            .await,

            positions_manager: PositionsManagerInitArgs {
                gov: self.gov,
                vault_utils: contracts.vault.vault_utils,
                fee_manager: contracts.vault.fee_manager,
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                positions_increase_manager: contracts.vault.positions_increase_manager,
                positions_liquidation_manager: contracts.vault.positions_liquidation_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
            }
            .init(client.clone(), contracts.vault.positions_manager)
            .await,

            positions_manager_utils: PositionsManagerUtilsInitArgs {
                positions_manager: contracts.vault.positions_manager,
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                vault: contracts.vault.vault,
                fee_manager: contracts.vault.fee_manager,
                vault_utils: contracts.vault.vault_utils,
            }
            .init(client.clone(), contracts.vault.positions_manager_utils)
            .await,

            positions_liquidation_manager: PositionsLiquidationManagerInitArgs {
                gov: self.gov,
                fee_manager: contracts.vault.fee_manager,
                funding_rate_manager: contracts.vault.funding_rate_manager,
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                positions_manager: contracts.vault.positions_manager,
                positions_manager_utils: contracts.vault.positions_manager_utils,
                vault: contracts.vault.vault,
                vault_utils: contracts.vault.vault_utils,
            }
            .init(
                client.clone(),
                contracts.vault.positions_liquidation_manager,
            )
            .await,

            swap_manager: SwapManagerInitArgs {
                gov: self.gov,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                fee_manager: contracts.vault.fee_manager,
                funding_rate_manager: contracts.vault.funding_rate_manager,
            }
            .init(client.clone(), contracts.vault.swap_manager)
            .await,
        }
    }
}
