use ethers::types::Address;

use crate::contracts::{
    orderbook_increase::{OrderbookIncrease, OrderbookIncreaseInitArgs},
    orderbook_swap::{OrderbookSwap, OrderbookSwapInitArgs},
    ContractAddresses, DeployContext, LiveClient,
};

/// Router contracts init helper
#[derive(Clone, Debug)]
pub struct OrderbookInitArgs {
    pub swap_router: Address,
    pub gov: Address,
}

/// All vault contracts
#[derive(Clone, Debug)]
pub struct OrderbookContracts {
    pub swap: OrderbookSwap<LiveClient>,
    pub increase: OrderbookIncrease<LiveClient>,
}

impl OrderbookInitArgs {
    /// Initialize all vault contracts
    pub async fn init(
        self,
        ctx: &DeployContext,
        contracts: &ContractAddresses,
    ) -> OrderbookContracts {
        println!("initializing orderbook contracts");
        OrderbookContracts {
            swap: OrderbookSwapInitArgs {
                gov: self.gov,
                swap_router: self.swap_router,
            }
            .init(ctx, contracts.orderbook.swap)
            .await,
            increase: OrderbookIncreaseInitArgs {
                gov: self.gov,
                swap_router: self.swap_router,
            }
            .init(ctx, contracts.orderbook.increase)
            .await,
        }
    }
}
