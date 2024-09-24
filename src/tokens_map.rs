use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::sync::RwLock;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub decimals: Option<u8>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

pub static RW_TOKEN_MAP: Lazy<RwLock<HashMap<String, TokenInfo>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
async fn fetch_and_save_token_list() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://ipfs.filebase.io/ipfs/QmVsqPqSDm6wqYHhiXEJB1hBxpv6Qz1KkW5sUMDKgAq2X6";
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let body = response.text().await?;
        let mut file = File::create("tokens-list.json")?;
        file.write_all(body.as_bytes())?;
        tracing::info!("Successfully fetched and saved the token list.");
    } else {
        return Err(format!("Failed to fetch data: HTTP {}", response.status()).into());
    }

    Ok(())
}

pub async fn load_token_list() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("tokens-list.json").exists() {
        fetch_and_save_token_list().await?;
    }

    let file = File::open("tokens-list.json")?;
    let reader = BufReader::new(file);
    let token_list: Vec<TokenInfo> = serde_json::from_reader(reader)?;

    let token_map = token_list
        .into_iter()
        .filter(|token| token.name.is_some() && token.symbol.is_some())
        .map(|token| (token.address.clone(), token))
        .collect::<HashMap<_, _>>();

    let mut rw_map = RW_TOKEN_MAP.write().unwrap();
    *rw_map = token_map;

    Ok(())
}

pub fn get_token_info(address: &str) -> Option<TokenInfo> {
    let token_map = RW_TOKEN_MAP.read().unwrap();
    let result = token_map.get(address).cloned();
    result
}
