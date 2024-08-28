use crate::price_fetcher::PriceFetcher;
use crate::state;
use crate::tokens_map;
use actix_web::{web, HttpResponse};
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

#[actix_web::get("/getTokens/{address}")]
pub async fn get_tokens(
    state: web::Data<state::AppState>,
    address: web::Path<String>,
) -> HttpResponse {
    let address = address.into_inner();
    if address.is_empty() {
        return HttpResponse::BadRequest().body("No address provided");
    }
    let parsed_pub_key = Pubkey::from_str(&address);
    if parsed_pub_key.is_err() {
        return HttpResponse::BadRequest().body("Invalid pubkey provided");
    }
    match fetch_and_process_tokens(&state, &address).await {
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
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error>> {
    let accounts = fetch_token_accounts(&state.rpc_client, address).await?;
    let token_infos = process_accounts(accounts, &state.price_fetcher).await;
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
) -> Vec<TokenInfo> {
    let tasks: Vec<_> = accounts
        .into_iter()
        .map(|(_, account)| {
            let fetcher = fetcher.clone();
            task::spawn(async move { process_single_account(account, &fetcher).await })
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
) -> Option<TokenInfo> {
    let mint_token_account = SplTokenAccount::unpack_from_slice(&account.data).ok()?;
    let mint_address = mint_token_account.mint.to_string();
    let token_data = tokens_map::get_token_info(&mint_address)?;

    let decimals = token_data.decimals.unwrap_or(0);

    let real_amount = mint_token_account.amount as f64 / 10f64.powi(decimals as i32);

    let price = match fetcher.fetch_price(&mint_address).await {
        Ok(price) => price,
        Err(_) => {
            eprintln!("failed to fetch price for token: {}", mint_address);
            return None;
        }
    };

    let value = price * real_amount;

    Some(TokenInfo {
        mint_address,
        name: token_data.name.unwrap_or_default(),
        balance: real_amount,
        price,
        value,
        media: TokenImage {
            uri: token_data.logo_uri,
        },
    })
}
