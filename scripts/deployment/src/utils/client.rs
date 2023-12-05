use std::{str::FromStr, sync::Arc};

use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};

use crate::contracts::LiveClient;

use super::private_key::key_from_file;

pub async fn client_from_str_key(
    key_path: impl AsRef<str>,
    endpoint: impl AsRef<str>,
) -> Arc<LiveClient> {
    let private_key = key_from_file(key_path.as_ref());
    let wallet = LocalWallet::from_str(&private_key).unwrap();
    let provider = Provider::<Http>::try_from(endpoint.as_ref()).unwrap();
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    client
}
