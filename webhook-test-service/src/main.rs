use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, debug};

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    #[serde(flatten)]
    data: Value,
}

#[derive(Debug, Serialize)]
struct WebhookResponse {
    status: String,
    received_at: String,
    payload_size: usize,
    message: String,
}

async fn health_check() -> &'static str {
    "Webhook Test Service - OK"
}

async fn receive_webhook(Json(payload): Json<WebhookPayload>) -> Result<ResponseJson<WebhookResponse>, StatusCode> {
    let received_at = chrono::Utc::now().to_rfc3339();
    let payload_json = serde_json::to_string_pretty(&payload.data).unwrap_or_default();
    let payload_size = payload_json.len();
    
    info!(
        received_at = %received_at,
        payload_size = payload_size,
        "📨 Webhook received"
    );
    
    // Pretty print the payload for debugging
    debug!("📋 Webhook payload:\n{}", payload_json);
    
    // Extract some interesting fields if they exist
    if let Some(logs) = payload.data.get("logs") {
        if let Some(logs_array) = logs.as_array() {
            info!(
                log_count = logs_array.len(),
                "📊 Processing log batch"
            );
            
            for (i, log_entry) in logs_array.iter().enumerate() {
                if let Some(level) = log_entry.get("level") {
                    if let Some(message) = log_entry.get("message") {
                        info!(
                            log_index = i,
                            level = level.as_str().unwrap_or("unknown"),
                            message = message.as_str().unwrap_or(""),
                            "🔍 Log entry"
                        );
                    }
                }
            }
        }
    }
    
    // Check if it's a Loki-style payload
    if let Some(streams) = payload.data.get("streams") {
        if let Some(streams_array) = streams.as_array() {
            info!(
                stream_count = streams_array.len(),
                "📈 Processing Loki streams"
            );
        }
    }
    
    let response = WebhookResponse {
        status: "success".to_string(),
        received_at,
        payload_size,
        message: format!("Successfully received and processed webhook payload"),
    };
    
    Ok(ResponseJson(response))
}

async fn simulate_failure(Json(_payload): Json<WebhookPayload>) -> StatusCode {
    warn!("💥 Simulating webhook failure for testing retry logic");
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn simulate_slow_response(Json(payload): Json<WebhookPayload>) -> ResponseJson<WebhookResponse> {
    info!("🐌 Simulating slow webhook response for testing timeouts");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    let response = WebhookResponse {
        status: "slow_success".to_string(),
        received_at: chrono::Utc::now().to_rfc3339(),
        payload_size: serde_json::to_string(&payload.data).unwrap_or_default().len(),
        message: "Slow webhook completed".to_string(),
    };
    
    ResponseJson(response)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/webhook", post(receive_webhook))
        .route("/webhook/fail", post(simulate_failure))
        .route("/webhook/slow", post(simulate_slow_response))
        .layer(CorsLayer::permissive());

    // Run on port 8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(&addr).await.unwrap();
    
    info!("🚀 Webhook Test Service listening on http://{}", addr);
    info!("📍 Available endpoints:");
    info!("  GET  /health - Health check");
    info!("  POST /webhook - Main webhook endpoint");
    info!("  POST /webhook/fail - Simulate failures");
    info!("  POST /webhook/slow - Simulate slow responses");
    
    axum::serve(listener, app).await.unwrap();
}