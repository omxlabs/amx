use std::sync::Arc;

use ethers::types::U256;

use omx_tests::{
    constants::ETH_DECIMALS,
    contracts::{reward_router::Forbidden, ContractAddresses},
    init::{Contracts, ContractsInitArgs},
    stylus_testing::provider::{TestClient, TestProvider},
    utils::{
        errors::ContractRevertExt,
        logs::configure_logs,
        prices::expand_decimals,
        test_helpers::{create_gov, create_user, ConnectAcc},
    },
};
use log::LevelFilter;

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
async fn test_set_handler() {
    let (contracts, gov) = init().await;

    let user0 = create_user(gov.clone(), 0, 0).await;
    let user1 = create_user(gov.clone(), 1, 0).await;

    contracts
        .staking
        .olp_manager
        .connect_acc(user0.clone())
        .set_handler(user1.address(), true)
        .await
        .assert_revert(Forbidden {});

    contracts
        .staking
        .olp_manager
        .set_gov(user0.address())
        .await
        .unwrap();

    assert_eq!(
        contracts
            .staking
            .olp_manager
            .is_handler(user1.address())
            .await
            .unwrap(),
        false
    );

    contracts
        .staking
        .olp_manager
        .connect_acc(user0.clone())
        .set_handler(user1.address(), true)
        .await
        .unwrap();

    assert_eq!(
        contracts
            .staking
            .olp_manager
            .is_handler(user1.address())
            .await
            .unwrap(),
        true
    );
}
