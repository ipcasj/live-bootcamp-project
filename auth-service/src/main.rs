//! Binary entry point for the auth-service.
//!
//! - Initializes logging and application state.
//! - Starts the Axum server.
use tracing_subscriber;
use auth_service::Application;
use auth_service::app_state::{AppState, UserStoreType};
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::grpc;
use auth_service::services::two_fa_code_store_factory::default_two_fa_code_store;
use tonic::transport::Server;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();
    use auth_service::services::hashset_banned_token_store::HashsetBannedTokenStore;
    let user_store: UserStoreType = std::sync::Arc::new(tokio::sync::RwLock::new(HashmapUserStore::default()));
    let banned_token_store = std::sync::Arc::new(HashsetBannedTokenStore::default());
    let two_fa_code_store = default_two_fa_code_store();
    use auth_service::services::mock_email_client::MockEmailClient;
    let email_client = std::sync::Arc::new(MockEmailClient);
    let app_state = std::sync::Arc::new(AppState::new(user_store, banned_token_store, two_fa_code_store, email_client));

    // Set up graceful shutdown signal (Ctrl+C)
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let app = Application::build(app_state.clone(), "0.0.0.0:3000", Some(shutdown_rx))
        .await
        .expect("Failed to build app");

    // gRPC server address
    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_service = grpc::grpc_service(app_state.clone());

    // Spawn a task to listen for Ctrl+C
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_tx.send(());
        }
    });

    // Run both REST and gRPC servers in parallel
    tokio::select! {
        res = app.run() => {
            if let Err(e) = res {
                eprintln!("REST server error: {}", e);
            }
        }
        res = Server::builder().add_service(grpc_service).serve(grpc_addr) => {
            if let Err(e) = res {
                eprintln!("gRPC server error: {}", e);
            }
        }
    }
}