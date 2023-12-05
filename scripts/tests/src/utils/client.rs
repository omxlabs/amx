use std::sync::Arc;

use ethers::{
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
};

use crate::stylus_testing::provider::{TestClient, TestInnerProvider};

pub async fn client_from_key<T>(private_key: &[u8]) -> Arc<TestClient> {
    let wallet = LocalWallet::from_bytes(private_key).unwrap();
    let provider = Provider::<TestInnerProvider>::new(TestInnerProvider::new());
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    client
}
