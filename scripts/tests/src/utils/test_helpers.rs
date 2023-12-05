use std::{str::FromStr, sync::Arc};

use ethers::{
    contract::Contract,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::U256,
};

use crate::{
    constants::{ETH_DECIMALS, INITIAL_TIMESTAMP},
    stylus_testing::provider::{TestClient, TestInnerProvider, TestProvider},
};

const USER_INDEX_OFFSET: u64 = 1000;

pub async fn create_user(
    gov: Arc<TestClient>,
    index: u64,
    eth_balance: impl ToString,
) -> Arc<TestClient> {
    let private_key = format!("{:064x}", index + USER_INDEX_OFFSET);

    let wallet = LocalWallet::from_str(&private_key).unwrap();
    let provider = gov.provider().clone();
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    let eth_balance = U256::from_dec_str(eth_balance.to_string().as_ref()).unwrap()
        * U256::from(10).pow(U256::from(ETH_DECIMALS));

    gov.mint_eth(wallet.address(), eth_balance);

    gov.set_label(wallet.address(), format!("user{}", index));

    Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ))
}

pub trait ConnectAcc<T, M>
where
    T: ::core::ops::Deref<Target = Contract<M>> + From<Contract<M>>,
    M: Middleware,
    Self: ::core::ops::Deref<Target = Contract<M>> + From<Contract<M>>,
{
    fn connect_acc(&self, account: Arc<M>) -> T;
}

impl<T, M> ConnectAcc<T, M> for T
where
    T: ::core::ops::Deref<Target = Contract<M>> + From<Contract<M>>,
    M: Middleware,
    Self: ::core::ops::Deref<Target = Contract<M>> + From<Contract<M>>,
{
    fn connect_acc(&self, account: Arc<M>) -> T {
        From::from(self.connect(account))
    }
}

pub fn create_gov() -> Arc<TestClient> {
    let private_key = format!("{:064x}", USER_INDEX_OFFSET - 1);
    let wallet = LocalWallet::from_str(&private_key).unwrap();

    let provider = Provider::new(TestInnerProvider::new());

    let gov = Arc::new(TestClient::new(provider, wallet));

    gov.advance_block_timestamp(INITIAL_TIMESTAMP);

    gov.set_label(gov.address(), "gov".to_string());

    gov
}
