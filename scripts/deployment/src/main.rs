use std::{env, fs, str::FromStr};

use ethers::types::{Address, U256};
use omx_deploy::{
    constants::{
        ATOM_DECIMALS, BNB_DECIMALS, BTC_DECIMALS, DEV_OMX_AMOUNT, EXECUTION_PATH, OMX_DECIMALS,
        OSMO_DECIMALS, TESTNET_ENDPOINT, USDC_DECIMALS, USDT_DECIMALS,
    },
    contracts::{ContractAddresses, DeployContext},
    init::ContractsInitArgs,
    utils::{contract_call_helper::map_contract_error, prices::expand_decimals},
};

#[tokio::main]
async fn main() {
    let execution_path = fs::canonicalize(EXECUTION_PATH).expect("canonicalize");
    env::set_current_dir(&execution_path).expect("set_current_dir");

    let endpoint = TESTNET_ENDPOINT;

    let ctx = DeployContext::new(endpoint, "./key").await;

    let gov = ctx.client().address();

    let addresses = ContractAddresses::deploy_contracts(&ctx).await;

    println!(
        "contracts deployed: {}",
        serde_json::to_string_pretty(&addresses).unwrap()
    );

    let contracts = ContractsInitArgs {
        min_profit_time: U256::from(60 * 60),
        gov,
    }
    .init(&ctx, &addresses)
    .await;

    println!("configuring tokens");
    contracts
        .vault
        .set_eth_config(contracts.tokens.weth.address())
        .await;
    contracts
        .vault
        .set_btc_config(contracts.tokens.btc.address())
        .await;
    contracts
        .vault
        .set_atom_config(contracts.tokens.atom.address())
        .await;
    contracts
        .vault
        .set_osmo_config(contracts.tokens.osmo.address())
        .await;
    contracts
        .vault
        .set_bnb_config(contracts.tokens.bnb.address())
        .await;
    contracts
        .vault
        .set_usdt_config(contracts.tokens.usdt.address())
        .await;
    contracts
        .vault
        .set_usdc_config(contracts.tokens.usdc.address())
        .await;

    println!("depositing tokens to vault");
    contracts
        .deposit_to_vault(
            addresses.tokens.atom,
            expand_decimals(1_000_000, ATOM_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            addresses.tokens.osmo,
            expand_decimals(1_000_000, OSMO_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            addresses.tokens.bnb,
            expand_decimals(1_000_000, BNB_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            addresses.tokens.usdt,
            expand_decimals(1_000_000, USDT_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            addresses.tokens.usdc,
            expand_decimals(1_000_000, USDC_DECIMALS),
        )
        .await;
    contracts
        .deposit_to_vault(
            addresses.tokens.btc,
            expand_decimals(1_000_000, BTC_DECIMALS),
        )
        .await;

    // Address of frontend dev. He needs some tokens to test the frontend
    let dev_addresses = vec![
        "0x17dA053d90931da7698AB6561E52959D5Bad1a94",
        "0x0aa65ee00249b01397Cc738E1cd4B290122eC79F",
        "0x90D5D898284F4337264333035b4E3C54C47F691E",
        "0x3cc222811060826c2699EA8ff9b35FDB0963D6AD",
    ]
    .into_iter()
    .map(|v| Address::from_str(v).unwrap())
    .collect::<Vec<_>>();

    for dev_address in dev_addresses {
        println!("\tminting tokens for {dev_address}");
        contracts
            .tokens
            .mint_es_omx(dev_address, expand_decimals(DEV_OMX_AMOUNT, OMX_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_olp(dev_address, expand_decimals(DEV_OMX_AMOUNT, OMX_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_omx(dev_address, expand_decimals(DEV_OMX_AMOUNT, OMX_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_atom(dev_address, expand_decimals(DEV_OMX_AMOUNT, ATOM_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_osmo(dev_address, expand_decimals(DEV_OMX_AMOUNT, OSMO_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_bnb(dev_address, expand_decimals(DEV_OMX_AMOUNT, BNB_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_usdt(dev_address, expand_decimals(DEV_OMX_AMOUNT, USDT_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_usdc(dev_address, expand_decimals(DEV_OMX_AMOUNT, USDC_DECIMALS))
            .await;
        contracts
            .tokens
            .mint_btc(dev_address, expand_decimals(DEV_OMX_AMOUNT, BTC_DECIMALS))
            .await;
    }

    println!("getting price");
    let price = contracts
        .vault
        .vault
        .get_price(addresses.tokens.btc)
        .call()
        .await
        .map_err(map_contract_error)
        .unwrap();

    println!("price: {}", price);
}
