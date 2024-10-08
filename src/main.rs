mod constants;
mod currencies;
mod exchange_rate;
mod get_tokens;
mod price_fetcher;
mod process;
mod server;
mod state;
mod tokens_map;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().init();
    //TODO: Move to Redis
    tracing::info!("Tokens Loadeding...");
    let token_load = tokens_map::load_token_list().await;
    match token_load {
        Ok(_) => {
            tracing::info!("Tokens Loaded");
        }
        Err(e) => {
            panic!("{}", format!("Token loading failed :{}", e));
        }
    }
    let app_state = state::AppState::new().await;
    server::start_server(app_state, "0.0.0.0:8081").await?;
    Ok(())
}
