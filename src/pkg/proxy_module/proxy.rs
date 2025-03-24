use actix_web::{web, HttpRequest, HttpResponse, http::header::{HeaderName, HeaderValue}, http::StatusCode};
use reqwest::{Client, Method, header::HeaderName as ReqHeaderName, header::HeaderValue as ReqHeaderValue};
use std::sync::Arc;
use std::env;
use crate::pkg::config::readerfile::load_config;

pub async fn proxy(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let host = req.headers().get("host").and_then(|h| h.to_str().ok()).unwrap_or(""); 
    let config_path = match env::var("CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => return HttpResponse::InternalServerError().body("CONFIG_PATH not found in .env"),
    };

    let config = match load_config(&config_path,host) {
        Ok(cfg) => cfg,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to load config"),
    };
    let target_url = format!("http://{}{}", config.destination, req.uri().to_string());
    // Convert Actix HTTP method to Reqwest method
    let req_method = match Method::from_bytes(req.method().as_str().as_bytes()) {
        Ok(method) => method,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid HTTP method"),
    };
    let mut req_builder = client.request(req_method, target_url);
    req_builder = req_builder.body(body.to_vec());
    // Convert headers
    for (key, value) in req.headers() {
        if let (Ok(req_key), Ok(req_value)) = (
            ReqHeaderName::from_bytes(key.as_str().as_bytes()),
            ReqHeaderValue::from_str(value.to_str().unwrap_or("")),
        ) {
            req_builder = req_builder.header(req_key, req_value);
        }
    }

    match req_builder.send().await {
        Ok(res) => {
            // Convert Reqwest status code to Actix status code
            let status = match StatusCode::from_u16(res.status().as_u16()) {
                Ok(code) => code,
                Err(_) => return HttpResponse::InternalServerError().body("Invalid response status"),
            };

            let mut response = HttpResponse::build(status);

            // Convert headers back to Actix
            for (key, value) in res.headers() {
                if let (Ok(actix_key), Ok(actix_value)) = (
                    key.to_string().parse::<HeaderName>(),
                    value.to_str().unwrap_or("").parse::<HeaderValue>(),
                ) {
                    response.insert_header((actix_key, actix_value));
                }
            }

            match res.bytes().await {
                Ok(body) => response.body(body),
                Err(_) => HttpResponse::InternalServerError().body("Failed to read response body"),
            }
        }
        Err(e) => {
            HttpResponse::BadGateway().body(format!("Bad Gateway: {:?}", e))
        }
    }
}
