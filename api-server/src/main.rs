use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod middleware;
mod models;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configure CORS (allow localhost development)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/convert", post(handlers::convert))
        .route("/api/v1/export", post(handlers::export))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Bind to localhost:8080
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
