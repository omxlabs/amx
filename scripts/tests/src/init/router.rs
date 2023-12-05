use std::sync::Arc;

use crate::{
    contracts::{
        positions_decrease_router::{PositionsDecreaseRouter, PositionsDecreaseRouterInitArgs},
        positions_increase_router::{PositionsIncreaseRouter, PositionsIncreaseRouterInitArgs},
        swap_router::{SwapRouter, SwapRouterInitArgs},
        ContractAddresses,
    },
    stylus_testing::provider::TestClient,
};

/// Router contracts init helper
#[derive(Clone, Debug)]
pub struct RouterContractsInitArgs {}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct RouterContracts {
    pub positions_decrease: PositionsDecreaseRouter<TestClient>,
    pub positions_increase: PositionsIncreaseRouter<TestClient>,
    pub swap: SwapRouter<TestClient>,
}

impl RouterContractsInitArgs {
    pub async fn init(
        self,
        client: Arc<TestClient>,
        contracts: &ContractAddresses,
    ) -> RouterContracts {
        RouterContracts {
            positions_decrease: PositionsDecreaseRouterInitArgs {
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                swap_router: contracts.vault.swap_manager,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(client.clone(), contracts.router.positions_decrease)
            .await,
            positions_increase: PositionsIncreaseRouterInitArgs {
                positions_increase_manager: contracts.vault.positions_increase_manager,
                swap_router: contracts.vault.swap_manager,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(client.clone(), contracts.router.positions_increase)
            .await,
            swap: SwapRouterInitArgs {
                positions_router: contracts.router.positions_increase,
                swap_manager: contracts.vault.swap_manager,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(client.clone(), contracts.router.swap)
            .await,
        }
    }
}
