use crate::currencies::Currency;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::{sync::Arc, sync::Mutex};

#[derive(Clone)]
pub struct ExchangeRates {
    pub client: Client,
    pub exchange_rate_cache: Arc<Mutex<HashMap<Currency, f64>>>,
}

#[derive(Deserialize)]
struct ExchangeRateResponse {
    conversion_rates: HashMap<String, f64>,
}

const BASE_CURRENCY: &str = "USD";

impl ExchangeRates {
    pub async fn new() -> Self {
        ExchangeRates {
            client: Client::new(),
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn setup_exchange_prices(&self) -> Result<(), Box<dyn std::error::Error>> {
        let exchangerate_api_key =
            std::env::var("EXCHANGE_RATE_API_KEY").expect("EXCHANGE_RATE_API_KEY not set");
        let url = format!(
            "https://v6.exchangerate-api.com/v6/{}/latest/{}",
            exchangerate_api_key, BASE_CURRENCY
        );

        let response: ExchangeRateResponse = self.client.get(&url).send().await?.json().await?;

        let mut cache = self.exchange_rate_cache.lock().unwrap();
        cache.clear();
        cache.insert(Currency::USD, 1.0);

        for (currency_str, rate) in response.conversion_rates {
            if let Ok(currency) = serde_json::from_value(serde_json::Value::String(currency_str)) {
                cache.insert(currency, rate);
            }
        }
        Ok(())
    }

    pub fn convert(&self, usd_amount: f64, to_currency: Currency) -> Option<f64> {
        let cache = self.exchange_rate_cache.lock().unwrap();
        let to_rate = cache.get(&to_currency)?;
        Some(usd_amount * to_rate)
    }
}
