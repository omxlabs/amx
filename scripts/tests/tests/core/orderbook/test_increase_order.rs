use std::fs;

use ethers::types::{Address, U256};
use omx_tests::{
    constants::ETH_DECIMALS,
    contracts::{
        erc20::{Erc20InitArgs, ERC20_ABI},
        orderbook_increase::{OrderbookIncreaseInitArgs, ORDERBOOKINCREASE_ABI},
    },
    stylus_testing::provider::TestProvider,
    utils::{
        prices::expand_decimals,
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};

#[tokio::test]
async fn test_increase_order_two_traders() {
    // Set up contracts and gov user
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

    // Create two users
    let user1 = create_user(gov.clone(), 1, "100").await;
    let user2 = create_user(gov.clone(), 2, "150").await;

    // Approve and mint tokens for users
    let collateral_amount = expand_decimals(1, 18);
    for user in &[user1.clone(), user2.clone()] {
        collateral_token
            .mint(user.address(), collateral_amount)
            .await
            .unwrap();

        collateral_token
            .connect_acc(user.clone())
            .approve(orderbook.address(), collateral_amount)
            .await
            .unwrap();
    }

    // Create order parameters
    let index_token = Address::from_low_u64_be(456);
    let size_delta_1 = expand_decimals(10, 18);
    let size_delta_2 = expand_decimals(5, 18);
    let is_long = true;
    let trigger_price = U256::from(12345);
    let trigger_above_threshold = true;
    let execution_fee = U256::from_dec_str("10000000000000000").unwrap();

    // User1 places an order
    orderbook
        .connect_acc(user1.clone())
        .create_increase_order(
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta_1,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        )
        .value(execution_fee)
        .await
        .unwrap();

    let order = orderbook
        .get_increase_order(user1.address(), U256::zero())
        .await
        .unwrap();
    assert_eq!(
        order,
        (
            user1.address(),
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta_1,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        )
    );

    // User2 places a slightly different order
    orderbook
        .connect_acc(user2.clone())
        .create_increase_order(
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta_2,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        )
        .value(execution_fee)
        .await
        .unwrap();

    let order = orderbook
        .get_increase_order(user2.address(), U256::zero())
        .await
        .unwrap();

    assert_eq!(
        order,
        (
            user2.address(),
            collateral_amount,
            collateral_token.address(),
            index_token,
            size_delta_2,
            is_long,
            trigger_price,
            trigger_above_threshold,
            execution_fee,
        ),
    );
}
