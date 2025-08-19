use axum::{serve::Serve, Router, routing::post, http::StatusCode, response::IntoResponse};
use tower_http::services::ServeDir;
use std::error::Error;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<tokio::net::TcpListener, Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {

        let router = Router::new()
            .route("/signup", post(dummy_handler))
            .route("/login", post(dummy_handler))
            .route("/logout", post(dummy_handler))
            .route("/verify-2fa", post(dummy_handler))
            .route("/verify-token", post(dummy_handler))
            .fallback_service(ServeDir::new("assets"));
        
    // Dummy handler that always returns 200 OK
    async fn dummy_handler() -> impl IntoResponse {
        StatusCode::OK
    }

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
// This module is used to define the Application struct and its methods.