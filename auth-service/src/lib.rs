mod api_doc;
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
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use crate::domain::AuthAPIError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
mod domain;
pub mod app_state {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::domain::UserStore;

    pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;

    #[derive(Clone)]
    pub struct AppState {
        pub user_store: UserStoreType,
    }

    impl AppState {
        pub fn new(user_store: UserStoreType) -> Self {
            Self { user_store }
        }
    }
}
mod routes;
pub mod services;
use axum::{serve::Serve, Router, routing::post};
use crate::app_state::AppState;
use tower_http::services::ServeDir;
use std::error::Error;


// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<tokio::net::TcpListener, Router, Router>,
    pub address: String,
    shutdown_signal: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str, shutdown_signal: Option<tokio::sync::oneshot::Receiver<()>>) -> Result<Self, Box<dyn Error>> {
    use utoipa::OpenApi;
        use axum::routing::get;
        let openapi = crate::api_doc::ApiDoc::openapi();
        let openapi_json = serde_json::to_string(&openapi).unwrap();
        let router = Router::new()
            .route("/signup", post(routes::signup::signup))
            .route("/login", post(routes::auth::dummy_handler))
            .route("/logout", post(routes::auth::dummy_handler))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::auth::dummy_handler))
            .route("/health", axum::routing::get(routes::signup::health))
            .route("/openapi.json", get(|| async move { openapi_json }))
            .fallback_service(ServeDir::new("assets"))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);
    Ok(Application { server, address, shutdown_signal })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
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