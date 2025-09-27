//! Simple webhook receiver service for testing external logging to HTTP endpoints
//! This service receives HTTP requests and logs them to demonstrate external logging.

use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct WebhookResponse {
    status: String,
    message: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct LogsResponse {
    status: String,
    message: String,
    count: usize,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    timestamp: DateTime<Utc>,
}

/// Receive and log webhook data
async fn receive_webhook(request: Request) -> Result<Json<WebhookResponse>, StatusCode> {
    let timestamp = Utc::now();
    
    // Extract request information
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let headers: HashMap<String, String> = request
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    
    // Get the body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    let body_str = String::from_utf8_lossy(&body_bytes);
    let body_json: Value = serde_json::from_slice(&body_bytes)
        .unwrap_or_else(|_| Value::String(body_str.to_string()));

    // Log the complete webhook data
    info!(
        method = %method,
        uri = %uri,
        headers = ?headers,
        body = %serde_json::to_string_pretty(&body_json).unwrap_or_else(|_| body_str.to_string()),
        "Webhook received"
    );

    let response = WebhookResponse {
        status: "success".to_string(),
        message: "Webhook received and logged successfully".to_string(),
        timestamp,
    };

    Ok(Json(response))
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "webhook-receiver".to_string(),
        timestamp: Utc::now(),
    })
}

/// Get logs endpoint (placeholder - in real scenario you'd read from log files)
async fn get_logs() -> Json<LogsResponse> {
    info!("Logs endpoint accessed");
    
    Json(LogsResponse {
        status: "success".to_string(),
        message: "Check the console/log files for webhook logs".to_string(),
        count: 0,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(true)
        .init();

    info!("Starting webhook receiver service...");

    // Build the application with routes
    let app = Router::new()
        .route("/webhook", post(receive_webhook))
        .route("/health", get(health_check))
        .route("/logs", get(get_logs))
        .layer(CorsLayer::permissive()); // Allow CORS for testing

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("Webhook receiver listening on http://0.0.0.0:8080");
    info!("Endpoints:");
    info!("  POST /webhook - Receive webhook data");
    info!("  GET  /health  - Health check");
    info!("  GET  /logs    - Get logs info");

    axum::serve(listener, app).await?;

    Ok(())
}