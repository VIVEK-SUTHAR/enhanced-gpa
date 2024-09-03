mod constants;
mod currencies;
mod exchange_rate;
mod get_tokens;
mod price_fetcher;
mod server;
mod state;
mod tokens_map;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().init();
    //TODO: Move to Redis
    let token_load = tokens_map::load_token_list();
    match token_load {
        Ok(_) => {
            tracing::info!("Tokens Loaded");
        }
        Err(e) => {
            panic!("{}", format!("Token loading failed :{}", e));
        }
    }
    let app_state = state::AppState::new().await;
    server::start_server(app_state, "127.0.0.1:8080").await?;
    Ok(())
}
