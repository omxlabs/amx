use ethers::types::{Address, U256};

use crate::{
    contracts::{
        vault_price_feed::{VaultPriceFeed, VaultPriceFeedInitArgs},
        ContractAddresses, DeployContext, LiveClient,
    },
    utils::contract_call_helper::send,
};

pub async fn init_vault_price_feed(
    ctx: &DeployContext,
    contracts: &ContractAddresses,
    gov: Address,
) -> VaultPriceFeed<LiveClient> {
    println!("initializing price feed contract");

    let price_feed = VaultPriceFeedInitArgs {
        gov,
        max_strict_price_deviation: U256::zero(),
    }
    .init(&ctx, contracts.vault_price_feed)
    .await;

    macro_rules! config {
        ($token:ident, $price:expr) => {
            send(price_feed.set_token_config(contracts.tokens.$token, false))
                .await
                .unwrap();
            let amount = format!("{}000000000000000000000000000000", $price);
            println!("\t{}: {}", stringify!($token), amount);
            send(price_feed.set_price(
                contracts.tokens.$token,
                U256::from_dec_str(&amount).unwrap(), // set default price $1
            ))
            .await
            .unwrap();
        };
    }

    println!("setting prices:");

    config!(weth, 2000);
    config!(btc, 1);
    config!(atom, 1);
    config!(osmo, 1);
    config!(bnb, 1);
    config!(usdt, 1);
    config!(usdc, 1);

    price_feed
}
