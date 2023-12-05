use std::sync::Arc;

use ethers::types::U256;

use crate::{
    contracts::{
        vault_price_feed::{VaultPriceFeed, VaultPriceFeedInitArgs},
        ContractAddresses,
    },
    stylus_testing::provider::TestClient,
};

pub async fn init_vault_price_feed(
    gov: Arc<TestClient>,
    contracts: &ContractAddresses,
) -> VaultPriceFeed<TestClient> {
    let price_feed = VaultPriceFeedInitArgs {
        gov: gov.address(),
        max_strict_price_deviation: U256::zero(),
    }
    .init(gov, contracts.vault_price_feed)
    .await;

    macro_rules! config {
        ($token:ident, $price:expr) => {
            price_feed
                .set_token_config(contracts.tokens.$token, false)
                .await
                .unwrap();
            let amount = format!("{}000000000000000000000000000000", $price);
            price_feed
                .set_price(
                    contracts.tokens.$token,
                    U256::from_dec_str(&amount).unwrap(), // set default price $1
                )
                .await
                .unwrap();
        };
    }

    config!(weth, 1);
    config!(btc, 1);
    config!(atom, 1);
    config!(osmo, 1);
    config!(bnb, 1);
    config!(usdt, 1);
    config!(usdc, 1);

    price_feed
}
