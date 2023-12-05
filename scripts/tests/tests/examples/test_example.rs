use std::sync::Arc;

use ethers::types::{Address, U256};

use log::LevelFilter;
use omx_tests::{
    constants::{
        ETH_DECIMALS, OMX_DECIMALS, OSMO_DECIMALS, PRICE_DECIMALS, USDC_DECIMALS, USD_DECIMALS,
    },
    contracts::ContractAddresses,
    init::{Contracts, ContractsInitArgs},
    stylus_testing::provider::{TestClient, TestProvider},
    utils::{
        logs::configure_logs,
        prices::expand_decimals,
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};

async fn print_balance(contracts: &Contracts, account: Address, msg: &str) {
    println!("{}", msg);
    println!(
        "\tusdc: {}",
        contracts
            .balance(contracts.tokens.usdc.address(), account)
            .await
    );
    println!(
        "\tusdo: {}",
        contracts
            .balance(contracts.tokens.usdo.address(), account)
            .await
    );
    println!(
        "\tosmo: {}",
        contracts
            .balance(contracts.tokens.osmo.address(), account)
            .await
    );
    println!(
        "\teth: {}",
        contracts
            .balance(contracts.tokens.weth.address(), account)
            .await
    );
}

pub async fn init() -> (Contracts, Arc<TestClient>) {
    configure_logs(LevelFilter::Info);

    let gov = create_gov();

    gov.mint_eth(gov.address(), expand_decimals(1000, ETH_DECIMALS));

    let addresses = ContractAddresses::deploy_contracts(gov.clone()).await;

    let contracts = ContractsInitArgs {
        min_profit_time: U256::from(60 * 60),
        gov: gov.address(),
    }
    .init(gov.clone(), &addresses)
    .await;

    contracts
        .set_price(addresses.tokens.usdc, expand_decimals(1, PRICE_DECIMALS))
        .await;
    contracts.vault.set_usdc_config(addresses.tokens.usdc).await;

    contracts
        .set_price(addresses.tokens.osmo, expand_decimals(50, PRICE_DECIMALS))
        .await;
    contracts.vault.set_osmo_config(addresses.tokens.osmo).await;

    contracts
        .set_price(addresses.tokens.weth, expand_decimals(1000, PRICE_DECIMALS))
        .await;
    contracts.vault.set_eth_config(addresses.tokens.weth).await;

    (contracts, gov)
}

/// Deposit and withdraw example
///
/// 1. Mint some USDC to the user
/// 2. Deposit USDC to the vault and receive USDO
/// 3. Withdraw USDC from the vault by spending USDO
#[tokio::test]
pub async fn test_examples_deposit_withdraw() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;

    // mint some usdc tokens to the user

    contracts
        .tokens
        .usdc
        .mint(user0.address(), expand_decimals(1, USDC_DECIMALS))
        .await
        .unwrap();
    // allow router to spend user's usdc

    contracts
        .tokens
        .usdc
        .connect_acc(user0.clone())
        .increase_allowance(
            contracts.router.swap.address(),
            expand_decimals(1, USDC_DECIMALS),
        )
        .await
        .unwrap();

    println!();
    print_balance(&contracts, user0.address(), "user0 state before deposit").await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state before deposit",
    )
    .await;

    // ============== DEPOSIT ==============

    // Deposit USDC to the vault
    contracts
        .router
        .swap
        .connect_acc(user0.clone())
        .swap(
            vec![
                contracts.tokens.usdc.address(),
                contracts.tokens.usdo.address(),
            ],
            expand_decimals(1, USDC_DECIMALS),
            U256::from_dec_str("990000000000000000").unwrap(), // 0.99usdo (the system will take some fees)
            user0.address(),
        )
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdc.address(), user0.address())
            .await,
        U256::zero()
    ); // user will spend all usdc

    assert_eq!(
        contracts
            .balance(contracts.tokens.usdo.address(), user0.address())
            .await,
        U256::from_dec_str("997000000000000000").unwrap()
    ); // user will receive 0.997usdo

    println!();
    print_balance(&contracts, user0.address(), "user0 state after deposit").await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after deposit",
    )
    .await;

    // ============== WITHDRAW ==============

    // allow router to spend user's usdo
    contracts
        .tokens
        .usdo
        .connect_acc(user0.clone())
        .approve(
            contracts.router.swap.address(),
            U256::from_dec_str("997000000000000000").unwrap(),
        )
        .await
        .unwrap();

    // Withdraw OLP from the vault
    contracts
        .router
        .swap
        .connect_acc(user0.clone())
        .swap(
            vec![
                contracts.tokens.usdo.address(),
                contracts.tokens.usdc.address(),
            ],
            U256::from_dec_str("997000000000000000").unwrap(), // 0.997usdo
            U256::from_dec_str("990000").unwrap(), // ~0.99usdc (the system will take some fees)
            user0.address(),
        )
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdc.address(), user0.address())
            .await,
        U256::from_dec_str("994009").unwrap(),
    ); // user will receive ~ 0.99usdc
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdo.address(), user0.address())
            .await,
        U256::zero()
    ); // user will spend all usdo

    println!();
    print_balance(&contracts, user0.address(), "user0 state after withdraw").await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after withdraw",
    )
    .await;
}

