use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod api;
mod dashboard;
mod lib;
mod metrics;

/// Explicit allowlist of origins permitted to make cross-origin requests.
/// No wildcard: browsers only receive CORS headers for these origins.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://app.example.com",
    "https://admin.example.com",
    "http://localhost:3000",
    "http://localhost:5173",
];

fn cors_layer() -> CorsLayer {
    let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(api::routes::router())
        .merge(dashboard::router())
        .layer(cors_layer());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
