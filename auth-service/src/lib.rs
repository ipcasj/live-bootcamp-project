use axum::routing::get_service;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod auth {
    tonic::include_proto!("auth");
}
pub mod grpc;
pub mod api_doc;
pub mod utils;
/// Main library file for the auth-service crate.
///
/// - Defines application state, error handling, and main application struct.
/// - Integrates API routes and shared state.
/// - See also: routes, domain, app_state, services modules.
///
/// # Quick Test Commands
/// To verify that all features are set up correctly, use the following commands after starting the server with `cargo run`:
///
/// ```sh
/// # Health check
/// curl -i http://localhost:3000/health
///
/// # OpenAPI JSON
/// curl -s http://localhost:3000/openapi.json | jq .
///
/// # Signup example
/// curl -i -X POST http://localhost:3000/signup \
///   -H "Content-Type: application/json" \
///   -d '{"email":"test@example.com","password":"password123","requires2FA":false}'
///
/// # To test graceful shutdown, press Ctrl+C in the server terminal
/// ```
/// Error response returned by API endpoints.
/// Main application struct for the auth-service.
/// Application builder and runner implementation.
use axum::{response::{IntoResponse, Response}, Json};
use crate::domain::AuthAPIError;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub code: String,
    pub error: String,
    pub trace_id: Option<String>,
}

// use tracing::Span; // Uncomment if needed for future tracing
// use tracing::field::Field;
// use tracing::dispatcher;
// use uuid::Uuid; // Used in add_trace_id, keep if needed

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        use axum::http::StatusCode;
        let (status, error_message) = match &self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthAPIError::MalformedCredentials => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthAPIError::BannedToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthAPIError::UnexpectedError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Unexpected error: {}", e))
            }
        };
        // Get trace_id from tracing span if available
        // No access to request extensions here, so trace_id is not available
        let body = Json(ErrorResponse {
            code: self.code().to_string(),
            error: error_message,
            trace_id: None,
        });
        // axum 0.6 does not implement IntoResponse for (StatusCode, Json<T>), so do it manually
        let mut response = status.into_response();
    // Serialize the inner ErrorResponse, not the Json wrapper
    let inner = body.0;
    *response.body_mut() = axum::body::boxed(axum::body::Full::from(serde_json::to_vec(&inner).unwrap()));
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/json"),
        );
        response
    }
}
// mod domain; // removed duplicate, now public below
mod auth_middleware;
pub mod app_state {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::domain::UserStore;

    pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

    use crate::domain::data_stores::BannedTokenStore;
    #[derive(Clone)]
    pub struct AppState {
        pub user_store: UserStoreType,
        pub banned_token_store: Arc<dyn BannedTokenStore>,
        pub two_fa_code_store: Arc<tokio::sync::RwLock<crate::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore>>,
        pub email_client: Arc<dyn crate::domain::email_client::EmailClient>,
    }

    impl AppState {
        pub fn new(
            user_store: UserStoreType,
            banned_token_store: Arc<dyn BannedTokenStore>,
            two_fa_code_store: Arc<tokio::sync::RwLock<crate::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore>>,
            email_client: Arc<dyn crate::domain::email_client::EmailClient>,
        ) -> Self {
            Self { user_store, banned_token_store, two_fa_code_store, email_client }
        }
    }
}
// pub mod services; // removed duplicate, now public below
use axum::{Router, routing::post};
pub mod domain;
pub mod routes;
pub mod services;
use crate::app_state::AppState;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use http::Method;
use std::error::Error;


