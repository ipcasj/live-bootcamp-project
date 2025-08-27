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
        use crate::services::hashmap_user_store::HashmapUserStore;
    // If the file is actually named differently (e.g., user_store.rs), update the import like this:
    // use crate::services::user_store::HashmapUserStore;

    // Using a type alias to improve readability!
    pub type UserStoreType = Arc<RwLock<HashmapUserStore>>;

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
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .route("/signup", post(routes::signup::signup))
            .route("/login", post(routes::auth::dummy_handler))
            .route("/logout", post(routes::auth::dummy_handler))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/verify-token", post(routes::auth::dummy_handler))
            .fallback_service(ServeDir::new("assets"))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
// This module is used to define the Application struct and its methods.