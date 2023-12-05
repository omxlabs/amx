use std::{fs, sync::Arc};

use omx_tests::{
    constants::ETH_DECIMALS,
    contracts::weth::{Weth, WethInitArgs, WETH_ABI},
    stylus_testing::provider::{TestClient, TestProvider},
    utils::{
        prices::expand_decimals,
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};

pub async fn init() -> (Weth<TestClient>, Arc<TestClient>) {
    let gov = create_gov();

    let weth_bin = fs::read("../../artifacts/omx_weth.wasm").expect("read weth binaries");
    let weth = gov.deploy_contract(&weth_bin, WETH_ABI.clone(), "weth");
    let weth = WethInitArgs {
        name: "Wrapped Ether".to_string(),
        symbol: "WETH".to_string(),
    }
    .init(gov.clone(), weth)
    .await;

    (weth, gov)
}

#[tokio::test]
async fn test_weth_withdraw_deposit() {
    let (weth, client) = init().await;

    let user0 = create_user(client.clone(), 0, 5).await;

    assert_eq!(weth.balance_of(user0.address()).await.unwrap(), 0.into());

    weth.connect_acc(user0.clone())
        .deposit(user0.address())
        .value(expand_decimals(1, ETH_DECIMALS))
        .await
        .unwrap();

    assert_eq!(
        weth.balance_of(user0.address()).await.unwrap(),
        expand_decimals(1, ETH_DECIMALS),
    );

    weth.connect_acc(user0.clone())
        .deposit(user0.address())
        .value(expand_decimals(4, ETH_DECIMALS))
        .await
        .unwrap();

    assert_eq!(
        weth.balance_of(user0.address()).await.unwrap(),
        expand_decimals(5, ETH_DECIMALS),
    );
}

#[tokio::test]
async fn test_deposit_approve() {
    let (weth, client) = init().await;

    let user0 = create_user(client.clone(), 0, 2).await;
    let user1 = create_user(client.clone(), 1, 1).await;

    assert_eq!(weth.balance_of(user1.address()).await.unwrap(), 0.into());
    let amount = expand_decimals(1, ETH_DECIMALS);

    weth.connect_acc(user0.clone())
        .deposit_approve(user1.address())
        .value(amount)
        .await
        .unwrap();

    weth.connect_acc(user1.clone())
        .transfer_from(user0.address(), user1.address(), amount)
        .await
        .unwrap();

    assert_eq!(weth.balance_of(user1.address()).await.unwrap(), amount);
}
