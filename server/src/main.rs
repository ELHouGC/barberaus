mod config;
mod error;
mod auth;
mod domain;
mod projections;
mod transport;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // CORS configuration for local Vue 3 dev server
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/ws", get(ws_handler))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("BarberOS Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// REST Health Check Endpoint
async fn health_check() -> Json<SystemStatus> {
    Json(SystemStatus {
        status: "online".to_string(),
        service: "BarberOS Core API".to_string(),
        version: "0.1.0".to_string(),
    })
}

// WebSocket Upgrade Handler (Axum 0.8 compatible signature)
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("Client connected via WebSocket");

    // Send welcome heartbeat event to connected client
    let welcome_msg = serde_json::json!({
        "event": "CONNECTED",
        "payload": { "message": "Established socket handshake with BarberOS Engine" }
    });

    if socket
        .send(Message::Text(welcome_msg.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            tracing::info!("Received WS Message: {}", text);

            let ack = serde_json::json!({
                "event": "ACK",
                "payload": { "received": text.as_str() }
            });

            if socket
                .send(Message::Text(ack.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}