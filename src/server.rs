use crate::get_tokens;
use crate::state;
use actix_cors::Cors;
use actix_web::HttpResponse;
use actix_web::{http, middleware::Logger, web, App, HttpServer};

pub async fn start_server(app_state: state::AppState, addr: &str) -> std::io::Result<()> {
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
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
