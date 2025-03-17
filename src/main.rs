
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger;
mod pkg;
use reqwest::Client;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Aktifkan logging
    dotenv().ok();

    let client = Arc::new(Client::new());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone())) // Registrasi client
            .default_service(web::to(pkg::proxy_module::proxy::proxy))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

