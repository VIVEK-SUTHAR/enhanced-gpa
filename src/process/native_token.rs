use crate::constants::{LAMPORTS_PER_SOL, NATIVE_SOL_MINT};
use crate::currencies;
use crate::exchange_rate::ExchangeRates;
use crate::get_tokens::{TokenImage, TokenInfo};
use crate::price_fetcher::PriceFetcher;
use crate::tokens_map;
use currencies::Currency;

const NATIVE_TOKEN_NAME: &str = "Solana";
const NATIVE_TOKEN_SYMBOL: &str = "SOL";

pub async fn process_native_balance(
    fetcher: &PriceFetcher,
    native_balance_in_lamports: u64,
    exchange_rates: &ExchangeRates,
    currency: Currency,
) -> Option<TokenInfo> {
    let token_data = tokens_map::get_token_info(&NATIVE_SOL_MINT)?;
    let usd_price = match fetcher.fetch_price(&NATIVE_SOL_MINT).await {
        Ok(price) => price,
        Err(_) => {
            return None;
        }
    };
    let ui_amount = lamports_to_sol(native_balance_in_lamports);
    let converted_total_value =
        ExchangeRates::convert(exchange_rates, usd_price * ui_amount, currency)?;

    let converted_token_price = ExchangeRates::convert(exchange_rates, usd_price, currency)?;

    Some(TokenInfo {
        mint_address: NATIVE_SOL_MINT.to_string(),
        name: NATIVE_TOKEN_NAME.to_string(),
        balance: ui_amount,
        price: converted_token_price,
        symbol: Some(NATIVE_TOKEN_SYMBOL.to_string()),
        value: converted_total_value,
        media: TokenImage {
            uri: token_data.logo_uri,
        },
    })
}

fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL
}
