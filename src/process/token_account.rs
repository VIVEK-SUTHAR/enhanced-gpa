use super::native_token::process_native_balance;
use crate::constants::DEFAULT_DECIMALS;
use crate::currencies;
use crate::exchange_rate::ExchangeRates;
use crate::get_tokens::{TokenImage, TokenInfo};
use crate::price_fetcher::PriceFetcher;
use crate::tokens_map;
use currencies::Currency;
use solana_sdk::account::Account;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Account as SplTokenAccount;
use std::sync::Arc;

pub async fn process_accounts(
    accounts: Vec<(Pubkey, Account)>,
    native_balance: u64,
    fetcher: Arc<PriceFetcher>,
    exchange_rates: Arc<ExchangeRates>,
    currency: Currency,
    sort_by_value: bool,
) -> Vec<TokenInfo> {
    let tasks: Vec<_> = accounts
        .into_iter()
        .map(|(_, account)| {
            let fetcher = Arc::clone(&fetcher);
            let exchange_rates = Arc::clone(&exchange_rates);
            tokio::spawn(async move {
                process_single_account(account, &fetcher, &exchange_rates, currency).await
            })
        })
        .collect();

    let native_balance_task = tokio::spawn({
        let fetcher = Arc::clone(&fetcher);
        let exchange_rates = Arc::clone(&exchange_rates);
        async move { process_native_balance(&fetcher, native_balance, &exchange_rates, currency).await }
    });

    let mut results = futures::future::join_all(tasks).await;

    if let Ok(native_balance_result) = native_balance_task.await {
        results.push(Ok(native_balance_result));
    }

    let mut token_info_list: Vec<TokenInfo> = results
        .into_iter()
        .filter_map(|result| result.ok().and_then(|r| r))
        .collect();

    if sort_by_value {
        token_info_list.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    token_info_list
}

async fn process_single_account(
    account: solana_sdk::account::Account,
    fetcher: &PriceFetcher,
    exchange_rates: &ExchangeRates,
    currency: Currency,
) -> Option<TokenInfo> {
    let mint_token_account = SplTokenAccount::unpack_from_slice(&account.data).ok()?;

    let mint_address = mint_token_account.mint.to_string();

    let token_data = tokens_map::get_token_info(&mint_address)?;

    if mint_token_account.amount == 0 {
        return None;
    }

    let decimals = token_data.decimals.unwrap_or(DEFAULT_DECIMALS);

    let real_amount = mint_token_account.amount as f64 / 10f64.powi(decimals as i32);

    let usd_price = match fetcher.fetch_price(&mint_address).await {
        Ok(price) => price,
        Err(_) => {
            tracing::error!("failed to fetch price for token: {}", mint_address);
            return None;
        }
    };

    let converted_total_value =
        ExchangeRates::convert(exchange_rates, usd_price * real_amount, currency)?;

    let converted_token_price = ExchangeRates::convert(exchange_rates, usd_price, currency)?;

    Some(TokenInfo {
        mint_address,
        symbol: token_data.symbol,
        name: token_data.name.unwrap_or_default(),
        balance: real_amount,
        price: converted_token_price,
        value: converted_total_value,
        media: TokenImage {
            uri: token_data.logo_uri,
        },
    })
}
