use std::sync::Arc;

use ethers::types::U256;

use log::LevelFilter;
use omx_tests::{
    constants::{ETH_DECIMALS, OMX_DECIMALS},
    contracts::{reward_router, reward_tracker::AmountExceedsDepositBalance, ContractAddresses},
    init::{Contracts, ContractsInitArgs},
    stylus_testing::provider::{TestClient, TestProvider},
    utils::{
        errors::ContractRevertExt,
        logs::configure_logs,
        prices::expand_decimals,
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};

pub async fn init() -> (Contracts, Arc<TestClient>) {
    configure_logs(LevelFilter::Info);

    let gov = create_gov();

    let addresses = ContractAddresses::deploy_contracts(gov.clone()).await;

    let contracts = ContractsInitArgs {
        min_profit_time: U256::from(60 * 60),
        gov: gov.address(),
    }
    .init(gov.clone(), &addresses)
    .await;

    // mint es_omx for distributors
    contracts
        .tokens
        .es_omx
        .set_minter(gov.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .mint_es_omx(
            contracts.staking.staked_omx_distributor.address(),
            expand_decimals(50000, OMX_DECIMALS),
        )
        .await;
    contracts
        .staking
        .staked_omx_distributor
        .set_tokens_per_interval(U256::from_dec_str("20667989410000000").unwrap())
        .await
        .unwrap();
    contracts
        .tokens
        .mint_es_omx(
            contracts.staking.staked_olp_distributor.address(),
            expand_decimals(50000, OMX_DECIMALS),
        )
        .await;
    contracts
        .staking
        .staked_olp_distributor
        .set_tokens_per_interval(U256::from_dec_str("20667989410000000").unwrap())
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_in_private_transfer_mode(true)
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_handler(contracts.staking.staked_omx_distributor.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_handler(contracts.staking.staked_olp_distributor.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_handler(contracts.staking.staked_omx_tracker_staking.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_handler(contracts.staking.staked_olp_tracker_staking.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .es_omx
        .set_handler(contracts.staking.reward_router.address(), true)
        .await
        .unwrap();

    // mint bn_omx for distributor
    contracts
        .tokens
        .bn_omx
        .set_minter(gov.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .mint_bn_omx(
            contracts.staking.bonus_omx_distributor.address(),
            expand_decimals(1500, OMX_DECIMALS),
        )
        .await;

    (contracts, gov)
}

#[tokio::test]
async fn test_stake_unstake_claim() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;
    let user2 = create_user(gov.clone(), 2, 0).await;
    contracts
        .tokens
        .mint_weth(
            contracts.staking.fee_omx_distributor.address(),
            expand_decimals(100, ETH_DECIMALS),
        )
        .await;
    contracts
        .staking
        .fee_omx_distributor
        .set_tokens_per_interval(U256::from_dec_str("41335970000000").unwrap())
        .await
        .unwrap();
    contracts
        .tokens
        .omx
        .set_minter(gov.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .mint_omx(user0.address(), expand_decimals(1500, OMX_DECIMALS))
        .await;
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user0.address())
            .await
            .unwrap(),
        expand_decimals(1500, OMX_DECIMALS)
    );
    contracts
        .tokens
        .omx
        .connect_acc(user0.clone())
        .approve(
            contracts.staking.staked_omx_tracker_staking.address(),
            expand_decimals(1000, OMX_DECIMALS),
        )
        .await
        .unwrap();
    contracts
        .staking
        .reward_router
        .connect_acc(user0.clone())
        .stake_omx_for_account(user1.address(), expand_decimals(1000, OMX_DECIMALS))
        .await
        .assert_revert(reward_router::Forbidden {});
    contracts
        .staking
        .reward_router
        .set_gov(user0.address())
        .await
        .unwrap();
    contracts
        .staking
        .reward_router
        .connect_acc(user0.clone())
        .stake_omx_for_account(user1.address(), expand_decimals(800, OMX_DECIMALS))
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user0.address())
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    contracts
        .tokens
        .mint_omx(user1.address(), expand_decimals(200, OMX_DECIMALS))
        .await;
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        expand_decimals(200, OMX_DECIMALS)
    );
    contracts
        .tokens
        .omx
        .connect_acc(user1.clone())
        .approve(
            contracts.staking.staked_omx_tracker_staking.address(),
            expand_decimals(200, OMX_DECIMALS),
        )
        .await
        .unwrap();
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .stake_omx(expand_decimals(200, OMX_DECIMALS))
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user0.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .deposit_balance(
                user0.address(),
                contracts.staking.staked_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.staked_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user0.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    gov.advance_block_timestamp(24 * 60 * 60);
    gov.mine_block();
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("1785714285024000000000").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("2739726027397260273").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3571427808000000000").unwrap()
    );
    contracts
        .tokens
        .es_omx
        .set_minter(gov.address(), true)
        .await
        .unwrap();
    contracts
        .tokens
        .mint_es_omx(user2.address(), expand_decimals(500, OMX_DECIMALS))
        .await;
    contracts
        .staking
        .reward_router
        .connect_acc(user2.clone())
        .stake_es_omx(expand_decimals(500, OMX_DECIMALS))
        .await
        .unwrap();
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user0.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user2.address())
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user2.address(), contracts.tokens.es_omx.address())
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .deposit_balance(
                user0.address(),
                contracts.staking.staked_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.staked_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user2.address())
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .deposit_balance(
                user2.address(),
                contracts.staking.staked_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user0.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user2.address())
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user2.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(500, OMX_DECIMALS)
    );
    gov.advance_block_timestamp(24 * 60 * 60);
    gov.mine_block();
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("2976190475040000000000").unwrap()
    );

    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .claimable(user2.address())
            .await
            .unwrap(),
        U256::from_dec_str("595238095008000000000").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("5479452054794520546").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .claimable(user2.address())
            .await
            .unwrap(),
        U256::from_dec_str("1369863013698630136").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .claimable(user0.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .claimable(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("5952379680000000000").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .claimable(user2.address())
            .await
            .unwrap(),
        U256::from_dec_str("1190475936000000000").unwrap()
    );
    assert_eq!(
        contracts
            .tokens
            .es_omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .claim_es_omx()
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .es_omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("2976190475040000000000").unwrap()
    );
    assert_eq!(
        contracts
            .tokens
            .weth
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .claim_fees()
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .weth
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("5952379680000000000").unwrap()
    );
    assert_eq!(
        contracts
            .tokens
            .es_omx
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user2.clone())
        .claim_es_omx()
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .es_omx
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::from_dec_str("595238095008000000000").unwrap()
    );
    assert_eq!(
        contracts
            .tokens
            .weth
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user2.clone())
        .claim_fees()
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .weth
            .balance_of(user2.address())
            .await
            .unwrap(),
        U256::from_dec_str("1190475936000000000").unwrap()
    );
    gov.advance_block_timestamp(24 * 60 * 60);
    gov.mine_block();
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .compound()
        .await
        .unwrap();
    gov.advance_block_timestamp(24 * 60 * 60);
    gov.mine_block();
    contracts
        .staking
        .reward_router
        .connect_acc(user0.clone())
        .batch_compound_for_accounts(vec![user1.address(), user2.address()])
        .await
        .unwrap();
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3644332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        expand_decimals(1000, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.es_omx.address())
            .await
            .unwrap(),
        U256::from_dec_str("2644332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3644332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3658552550744247299279").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::from_dec_str("3644332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.bn_omx.address())
            .await
            .unwrap(),
        U256::from_dec_str("14220482712372602737").unwrap()
    );
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        U256::zero()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .unstake_omx(expand_decimals(300, OMX_DECIMALS))
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        expand_decimals(300, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3344332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.es_omx.address())
            .await
            .unwrap(),
        U256::from_dec_str("2644332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3344332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("3357381926132090120985").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        U256::from_dec_str("3344332068031874696542").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.bn_omx.address())
            .await
            .unwrap(),
        U256::from_dec_str("13049858100215424443").unwrap()
    );
    let es_omx_balance_1 = contracts
        .tokens
        .es_omx
        .balance_of(user1.address())
        .await
        .unwrap();
    let es_omx_unstake_balance_1 = contracts
        .staking
        .staked_omx_tracker_staking
        .deposit_balance(user1.address(), contracts.tokens.es_omx.address())
        .await
        .unwrap();
    contracts
        .staking
        .reward_router
        .connect_acc(user1.clone())
        .unstake_es_omx(es_omx_unstake_balance_1)
        .await
        .unwrap();
    assert_eq!(
        contracts
            .tokens
            .es_omx
            .balance_of(user1.address())
            .await
            .unwrap(),
        es_omx_balance_1 + es_omx_unstake_balance_1
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.omx.address())
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .staked_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.es_omx.address())
            .await
            .unwrap(),
        U256::zero()
    );
    assert_eq!(
        contracts
            .staking
            .bonus_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .staked_amount(user1.address())
            .await
            .unwrap(),
        U256::from_dec_str("702731457428366749356").unwrap()
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(
                user1.address(),
                contracts.staking.bonus_omx_tracker.address()
            )
            .await
            .unwrap(),
        expand_decimals(700, OMX_DECIMALS)
    );
    assert_eq!(
        contracts
            .staking
            .fee_omx_tracker_staking
            .deposit_balance(user1.address(), contracts.tokens.bn_omx.address())
            .await
            .unwrap(),
        U256::from_dec_str("2731457428366749356").unwrap()
    );
    contracts
        .staking
        .reward_router
        .connect_acc(user1)
        .unstake_es_omx(expand_decimals(1, OMX_DECIMALS))
        .await
        .assert_revert(AmountExceedsDepositBalance {
            amount: expand_decimals(1, OMX_DECIMALS),
            deposit_balance: U256::zero(),
        });
}
