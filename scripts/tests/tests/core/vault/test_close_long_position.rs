use std::sync::Arc;

use ethers::types::{I256, U256};
use log::LevelFilter;
use omx_tests::{
    constants::{BTC_DECIMALS, ETH_DECIMALS, USD_DECIMALS},
    contracts::ContractAddresses,
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
    contracts
        .tokens
        .yield_tracker
        .set_distributor(contracts.tokens.distributor.address())
        .await
        .unwrap();
    contracts
        .tokens
        .distributor
        .set_distribution(
            vec![contracts.tokens.yield_tracker.address()],
            vec![U256::from(1000)],
            vec![contracts.tokens.bnb.address()],
        )
        .await
        .unwrap();
    contracts
        .tokens
        .mint_bnb(contracts.tokens.distributor.address(), U256::from(5000))
        .await;
    contracts
        .tokens
        .usdo
        .set_yield_trackers(vec![contracts.tokens.yield_tracker.address()])
        .await
        .unwrap();

    (contracts, gov)
}

#[tokio::test]
async fn test_close_long_position() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;
    let user2 = create_user(gov.clone(), 2, 0).await;

    contracts
        .set_price(contracts.tokens.dai.address(), to_price(1))
        .await;
    contracts
        .vault
        .set_dai_config(contracts.tokens.dai.address())
        .await;

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(40000))
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .tokens
        .mint_btc(user1.address(), expand_decimals(1, BTC_DECIMALS))
        .await;
    contracts
        .tokens
        .btc
        .connect_acc(user1.clone())
        .transfer(contracts.vault.vault.address(), U256::from(250000))
        .await
        .unwrap();

    contracts
        .vault
        .swap_manager
        .buy_usdo(contracts.tokens.btc.address(), user1.address())
        .await
        .unwrap();

    contracts
        .tokens
        .mint_btc(user0.address(), expand_decimals(1, BTC_DECIMALS))
        .await;
    contracts
        .tokens
        .btc
        .connect_acc(user1.clone())
        .transfer(contracts.vault.vault.address(), U256::from(25000))
        .await
        .unwrap();
    contracts
        .vault
        .positions_increase_manager
        .connect_acc(user0.clone())
        .increase_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(110),
            true,
        )
        .await
        .assert_revert_str("Vault: pool_amount < reserved");

    contracts
        .vault
        .positions_increase_manager
        .connect_acc(user0.clone())
        .increase_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(90),
            true,
        )
        .await
        .unwrap();

    let position = contracts
        .vault
        .positions_manager
        .position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            true,
        )
        .await
        .unwrap();
    assert_eq!(position.0, to_price(90));
    assert_eq!(position.1, expand_decimals(991, USD_DECIMALS - 2));
    assert_eq!(position.2, to_price(40000));
    assert_eq!(position.3, U256::zero());
    assert_eq!(position.4, U256::from(225000));

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(45100))
        .await;

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(975)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .reserved_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(225000)
    );
    assert_eq!(
        contracts
            .vault
            .positions_manager
            .guaranteed_usd(contracts.tokens.btc.address())
            .await
            .unwrap(),
        expand_decimals(8009, USD_DECIMALS - 2)
    );

    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(274025)
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::zero()
    );

    let delta = contracts
        .vault
        .vault_utils
        .get_delta(
            contracts.tokens.btc.address(),
            position.0,
            position.2,
            true,
            position.6,
        )
        .await
        .unwrap();
    assert_eq!(delta, (true, expand_decimals(11475, USD_DECIMALS - 3)));

    contracts
        .vault
        .positions_decrease_manager
        .connect_acc(user0.clone())
        .decrease_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(4),
            to_price(90),
            true,
            user2.address(),
        )
        .await
        .unwrap();

    let position = contracts
        .vault
        .positions_manager
        .position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            true,
        )
        .await
        .unwrap();
    assert_eq!(position.0, U256::zero());
    assert_eq!(position.1, U256::zero());
    assert_eq!(position.2, U256::zero());
    assert_eq!(position.3, U256::zero());
    assert_eq!(position.4, U256::zero());
    assert_eq!(position.5, I256::zero());

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(1174)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .reserved_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .positions_manager
            .guaranteed_usd(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(226609)
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::from(47217)
    );

    contracts
        .validate_vault_balance(contracts.tokens.btc.address(), U256::zero())
        .await;
}

