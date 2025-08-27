//! Binary entry point for the auth-service.
//!
//! - Initializes logging and application state.
//! - Starts the Axum server.
use tracing_subscriber;

use auth_service::Application;
use auth_service::app_state::{AppState, UserStoreType};
use auth_service::services::hashmap_user_store::HashmapUserStore;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();
    let user_store: UserStoreType = std::sync::Arc::new(tokio::sync::RwLock::new(HashmapUserStore::default()));
    let app_state = AppState::new(user_store);

    // Set up graceful shutdown signal (Ctrl+C)
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let app = Application::build(app_state, "0.0.0.0:3000", Some(shutdown_rx))
        .await
        .expect("Failed to build app");

    // Spawn a task to listen for Ctrl+C
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_tx.send(());
        }
    });

    app.run().await.expect("Failed to run app");
}