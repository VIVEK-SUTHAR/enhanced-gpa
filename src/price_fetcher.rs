use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
#[derive(Debug, Deserialize)]
pub struct PriceData {
    pub value: f64,
}
#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    pub data: PriceData,
}

pub struct CachedPrice {
    price: f64,
    timestamp: Instant,
}

#[derive(Clone)]
pub struct PriceFetcher {
    pub client: Client,
    pub cache: Arc<Mutex<HashMap<String, CachedPrice>>>,
    pub duration: Duration,
}

impl PriceFetcher {
    pub fn new(cache_duration: Duration) -> Self {
        PriceFetcher {
            client: Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            duration: cache_duration,
        }
    }

    pub async fn fetch_price(&self, address: &str) -> Result<f64, Box<dyn std::error::Error>> {
        {
            let cache = self.cache.lock().await;
            if let Some(cached_price) = cache.get(address) {
                if cached_price.timestamp.elapsed() < self.duration {
                    return Ok(cached_price.price);
                }
            }
        }

        let url = format!(
            "https://public-api.birdeye.so/defi/price?address={}",
            address
        );
        let birdeye_key = std::env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY Not set");
        let mut custom_headers = HeaderMap::new();
        custom_headers.insert("X-API-KEY", HeaderValue::from_str(&birdeye_key)?);
        custom_headers.insert("X-CHAIN", HeaderValue::from_static("solana"));
        custom_headers.insert("accept", HeaderValue::from_static("application/json"));
        let response = self.client.get(&url).headers(custom_headers).send().await?;

        let price_data: PriceResponse = response.json().await?;

        let price = price_data.data.value;

        let mut cache = self.cache.lock().await;
        cache.insert(
            address.to_string(),
            CachedPrice {
                price,
                timestamp: Instant::now(),
            },
        );

        Ok(price)
    }
}
