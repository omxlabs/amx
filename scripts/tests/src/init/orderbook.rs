use std::sync::Arc;

use ethers::types::Address;

use crate::{
    contracts::{
        orderbook_increase::{OrderbookIncrease, OrderbookIncreaseInitArgs},
        orderbook_swap::{OrderbookSwap, OrderbookSwapInitArgs},
        ContractAddresses,
    },
    stylus_testing::provider::TestClient,
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
    pub swap: OrderbookSwap<TestClient>,
    pub increase: OrderbookIncrease<TestClient>,
}

impl OrderbookInitArgs {
    pub async fn init(
        self,
        client: Arc<TestClient>,
        contracts: &ContractAddresses,
    ) -> OrderbookContracts {
        OrderbookContracts {
            swap: OrderbookSwapInitArgs {
                gov: self.gov,
                swap_router: self.swap_router,
            }
            .init(client.clone(), contracts.orderbook.swap)
            .await,
            increase: OrderbookIncreaseInitArgs {
                gov: self.gov,
                swap_router: self.swap_router,
            }
            .init(client.clone(), contracts.orderbook.increase)
            .await,
        }
    }
}