#[tokio::test]
async fn test_close_long_position_with_loss() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;
    let user2 = create_user(gov.clone(), 2, 0).await;

    contracts
        .set_price(contracts.tokens.dai.address(), to_price(1))
        .await;
    contracts
        .vault
        .set_dai_config(contracts.tokens.dai.address())
        .await;

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(40000))
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .tokens
        .mint_btc(user1.address(), expand_decimals(1, BTC_DECIMALS))
        .await;
    contracts
        .tokens
        .btc
        .connect_acc(user1.clone())
        .transfer(contracts.vault.vault.address(), U256::from(250000))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .buy_usdo(contracts.tokens.btc.address(), user1.address())
        .await
        .unwrap();

    contracts
        .tokens
        .mint_btc(user0.address(), expand_decimals(1, BTC_DECIMALS))
        .await;
    contracts
        .tokens
        .btc
        .connect_acc(user1.clone())
        .transfer(contracts.vault.vault.address(), U256::from(25000))
        .await
        .unwrap();
    contracts
        .vault
        .positions_increase_manager
        .connect_acc(user0.clone())
        .increase_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(110),
            true,
        )
        .await
        .assert_revert_str("Vault: pool_amount < reserved");

    contracts
        .vault
        .positions_increase_manager
        .connect_acc(user0.clone())
        .increase_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(90),
            true,
        )
        .await
        .unwrap();

    let position = contracts
        .vault
        .positions_manager
        .position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            true,
        )
        .await
        .unwrap();
    assert_eq!(position.0, to_price(90));
    assert_eq!(position.1, expand_decimals(991, USD_DECIMALS - 2));
    assert_eq!(position.2, to_price(40000));
    assert_eq!(position.3, U256::zero());
    assert_eq!(position.4, U256::from(225000));

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(39000))
        .await;

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(975)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .reserved_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(225000)
    );
    assert_eq!(
        contracts
            .vault
            .positions_manager
            .guaranteed_usd(contracts.tokens.btc.address())
            .await
            .unwrap(),
        expand_decimals(8009, USD_DECIMALS - 2)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(274025)
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::zero()
    );

    let delta = contracts
        .vault
        .vault_utils
        .get_delta(
            contracts.tokens.btc.address(),
            position.0,
            position.2,
            true,
            position.6,
        )
        .await
        .unwrap();
    assert_eq!(
        delta,
        (
            false,
            U256::from_dec_str("2250000000000000000000000000000").unwrap()
        )
    );

    contracts
        .vault
        .positions_decrease_manager
        .connect_acc(user0.clone())
        .decrease_position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            to_price(4),
            to_price(90),
            true,
            user2.address(),
        )
        .await
        .unwrap();

    let position = contracts
        .vault
        .positions_manager
        .position(
            user0.address(),
            contracts.tokens.btc.address(),
            contracts.tokens.btc.address(),
            true,
        )
        .await
        .unwrap();
    assert_eq!(position.0, U256::zero());
    assert_eq!(position.1, U256::zero());
    assert_eq!(position.2, U256::zero());
    assert_eq!(position.3, U256::zero());
    assert_eq!(position.4, U256::zero());
    assert_eq!(position.5, I256::zero());

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(1205)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .reserved_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .positions_manager
            .guaranteed_usd(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(254384)
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::from(19410)
    );

    contracts
        .validate_vault_balance(contracts.tokens.btc.address(), U256::from(1))
        .await;
}
