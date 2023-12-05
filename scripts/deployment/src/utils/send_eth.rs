use std::sync::Arc;

use ethers::{
    middleware::SignerMiddleware,
    signers::Signer,
    types::{Address, TransactionRequest},
    utils::parse_ether,
};
use ethers_providers::Middleware;

pub async fn send_eth<M, S>(from: Arc<SignerMiddleware<M, S>>, to: Address, amount: impl ToString)
where
    M: Middleware,
    S: Signer,
{
    let tx = TransactionRequest::new()
        .to(to)
        .value(parse_ether(amount).unwrap())
        .from(from.address());

    from.send_transaction(tx, None)
        .await
        .unwrap()
        .await
        .unwrap();
}
