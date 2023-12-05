use crate::contracts::{
    positions_decrease_router::{PositionsDecreaseRouter, PositionsDecreaseRouterInitArgs},
    positions_increase_router::{PositionsIncreaseRouter, PositionsIncreaseRouterInitArgs},
    swap_router::{SwapRouter, SwapRouterInitArgs},
    ContractAddresses, DeployContext, LiveClient,
};

/// Router contracts init helper
#[derive(Clone, Debug)]
pub struct RouterContractsInitArgs {}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct RouterContracts {
    pub positions_decrease: PositionsDecreaseRouter<LiveClient>,
    pub positions_increase: PositionsIncreaseRouter<LiveClient>,
    pub swap: SwapRouter<LiveClient>,
}

impl RouterContractsInitArgs {
    /// Initialize all vault contracts
    pub async fn init(self, ctx: &DeployContext, contracts: &ContractAddresses) -> RouterContracts {
        println!("initializing router contracts");
        RouterContracts {
            positions_decrease: PositionsDecreaseRouterInitArgs {
                positions_decrease_manager: contracts.vault.positions_decrease_manager,
                swap_router: contracts.vault.swap_manager,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(&ctx, contracts.router.positions_decrease)
            .await,
            positions_increase: PositionsIncreaseRouterInitArgs {
                positions_increase_manager: contracts.vault.positions_increase_manager,
                swap_router: contracts.vault.swap_manager,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(&ctx, contracts.router.positions_increase)
            .await,
            swap: SwapRouterInitArgs {
                positions_router: contracts.router.positions_increase,
                swap_manager: contracts.vault.swap_manager,
                usdo: contracts.tokens.usdo,
                vault: contracts.vault.vault,
                weth: contracts.tokens.weth,
            }
            .init(&ctx, contracts.router.swap)
            .await,
        }
    }
}
