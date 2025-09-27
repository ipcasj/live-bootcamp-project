use std::env;
use color_eyre::eyre::Result;
use tracing::{info, warn, debug, error};
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

// Import external logging modules
mod external_logging;
mod file_logging;

use external_logging::{ExternalLoggingConfig, ExternalLoggingLayer};
use file_logging::{FileLoggingConfig, FileLoggingManager};

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tower_http::services::ServeDir;
use axum::routing::get_service;
use std::net::TcpListener;

/// Initialize tracing for the app service with external logging support
fn init_tracing() -> Result<()> {
    // Create environment filter with sensible defaults
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("app_service=info,info"))
        .expect("Failed to create env filter");

    // Check if we should use JSON formatting (production)
    let use_json = env::var("LOG_JSON_FORMAT")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    // Initialize external logging configuration
    let external_config = ExternalLoggingConfig::from_env();
    
    // Initialize enhanced file logging if enabled
    let file_config = FileLoggingConfig::default();
    if file_config.enabled {
        let _file_manager = FileLoggingManager::new(file_config.clone())?;
        
        info!(
            log_dir = ?file_config.log_dir,
            rotation_policy = %file_config.rotation_policy,
            max_files = file_config.max_files,
            "📁 Enhanced file logging enabled for app-service"
        );
    }

    if external_config.enabled {
        info!(
            service_type = %external_config.service_type,
            endpoint = %external_config.endpoint,
            batch_size = external_config.batch_size,
            "🌐 External logging enabled for app-service"
        );
    }
    
    let registry = tracing_subscriber::registry()
        .with(filter);

    // Configure console output format based on external logging
    if external_config.enabled {
        match ExternalLoggingLayer::new(external_config.clone()) {
            Ok(external_layer) => {
                if use_json {
                    registry
                        .with(external_layer)
                        .with(fmt::layer()
                            .json()
                            .with_target(true)
                            .with_thread_ids(true))
                        .init();
                } else {
                    registry
                        .with(external_layer)
                        .with(fmt::layer()
                            .pretty()
                            .with_target(false))
                        .init();
                }
            },
            Err(e) => {
                tracing::warn!("Failed to initialize external logging layer, continuing without it: {}", e);
                if use_json {
                    registry
                        .with(fmt::layer()
                            .json()
                            .with_target(true)
                            .with_thread_ids(true))
                        .init();
                } else {
                    registry
                        .with(fmt::layer()
                            .pretty()
                            .with_target(false))
                        .init();
                }
            }
        }
    } else {
        if use_json {
            registry
                .with(fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true))
                .init();
        } else {
            registry
                .with(fmt::layer()
                    .compact()
                    .with_target(false))
                .init();
        }
    }

    info!(
        version = env!("CARGO_PKG_VERSION"),
        service = "app-service",
        external_logging_enabled = external_config.enabled,
        file_logging_enabled = file_config.enabled,
        "🚀 Tracing initialized for app service with enhanced logging"
    );

    // Test log for external logging verification
    info!("🔍 Testing external logging integration - this message should appear in webhook");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling and tracing
    color_eyre::install()?;
    init_tracing()?;

    info!("🌟 Starting app service...");

    let app = Router::new()
    .nest_service("/assets", get_service(ServeDir::new("assets")))
        .route("/", get(root))
        .route("/protected", get(protected));

    let listener = TcpListener::bind("0.0.0.0:8000")?;
    let local_addr = listener.local_addr()?;
    
    info!(
        address = %local_addr,
        port = local_addr.port(),
        "📡 App service listening and ready to serve requests"
    );

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    info!("👋 App service shutting down gracefully");
    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

async fn root() -> impl IntoResponse {
    debug!("📄 Serving root page request");
    
    let mut address = env::var("AUTH_SERVICE_IP").unwrap_or("localhost".to_owned());
    if address.is_empty() {
        debug!("AUTH_SERVICE_IP is empty, using localhost");
        address = "localhost".to_owned();
    }
    
    let login_link = format!("http://{}:3000", address);
    let logout_link = format!("http://{}:3000/logout", address);

    info!(
        auth_service_ip = %address,
        login_link = %login_link,
        logout_link = %logout_link,
        "🔗 Generated authentication links for root page"
    );

    let template = IndexTemplate {
        login_link,
        logout_link,
    };
    
    match template.render() {
        Ok(html) => {
            debug!("✅ Successfully rendered root template");
            Html(html)
        }
        Err(e) => {
            error!(error = %e, "❌ Failed to render root template");
            Html(format!("<h1>Error rendering template: {}</h1>", e))
        }
    }
}

async fn protected(jar: CookieJar) -> impl IntoResponse {
    debug!("🔒 Processing protected route request");
    
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => {
            debug!(cookie_name = "jwt", "🍪 Found JWT cookie in request");
            cookie
        }
        None => {
            warn!("⚠️  No JWT cookie found in protected route request");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    debug!("🌐 Building API client for token verification");
    let api_client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(e) => {
            error!(error = %e, "❌ Failed to build HTTP client");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let verify_token_body = serde_json::json!({
        "token": &jwt_cookie.value(),
    });

    let auth_hostname = env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("0.0.0.0".to_owned());
    let url = format!("http://{}:3000/verify-token", auth_hostname);

    info!(
        auth_hostname = %auth_hostname,
        verify_url = %url,
        "🔍 Sending token verification request to auth service"
    );

    let response = match api_client.post(&url).json(&verify_token_body).send().await {
        Ok(response) => {
            debug!(
                status = %response.status(),
                "📨 Received response from auth service"
            );
            response
        }
        Err(e) => {
            error!(
                error = %e,
                auth_hostname = %auth_hostname,
                url = %url,
                "❌ Failed to send verification request to auth service"
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
            warn!("🚫 Token verification failed - UNAUTHORIZED");
            StatusCode::UNAUTHORIZED.into_response()
        }
        reqwest::StatusCode::BAD_REQUEST => {
            warn!("🚫 Token verification failed - BAD_REQUEST");
            StatusCode::UNAUTHORIZED.into_response()
        }
        reqwest::StatusCode::OK => {
            info!("✅ Token verification successful - serving protected content");
            Json(ProtectedRouteResponse {
                img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
            })
            .into_response()
        }
        status => {
            error!(
                status = %status,
                "❌ Unexpected response status from auth service"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}
