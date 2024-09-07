use crate::constants::{ACCOUNT_DATA_SIZE, MEM_OFFSET};
use crate::currencies;
use crate::process::token_account::process_accounts;
use crate::state;
use actix_web::{web, HttpResponse};
use currencies::Currency;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct TokenImage {
    pub uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint_address: String,
    pub name: String,
    pub balance: f64,
    pub value: f64,
    pub price: f64,
    pub symbol: Option<String>,
    pub media: TokenImage,
}

#[derive(Deserialize)]
struct TokenQuery {
    currency: Option<String>,
    sortbyvalue: Option<bool>,
}
#[actix_web::get("/getTokens/{address}")]
pub async fn get_tokens(
    state: web::Data<state::AppState>,
    address: web::Path<String>,
    query: web::Query<TokenQuery>,
) -> HttpResponse {
    let address = address.into_inner();
    let parsed_pub_key = Pubkey::from_str(&address);
    if parsed_pub_key.is_err() {
        return HttpResponse::BadRequest().body("Invalid pubkey provided");
    }
    let currency = if let Some(currency_str) = &query.currency {
        match currencies::Currency::from_str(currency_str) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Invalid currency provided: {}", currency_str)
                }));
            }
        }
    } else {
        currencies::Currency::USD
    };

    let should_sort_by_value = query.sortbyvalue.unwrap_or(false);

    match fetch_and_process_tokens(&state, &address, currency, should_sort_by_value).await {
        Ok(tokens) => HttpResponse::Ok().json(tokens),
        Err(e) => {
            eprintln!("Error processing tokens: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn fetch_and_process_tokens(
    state: &web::Data<state::AppState>,
    address: &str,
    currency: Currency,
    sort_by_value: bool,
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error>> {
    let (accounts, native_balance) = tokio::join!(
        fetch_token_accounts(&state.rpc_client, address),
        fetch_native_balance(&state.rpc_client, address)
    );
    let accounts = accounts?;
    let native_balance = native_balance?;
    let token_infos = process_accounts(
        accounts,
        native_balance,
        state.price_fetcher.clone(),
        state.exchange_rates.clone(),
        currency,
        sort_by_value,
    )
    .await;
    Ok(token_infos)
}

async fn fetch_native_balance(
    connection: &Arc<RpcClient>,
    address: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    let balance = connection
        .get_balance(&Pubkey::from_str(address).unwrap())
        .await?;
    Ok(balance)
}

async fn fetch_token_accounts(
    connection: &Arc<RpcClient>,
    address: &str,
) -> Result<Vec<(Pubkey, solana_sdk::account::Account)>, Box<dyn std::error::Error>> {
    let filters = vec![
        RpcFilterType::DataSize(ACCOUNT_DATA_SIZE),
        RpcFilterType::Memcmp(Memcmp::new(
            MEM_OFFSET,
            MemcmpEncodedBytes::Base58(address.to_string()),
        )),
    ];

    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::finalized()),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };

    connection
        .get_program_accounts_with_config(&spl_token::id(), config)
        .await
        .map_err(|e| e.into())
}
