use std::sync::Arc;

use ethers::types::U256;
use log::LevelFilter;
use omx_tests::{
    constants::{BTC_DECIMALS, ETH_DECIMALS, USDO_DECIMALS},
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
pub async fn test_sell_usdo() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user1.address())
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
        .set_price(contracts.tokens.btc.address(), to_price(60000))
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .tokens
        .mint_bnb(user0.address(), U256::from(100))
        .await;

    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(100)
    );
    contracts
        .tokens
        .bnb
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), U256::from(100))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .buy_usdo(contracts.tokens.bnb.address(), user0.address())
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(29700)
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::one()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(29700)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(100 - 1)
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::from(29700)
    );

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user1.address())
        .await
        .assert_revert_str("Vault: zero USDO amount");

    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), U256::from(15000))
        .await
        .unwrap();

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.btc.address(), user1.address())
        .await
        .assert_revert_str("Vault: zero redemption amount");

    contracts
        .vault
        .swap_manager
        .set_in_manager_mode(true)
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user1.address())
        .await
        .assert_revert_str("Vault: manager only");

    contracts
        .vault
        .swap_manager
        .set_manager(user0.address(), true)
        .await
        .unwrap();

    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user1.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::from(50 - 1)
    ); // (15000 / 300) => 50

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(29700 - 15000)
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(2)
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(29700 - 15000)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(100 - 1 - 50)
    );
    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::from(29700 - 15000)
    );
}

#[tokio::test]
pub async fn test_sell_usdo_after_price_increase() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;

    contracts
        .set_price(contracts.tokens.bnb.address(), to_price(300))
        .await;
    contracts
        .vault
        .set_bnb_config(contracts.tokens.bnb.address())
        .await;

    contracts
        .tokens
        .mint_bnb(user0.address(), U256::from(100))
        .await;

    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::zero()
    );

    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(100)
    );
    contracts
        .tokens
        .bnb
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), U256::from(100))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .buy_usdo(contracts.tokens.bnb.address(), user0.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(29700)
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );

    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::one()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(29700)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(100 - 1)
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::from(29700)
    );

    contracts
        .set_price(contracts.tokens.bnb.address(), to_price(400))
        .await;

    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::from(39600)
    );

    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(contracts.vault.vault.address(), U256::from(15000))
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.bnb.address(), user1.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::from(29700 - 15000)
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(2)
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(29700 - 15000)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.bnb.address())
            .await
            .unwrap(),
        U256::from(100 - 1 - 37)
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .bnb
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::from(37 - 1)
    ); // (15000 / 500) => 30
    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::from(24800)
    );
}

#[tokio::test]
pub async fn test_sell_usdo_redeem_based_on_price() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(60000))
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;

    contracts
        .tokens
        .mint_btc(user0.address(), U256::from(2 * 10u64.pow(8)))
        .await;

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.btc.address())
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
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user0.address())
            .await
            .unwrap(),
        expand_decimals(2, BTC_DECIMALS)
    );

    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .tokens
        .btc
        .connect_acc(user0.clone())
        .transfer(
            contracts.vault.vault.address(),
            expand_decimals(2, BTC_DECIMALS),
        )
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
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        expand_decimals(119640, USDO_DECIMALS)
    );

    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user0.address())
            .await
            .unwrap(),
        expand_decimals(119640, USDO_DECIMALS)
    );
    assert_eq!(
        contracts
            .tokens
            .usdo
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        expand_decimals(6, BTC_DECIMALS - 3)
    );
    assert_eq!(
        contracts
            .vault
            .swap_manager
            .usdo_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        expand_decimals(119640, USDO_DECIMALS)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        expand_decimals(1994, BTC_DECIMALS - 3)
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );

    contracts
        .set_price(contracts.tokens.btc.address(), to_price(80000))
        .await;

    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        expand_decimals(159520, USDO_DECIMALS)
    );
    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .transfer(
            contracts.vault.vault.address(),
            expand_decimals(10000, USDO_DECIMALS),
        )
        .await
        .unwrap();
    contracts
        .vault
        .swap_manager
        .connect_acc(user0.clone())
        .sell_usdo(contracts.tokens.btc.address(), user1.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .tokens
            .btc
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::from(12462500)
    );
    assert_eq!(
        contracts
            .vault
            .fee_manager
            .get_fee_reserve(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(637500)
    );
    assert_eq!(
        contracts
            .vault
            .vault
            .pool_amount(contracts.tokens.btc.address())
            .await
            .unwrap(),
        U256::from(186900000)
    );
    assert_eq!(
        contracts
            .staking
            .olp_manager_utils
            .get_aum_in_usdo()
            .await
            .unwrap(),
        expand_decimals(149520, USDO_DECIMALS)
    );
}
