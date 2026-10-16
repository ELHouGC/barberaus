mod config;
mod cqrs;
mod domain;
mod infrastructure;
mod state;
mod transport;
use resend_rs::Method;
use transport::http;
mod application;

use axum::{Router, http::header, routing::get};
use deadpool_redis::{Config as RedisConfig, Runtime};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, CorsLayer};

use config::Config;
use state::AppState;

async fn health_check() -> &'static str {
    "BarberOS Platform Identity Engine Active"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let config = Config::from_env();

    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    let redis_cfg = RedisConfig::from_url(redis_url);
    let redis_pool = redis_cfg.create_pool(Some(Runtime::Tokio1))?;

    let app_state = AppState::new(db_pool, redis_pool, config);

    let cors = CorsLayer::new()
        // 1. Explicitly list frontend origins (Do NOT use Any / '*')
        .allow_origin(AllowOrigin::mirror_request())
        // 2. Allow necessary HTTP methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // 3. Allow standard authentication headers
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::HeaderName::from_static("x-refresh-token"),
        ])
        // 4. Crucial: Allow HttpOnly cookies across origins
        .allow_credentials(true);

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(http::router::auth::create_router(app_state.clone()))
        .merge(http::router::client_profile::create_router(
            app_state.clone(),
        ))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
