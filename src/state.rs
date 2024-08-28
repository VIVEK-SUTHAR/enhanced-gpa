use crate::price_fetcher;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use std::time::Duration;
#[derive(Clone)]
pub struct AppState {
    pub rpc_client: Arc<RpcClient>,
    pub price_fetcher: Arc<price_fetcher::PriceFetcher>,
}

impl AppState {
    pub fn new() -> Self {
        let rpc_url = std::env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URLNot set");
        let connection = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));
        let cache_duration = Duration::from_secs(60);
        let fetcher = price_fetcher::PriceFetcher::new(cache_duration);
        Self {
            price_fetcher: Arc::new(fetcher),
            rpc_client: connection,
        }
    }
}
