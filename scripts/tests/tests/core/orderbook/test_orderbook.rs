use std::fs;

use ethers::types::{Address, U256};
use omx_tests::{
    constants::ETH_DECIMALS,
    contracts::{
        erc20::{Erc20InitArgs, ERC20_ABI},
        orderbook_increase::{OrderbookIncreaseInitArgs, ORDERBOOKINCREASE_ABI},
        orderbook_swap::OrderbookSwapInitArgs,
    },
    stylus_testing::provider::TestProvider,
    utils::{prices::expand_decimals, test_helpers::create_gov},
};

#[tokio::test]
async fn test_swap_order() {
    let gov = create_gov();

    gov.mint_eth(gov.address(), expand_decimals(1000, ETH_DECIMALS));

    let orderbook_bin =
        fs::read("../../artifacts/omx_orderbook_swap.wasm").expect("read swap_orderbook binaries");
    let orderbook = gov.deploy_contract(&orderbook_bin, ORDERBOOKINCREASE_ABI.clone(), "orderbook");

    let orderbook = OrderbookSwapInitArgs {
        gov: gov.address(),
        swap_router: Address::default(),
    }
    .init(gov.clone(), orderbook)
    .await;

    let erc20_bin = fs::read("../../artifacts/omx_erc20.wasm").expect("read erc20 binaries");
    let token_in = gov.deploy_contract(&erc20_bin, ERC20_ABI.clone(), "eth");
    let token_in = Erc20InitArgs {
        decimals: 18,
        gov: gov.address(),
        name: "eth".into(),
        symbol: "ETH".into(),
    }
    .init(gov.clone(), token_in)
    .await;

    let amount_in = expand_decimals(10, 18);
    token_in.mint(gov.address(), amount_in).await.unwrap();

    orderbook
        .get_swap_order(gov.address(), U256::zero())
        .await
        .unwrap_err();

    let index = orderbook.get_current_index(gov.address()).await.unwrap();
    assert_eq!(index, U256::zero());

    let token_out = Address::from_low_u64_be(456);
    let execution_fee = U256::from_dec_str("10000000000000000").unwrap();

    token_in
        .approve(orderbook.address(), amount_in)
        .await
        .unwrap();

    orderbook
        .create_swap_order(
            token_in.address(),
            token_out,
            amount_in,
            U256::zero(),
            U256::from(12345),
            true,
            execution_fee,
        )
        .value(execution_fee)
        .await
        .unwrap();

    let order = orderbook
        .get_swap_order(gov.address(), U256::zero())
        .await
        .unwrap();

    assert_eq!(
        order,
        (
            gov.address(),
            token_in.address(),
            token_out,
            amount_in,
            U256::zero(),
            U256::from(12345),
            true,
            execution_fee,
        ),
    );

    let index = orderbook.get_current_index(gov.address()).await.unwrap();
    assert_eq!(index, U256::from(1));

    orderbook.cancel_swap_order(U256::zero()).await.unwrap();

    orderbook
        .get_swap_order(gov.address(), U256::zero())
        .await
        .unwrap_err();

    let index = orderbook.get_current_index(gov.address()).await.unwrap();

    assert_eq!(index, U256::from(1));
}

#[tokio::test]
async fn test_increase_order() {
    let gov = create_gov();

    gov.mint_eth(gov.address(), expand_decimals(1000, ETH_DECIMALS));

    let orderbook_bin = fs::read("../../artifacts/omx_orderbook_increase.wasm")
        .expect("read increase_orderbook binaries");
    let orderbook = gov.deploy_contract(&orderbook_bin, ORDERBOOKINCREASE_ABI.clone(), "orderbook");
    let orderbook = OrderbookIncreaseInitArgs {
        gov: gov.address(),
        swap_router: Address::default(),
    }
    .init(gov.clone(), orderbook)
    .await;

    let erc20_bin = fs::read("../../artifacts/omx_erc20.wasm").expect("read erc20 binaries");
    let collateral_token = gov.deploy_contract(&erc20_bin, ERC20_ABI.clone(), "eth");
    let collateral_token = Erc20InitArgs {
        decimals: 18,
        gov: gov.address(),
        name: "eth".into(),
        symbol: "ETH".into(),
    }
    .init(gov.clone(), collateral_token)
    .await;

    let collateral_amount = expand_decimals(1, 18);
    collateral_token
        .mint(gov.address(), collateral_amount)
        .await
        .unwrap();

    orderbook
        .get_increase_order(gov.address(), U256::zero())
        .await
        .unwrap_err();

    let index_token = Address::from_low_u64_be(456);
    let size_delta = expand_decimals(10, 18); // 10x leverage
    let is_long = true;
    let trigger_price = U256::from(12345);
    let trigger_above_threshold = true;
    let execution_fee = U256::from_dec_str("10000000000000000").unwrap();

    collateral_token
        .approve(orderbook.address(), collateral_amount)
        .await
        .unwrap();

    orderbook
        .create_increase_order(
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        )
        .value(execution_fee)
        .await
        .unwrap();

    let order = orderbook
        .get_increase_order(gov.address(), U256::zero())
        .await
        .unwrap();

    assert_eq!(
        order,
        (
            gov.address(),
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        ),
    );
}