#[tokio::test]
async fn test_examples_swap() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;

    // mint USDC and deposit it directly to the vault
    contracts
        .deposit_to_vault(
            contracts.tokens.usdc.address(),
            expand_decimals(1000, USDC_DECIMALS),
        )
        .await;

    println!();
    print_balance(&contracts, user0.address(), "user0 state before swap").await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state before swap",
    )
    .await;

    // swap OSMO to USDC
    contracts
        .router
        .swap
        .connect_acc(user0.clone())
        .swap_eth_to_tokens(
            vec![
                contracts.tokens.weth.address(),
                contracts.tokens.usdc.address(),
            ],
            U256::zero(),
            user0.address(),
        )
        .value(expand_decimals(1, ETH_DECIMALS))
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdc.address(), user0.address())
            .await,
        U256::from_dec_str("999600000").unwrap(),
    );
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdo.address(), user0.address())
            .await,
        U256::zero()
    );
    assert_eq!(
        contracts
            .balance(contracts.tokens.weth.address(), user0.address())
            .await,
        U256::zero()
    );

    println!();
    print_balance(&contracts, user0.address(), "user0 state after swap").await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after swap",
    )
    .await;
}

#[tokio::test]
async fn test_examples_long_position() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;

    // set OSMO/USDO price 100
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await;

    // mint some tokens and deposit them to the vault's pool
    contracts
        .deposit_to_vault(
            contracts.tokens.usdc.address(),
            expand_decimals(5000, USDC_DECIMALS),
        )
        .await;

    contracts
        .deposit_to_vault(
            contracts.tokens.osmo.address(),
            expand_decimals(100, OSMO_DECIMALS),
        )
        .await;

    // mint some tokens to the user too
    contracts
        .tokens
        .mint_osmo(user0.address(), expand_decimals(1, OSMO_DECIMALS))
        .await;
    // allow router to spend user's osmo
    contracts
        .tokens
        .osmo
        .connect_acc(user0.clone())
        .increase_allowance(
            contracts.router.positions_increase.address(),
            expand_decimals(1, OSMO_DECIMALS),
        )
        .await
        .unwrap();

    // ================== OPEN LONG POSITION ==================
    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state before open long position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state before open long position",
    )
    .await;

    // open long position
    contracts
        .router
        .positions_increase
        .connect_acc(user0.clone())
        .increase_position(
            vec![contracts.tokens.osmo.address()],
            contracts.tokens.osmo.address(),
            expand_decimals(1, OSMO_DECIMALS),
            U256::zero(),
            expand_decimals(500, USD_DECIMALS),
            true,
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await
        .unwrap();

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after open long position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after open long position",
    )
    .await;

    // ================== CLOSE LONG POSITION ==================

    // change OSMO/USDO price from 100 to 125
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(125, PRICE_DECIMALS),
        )
        .await;

    // close long position
    contracts
        .router
        .positions_decrease
        .connect_acc(user0.clone())
        .decrease_position(
            contracts.tokens.osmo.address(),
            contracts.tokens.osmo.address(),
            expand_decimals(1, OSMO_DECIMALS),
            expand_decimals(500, USD_DECIMALS),
            true,
            user0.address(),
            expand_decimals(125, PRICE_DECIMALS),
        )
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdc.address(), user0.address())
            .await,
        U256::zero()
    );
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdo.address(), user0.address())
            .await,
        U256::zero()
    );
    // user will receive 1.792osmo, which is 0.792osmo profit (more then 3x leverage)
    assert_eq!(
        contracts
            .balance(contracts.tokens.osmo.address(), user0.address())
            .await,
        U256::from_dec_str("1792000").unwrap(),
    );

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after close long position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after close long position",
    )
    .await;
}

