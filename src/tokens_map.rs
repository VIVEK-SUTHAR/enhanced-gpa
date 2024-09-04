use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub decimals: Option<u8>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

pub static TOKEN_MAP: Lazy<Mutex<HashMap<String, TokenInfo>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn load_token_list() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("tokens-list.json")?;
    let reader = BufReader::new(file);

    let token_list: Vec<TokenInfo> = serde_json::from_reader(reader)?;

    let token_map = token_list
        .into_iter()
        .filter(|token| token.name.is_some() && token.symbol.is_some())
        .map(|token| (token.address.clone(), token))
        .collect::<HashMap<_, _>>();

    let mut map = TOKEN_MAP.lock().unwrap();
    *map = token_map;

    Ok(())
}

pub fn get_token_info(address: &str) -> Option<TokenInfo> {
    let token_map = TOKEN_MAP.lock().unwrap();
    token_map.get(address).cloned()
}
