use crate::get_tokens;
use crate::state;
use actix_web::HttpResponse;
use actix_web::{middleware::Logger, web, App, HttpServer};
pub async fn start_server(app_state: state::AppState, addr: &str) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .service(get_tokens::get_tokens)
            .default_service(web::route().to(|| async {
                Err::<HttpResponse, actix_web::Error>(actix_web::error::ErrorNotFound(
                    "Route not found",
                ))
            }))
    })
    .bind(addr)?
    .run()
    .await
}
