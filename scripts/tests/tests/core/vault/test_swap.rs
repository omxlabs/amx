use std::sync::Arc;

use ethers::types::U256;

use ethers_providers::Middleware;
use log::LevelFilter;
use omx_tests::{
    constants::{BTC_DECIMALS, ETH_DECIMALS, USDC_DECIMALS},
    contracts::{swap_manager::SwapManager, ContractAddresses},
    init::{Contracts, ContractsInitArgs},
    stylus_testing::provider::{TestClient, TestProvider},
    utils::{
        errors::ContractRevertExt,
        logs::configure_logs,
        prices::{expand_decimals, to_price},
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};

pub async fn init() -> (Contracts, Arc<TestClient>) {
    configure_logs(LevelFilter::Info);

    let gov = create_gov();

    let addresses = ContractAddresses::deploy_contracts(gov.clone()).await;

    gov.mint_eth(gov.address(), expand_decimals(1000, ETH_DECIMALS));

    let contracts = ContractsInitArgs {
        min_profit_time: U256::from(60 * 60),
        gov: gov.address(),
    }
    .init(gov.clone(), &addresses)
    .await;

    (contracts, gov)
}

#[tokio::test]
pub async fn test_swap() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;
    let user1 = create_user(gov.clone(), 1, 1).await;
    let user2 = create_user(gov.clone(), 2, 1).await;
    let user3 = create_user(gov.clone(), 2, 1).await;

    contracts
        .vault
        .swap_manager
        .swap(
            contracts.tokens.bnb.address(),
            contracts.tokens.btc.address(),
            user1.address(),
        )
        .await
        .assert_revert_str("Vault: token not whitelisted");

    contracts
        .set_price(contracts.tokens.bnb.address(), to_price(300))
        .await;
    contracts
        .vault
        .set_bnb_config(contracts.tokens.bnb.address())
        .await;

    contracts
        .vault
        .swap_manager
        .connect_acc(user1.clone())
        .swap(
            contracts.tokens.bnb.address(),
            contracts.tokens.btc.address(),
            user2.address(),
        )
        .await
        .assert_revert_str("Vault: token not whitelisted");

    contracts
        .vault
        .swap_manager
        .connect_acc(user1.clone())
        .swap(
            contracts.tokens.bnb.address(),
            contracts.tokens.bnb.address(),
            user2.address(),
        )
        .await
        .assert_revert_str("Vault: token in and token out are the same");

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(60000))
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .tokens
        .mint_bnb(user0.address(), expand_decimals(200, 18))
        .await;
    contracts
        .tokens
        .mint_btc(user0.address(), expand_decimals(1, 8))
        .await;

    contracts
        .tokens
        .bnb
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(200, 18))
        .await
        .unwrap();

    SwapManager::from(contracts.vault.swap_manager.connect(user0.clone()))
        .buy_usdo(contracts.tokens.bnb.address(), user0.address())
        .await
        .unwrap();

    contracts
        .tokens
        .btc
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(1, 8))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .buy_usdo(contracts.tokens.btc.address(), user0.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(120000 - 360, 18),
    ); // 120,000 * 0.3% => 360

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("600000000000000000").unwrap()
    ); // 200 * 0.3% => 0.6
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(200 * 300 - 180, 18),
    ); // 60,000 * 0.3% => 180
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(200, 18) - U256::from_dec_str("600000000000000000").unwrap()
    );

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("300000").unwrap()
    ); // 1 * 0.3% => 0.003
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(200 * 300 - 180, 18),
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(1, 8) - U256::from_dec_str("300000").unwrap()
    );

    contracts
        .set_price(contracts.tokens.bnb.address(), to_price(500))
        .await;

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(90000))
        .await;

    contracts
        .tokens
        .mint_bnb(user1.address(), expand_decimals(100, 18))
        .await;
    contracts
        .tokens
        .bnb
        .connect_acc(user1.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(100, 18))
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user1.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );
    contracts
        .vault
        .swap_manager
        .connect_acc(user1.clone())
        .swap(
            contracts.tokens.bnb.address(),
            contracts.tokens.btc.address(),
            user2.address(),
        )
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user1.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("55388888").unwrap()
    );

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("600000000000000000").unwrap()
    ); // 200 * 0.3% => 0.6
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(109820, 18),
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(100, 18) + expand_decimals(200, 18)
            - U256::from_dec_str("600000000000000000").unwrap()
    );

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("466667").unwrap()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        expand_decimals(9820, 18),
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("44144445").unwrap()
    );

    contracts
        .set_price(contracts.tokens.bnb.address(), to_price(500))
        .await;

    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user3.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );

    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(50000, 18))
        .await
        .unwrap();

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user2.address())
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .call()
            .await
            .unwrap(),
        0.into()
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user3.address())
            .call()
            .await
            .unwrap(),
        U256::from_dec_str("99700000000000000000").unwrap()
    );

    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(30000, 18))
        .await
        .unwrap();

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.btc.address(), user3.address())
        .await
        .unwrap();

    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), expand_decimals(10000, 18))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.btc.address(), user2.address())
        .await
        .assert_revert_str("Vault: pool_amount exceeded");
}

/// swap native eth to btc
#[tokio::test]
async fn test_swap_eth_to_token() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;

    contracts
        .vault
        .set_eth_config(contracts.tokens.weth.address())
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .deposit_to_vault(
            contracts.tokens.btc.address(),
            expand_decimals(100, BTC_DECIMALS),
        )
        .await;

    assert_eq!(
        contracts
            .balance(contracts.tokens.btc.address(), user0.address())
            .await,
        U256::zero()
    );

    contracts
        .router
        .swap
        .connect_acc(user0.clone())
        .swap_eth_to_tokens(
            vec![
                contracts.tokens.weth.address(),
                contracts.tokens.btc.address(),
            ],
            U256::zero(),
            user0.address(),
        )
        .value(expand_decimals(1, 18))
        .await
        .unwrap();

    assert_eq!(
        contracts
            .balance(contracts.tokens.btc.address(), user0.address())
            .await,
        U256::from(99700000),
    );
}

/// swap usdc to native eth
#[tokio::test]
async fn test_swap_tokens_to_eth() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;

    contracts
        .vault
        .set_eth_config(contracts.tokens.weth.address())
        .await;
    contracts
        .vault
        .set_usdc_config(contracts.tokens.usdc.address())
        .await;

    contracts
        .tokens
        .mint_weth(
            contracts.vault.vault.address(),
            expand_decimals(100000, ETH_DECIMALS),
        )
        .await;
    contracts
        .vault
        .vault
        .direct_pool_deposit(contracts.tokens.weth.address())
        .await
        .unwrap();

    contracts
        .tokens
        .mint_usdc(gov.address(), expand_decimals(1, USDC_DECIMALS))
        .await;
    contracts
        .tokens
        .usdc
        .approve(
            contracts.router.swap.address(),
            expand_decimals(1, USDC_DECIMALS),
        )
        .await
        .unwrap();

    contracts
        .router
        .swap
        .swap_to_eth(
            contracts.tokens.usdc.address(),
            expand_decimals(1, USDC_DECIMALS),
            U256::zero(),
            user0.address(),
        )
        .await
        .unwrap();

    assert_eq!(
        user0.get_balance(user0.address(), None).await.unwrap(),
        U256::from_dec_str("999600000000000000").unwrap(),
    );
}