#[tokio::test]
async fn test_examples_short_position() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;

    // set OSMO/USDO price 100
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await;

    // mint some tokens and deposit them to the vault's pool
    contracts
        .deposit_to_vault(
            contracts.tokens.usdc.address(),
            expand_decimals(5000, USDC_DECIMALS),
        )
        .await;

    contracts
        .deposit_to_vault(
            contracts.tokens.osmo.address(),
            expand_decimals(100, OSMO_DECIMALS),
        )
        .await;

    // mint some tokens to the user too
    contracts
        .tokens
        .mint_usdc(user0.address(), expand_decimals(100, USDC_DECIMALS))
        .await;
    // allow router to spend user's osmo
    contracts
        .tokens
        .usdc
        .connect_acc(user0.clone())
        .increase_allowance(
            contracts.router.positions_increase.address(),
            expand_decimals(100, USDC_DECIMALS),
        )
        .await
        .unwrap();

    // ================== OPEN SHORT POSITION ==================
    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state before open short position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state before open short position",
    )
    .await;

    // open short position
    contracts
        .router
        .positions_increase
        .connect_acc(user0.clone())
        .increase_position(
            vec![contracts.tokens.usdc.address()],
            contracts.tokens.osmo.address(),
            expand_decimals(100, USDC_DECIMALS),
            U256::zero(),
            expand_decimals(300, USD_DECIMALS),
            false,
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await
        .unwrap();

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after open short position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after open short position",
    )
    .await;

    // ================== CLOSE SHORT POSITION ==================

    // change OSMO/USDO price from 100 to 75
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(75, PRICE_DECIMALS),
        )
        .await;

    // close short position
    contracts
        .router
        .positions_decrease
        .connect_acc(user0.clone())
        .decrease_position(
            contracts.tokens.usdc.address(),
            contracts.tokens.osmo.address(),
            expand_decimals(100, USDC_DECIMALS),
            expand_decimals(300, USD_DECIMALS),
            false,
            user0.address(),
            expand_decimals(75, PRICE_DECIMALS),
        )
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.usdc.address(), user0.address())
            .await,
        U256::from_dec_str("174400000").unwrap(),
    );
    assert_eq!(
        contracts
            .balance(
                contracts.tokens.usdc.address(),
                contracts.vault.vault.address()
            )
            .await,
        U256::from_dec_str("4925600000").unwrap(),
    );

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after close long position",
    )
    .await;
    print_balance(
        &contracts,
        contracts.vault.vault.address(),
        "vault state after close long position",
    )
    .await;
}

