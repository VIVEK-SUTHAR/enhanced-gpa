use crate::currencies;
use crate::exchange_rate::ExchangeRates;
use crate::price_fetcher::PriceFetcher;
use crate::state;
use crate::tokens_map;
use actix_web::{web, HttpResponse};
use currencies::Currency;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::account::Account;
use solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Account as SplTokenAccount;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;
const DATA_SIZE: u64 = 165;
const MEM_OFFSET: usize = 32;

#[derive(Serialize, Deserialize)]
struct TokenImage {
    uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TokenInfo {
    mint_address: String,
    name: String,
    balance: f64,
    value: f64,
    price: f64,
    media: TokenImage,
}
#[derive(Deserialize)]
struct TokenQuery {
    currency: Option<String>,
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
    match fetch_and_process_tokens(&state, &address, currency).await {
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
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error>> {
    let accounts = fetch_token_accounts(&state.rpc_client, address).await?;
    let token_infos = process_accounts(
        accounts,
        &state.price_fetcher,
        &state.exchange_rates,
        currency,
    )
    .await;
    Ok(token_infos)
}

async fn fetch_token_accounts(
    connection: &Arc<RpcClient>,
    address: &str,
) -> Result<Vec<(Pubkey, solana_sdk::account::Account)>, Box<dyn std::error::Error>> {
    let filters = vec![
        RpcFilterType::DataSize(DATA_SIZE),
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

async fn process_accounts(
    accounts: Vec<(Pubkey, Account)>,
    fetcher: &PriceFetcher,
    exchange_rates: &ExchangeRates,
    currency: Currency,
) -> Vec<TokenInfo> {
    let tasks: Vec<_> = accounts
        .into_iter()
        .map(|(_, account)| {
            let fetcher = fetcher.clone();
            let exchange_rates = exchange_rates.clone();
            task::spawn(async move {
                process_single_account(account, &fetcher, &exchange_rates, currency).await
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    results
        .into_iter()
        .filter_map(|result| result.ok().and_then(|r| r))
        .collect()
}

async fn process_single_account(
    account: solana_sdk::account::Account,
    fetcher: &PriceFetcher,
    exchange_rates: &ExchangeRates,
    currency: Currency,
) -> Option<TokenInfo> {
    let mint_token_account = SplTokenAccount::unpack_from_slice(&account.data).ok()?;

    if mint_token_account.amount == 0 {
        return None;
    }

    let mint_address = mint_token_account.mint.to_string();

    let token_data = tokens_map::get_token_info(&mint_address)?;

    let decimals = token_data.decimals.unwrap_or(0);

    let real_amount = mint_token_account.amount as f64 / 10f64.powi(decimals as i32);

    let usd_price = match fetcher.fetch_price(&mint_address).await {
        Ok(price) => price,
        Err(_) => {
            eprintln!("failed to fetch price for token: {}", mint_address);
            return None;
        }
    };

    let converted_total_value =
        ExchangeRates::convert(exchange_rates, usd_price * real_amount, currency)?;

    let converted_token_price = ExchangeRates::convert(exchange_rates, usd_price, currency)?;

    Some(TokenInfo {
        mint_address,
        name: token_data.name.unwrap_or_default(),
        balance: real_amount,
        price: converted_token_price,
        value: converted_total_value,
        media: TokenImage {
            uri: token_data.logo_uri,
        },
    })
}
