mod exchange_rate;
mod get_tokens;
mod price_fetcher;
mod server;
mod state;
mod tokens_map;
mod currencies;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let token_load = tokens_map::load_token_list();
    match token_load {
        Ok(_) => {
            println!("Tokens Loaded !");
        }
        Err(e) => {
            panic!("{}", format!("Token loading failed :{}", e));
        }
    }
    let app_state = state::AppState::new().await;
    server::start_server(app_state, "127.0.0.1:8080").await?;
    Ok(())
}