// This struct encapsulates our application-related logic.
pub struct Application {
    server: hyper::Server<hyper::server::conn::AddrIncoming, axum::routing::IntoMakeService<Router>>,
    pub address: String,
    shutdown_signal: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl Application {
    pub async fn build(app_state: Arc<AppState>, address: &str, shutdown_signal: Option<tokio::sync::oneshot::Receiver<()>>) -> Result<Self, Box<dyn Error>> {
    use utoipa::OpenApi;
    use axum::routing::get;
    let openapi = crate::api_doc::ApiDoc::openapi();
    let openapi_json = serde_json::to_string(&openapi).unwrap();
    use tower_http::trace::TraceLayer;
    use tower_http::catch_panic::CatchPanicLayer;
    use axum::middleware::from_fn;
    use uuid::Uuid;

        // Middleware to inject a trace_id into each request span
    async fn add_trace_id<B>(req: axum::http::Request<B>, next: axum::middleware::Next<B>) -> axum::response::Response {
            let trace_id = Uuid::new_v4();
            // Attach trace_id to request extensions for later retrieval
            let mut req = req;
            req.extensions_mut().insert(trace_id);
            next.run(req).await
        }

        // --- CORS dynamic origins ---
        let allowed_origins_env = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
        let allowed_origins: Vec<String> = allowed_origins_env
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let cors = if allowed_origins.contains(&"*".to_string()) {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers(Any)
        } else {
            let origins = allowed_origins.iter().map(|o| o.parse().unwrap()).collect::<Vec<_>>();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers(Any)
        };

        let router = Router::new()
            .route("/signup", post(routes::signup::signup))
            .route("/login", post(routes::login::login))
            .route("/logout", post(routes::logout::logout))
            .route("/verify-2fa", post(routes::verify_2fa::verify_2fa))
            .route("/verify-token", post(routes::verify_token::verify_token))
            .route("/refresh-token", post(routes::refresh_token::refresh_token))
            .route("/forgot-password", post(routes::reset_password::forgot_password))
            .route("/reset-password", post(routes::reset_password::reset_password))
            .route("/delete-account", axum::routing::delete(routes::auth::delete_account))
            .route("/account/settings", axum::routing::get(routes::account::get_account_settings))
            .route("/account/settings", axum::routing::patch(routes::account::update_2fa_setting))
            .route("/health", axum::routing::get(routes::signup::health))
            .route("/openapi.json", get(|| async move { openapi_json }))
            .fallback_service(
                get_service(ServeDir::new("assets"))
            )
            .with_state(app_state.clone());
        let router = router
            .layer(cors)
            .layer(from_fn(add_trace_id))
            .layer(CatchPanicLayer::new())
            .layer(TraceLayer::new_for_http());

    let std_listener = std::net::TcpListener::bind(address)?;
    std_listener.set_nonblocking(true)?;
    let address = std_listener.local_addr()?.to_string();
    let listener = tokio::net::TcpListener::from_std(std_listener)?;
    let server = axum::Server::from_tcp(listener.into_std()?)?.serve(router.into_make_service());
    Ok(Application { server, address, shutdown_signal })
    }

    pub async fn run(self) -> Result<(), hyper::Error> {
        println!("listening on {}", &self.address);
    if let Some(shutdown_signal) = self.shutdown_signal {
            self.server.with_graceful_shutdown(async move {
                let _ = shutdown_signal.await;
            }).await
        } else {
            self.server.await
        }
    }
}
// This module is used to define the Application struct and its methods.

/// Creates a PostgreSQL connection pool
/// 
/// # Arguments
/// * `url` - Database connection URL
/// 
/// # Returns
/// * `Result<PgPool, sqlx::Error>` - Connection pool or error
pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    // Create a new PostgreSQL connection pool
    PgPoolOptions::new().max_connections(5).connect(url).await
}

/// Creates a Redis connection pool manager
/// 
/// # Arguments
/// * `redis_hostname` - Redis hostname
/// 
/// # Returns
/// * `Result<bb8::Pool<bb8_redis::RedisConnectionManager>, Box<dyn std::error::Error + Send + Sync>>` - Connection pool or error
pub async fn get_redis_pool(redis_hostname: String) -> Result<bb8::Pool<bb8_redis::RedisConnectionManager>, Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = format!("redis://{}/", redis_hostname);
    let manager = bb8_redis::RedisConnectionManager::new(redis_url)?;
    let pool = bb8::Pool::builder().build(manager).await?;
    Ok(pool)
}