#[tokio::test]
async fn test_examples_liquidate_position() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 1).await;
    let user1 = create_user(gov.clone(), 1, 1).await;

    // set OSMO/USDO price 100
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await;

    // mint some tokens and deposit them to the vault's pool
    contracts
        .deposit_to_vault(
            contracts.tokens.usdc.address(),
            expand_decimals(5000, USDC_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            contracts.tokens.osmo.address(),
            expand_decimals(100, OSMO_DECIMALS),
        )
        .await;

    // mint some tokens to the user too
    contracts
        .tokens
        .mint_osmo(user0.address(), expand_decimals(1, OSMO_DECIMALS))
        .await;
    // allow router to spend user's osmo
    contracts
        .tokens
        .osmo
        .connect_acc(user0.clone())
        .increase_allowance(
            contracts.router.positions_increase.address(),
            expand_decimals(1, OSMO_DECIMALS),
        )
        .await
        .unwrap();

    // ================== OPEN LONG POSITION ==================
    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state before open long position",
    )
    .await;
    print_balance(
        &contracts,
        user1.address(),
        "user1 state before open long position",
    )
    .await;

    // open long position
    contracts
        .router
        .positions_increase
        .connect_acc(user0.clone())
        .increase_position(
            vec![contracts.tokens.osmo.address()],
            contracts.tokens.osmo.address(),
            expand_decimals(1, OSMO_DECIMALS),
            U256::zero(),
            expand_decimals(500, USD_DECIMALS),
            true,
            expand_decimals(100, PRICE_DECIMALS),
        )
        .await
        .unwrap();

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after open long position",
    )
    .await;
    print_balance(
        &contracts,
        user1.address(),
        "user1 state after open long position",
    )
    .await;

    // ================== LIQUIDATE POSITION ==================

    // make user1 a liquidator
    contracts
        .vault
        .positions_liquidation_manager
        .set_liquidator(user1.address(), true)
        .await
        .unwrap();

    // change OSMO/USDO price form 100 to 50
    contracts
        .set_price(
            contracts.tokens.osmo.address(),
            expand_decimals(50, PRICE_DECIMALS),
        )
        .await;

    // liquidate position
    contracts
        .vault
        .positions_liquidation_manager
        .connect_acc(user1.clone())
        .liquidate_position(
            user0.address(),
            contracts.tokens.osmo.address(),
            contracts.tokens.osmo.address(),
            true,
            user1.address(),
        )
        .await
        .unwrap();

    // check balances
    assert_eq!(
        contracts
            .balance(contracts.tokens.osmo.address(), user0.address())
            .await,
        U256::zero()
    );
    assert_eq!(
        contracts
            .balance(contracts.tokens.osmo.address(), user1.address())
            .await,
        U256::from_dec_str("1000").unwrap()
    );
    assert_eq!(
        contracts
            .balance(
                contracts.tokens.osmo.address(),
                contracts.vault.vault.address()
            )
            .await,
        U256::from_dec_str("100999000").unwrap()
    );

    println!();
    print_balance(
        &contracts,
        user0.address(),
        "user0 state after liquidation of long position",
    )
    .await;
    print_balance(
        &contracts,
        user1.address(),
        "user1 state after liquidation of long position",
    )
    .await;
}

#[tokio::test]
async fn test_examples_stake_omx() {
    let (contracts, gov) = init().await;

    let amount = expand_decimals(10, OMX_DECIMALS);

    // mint omx to the user
    contracts.tokens.mint_omx(gov.address(), amount).await;
    contracts
        .tokens
        .omx
        .approve(
            contracts.staking.staked_omx_tracker_staking.address(),
            amount,
        )
        .await
        .unwrap();

    // stake omx
    contracts
        .staking
        .reward_router
        .stake_omx(amount)
        .await
        .unwrap();

    // skip 1 day
    gov.advance_block_timestamp(24 * 60 * 60);
    gov.mine_block();

    // unstake omx
    contracts
        .staking
        .reward_router
        .unstake_omx(amount)
        .await
        .unwrap();
}
