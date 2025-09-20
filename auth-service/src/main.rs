//! Binary entry point for the auth-service.
//!
//! - Initializes logging and application state.
//! - Starts the Axum server with modern configuration management.
use tracing_subscriber;
use auth_service::{Application, get_postgres_pool, get_redis_pool};
use auth_service::app_state::{AppState, UserStoreType};
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::grpc;
use auth_service::config::AppConfig;
use tonic::transport::Server;
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt::init();

    // Load configuration from multiple sources
    let config = AppConfig::load().expect("Failed to load configuration");
    
    tracing::info!("Starting auth-service with configuration: environment={}", config.environment);

    // Configure PostgreSQL and Redis connections with config
    let pg_pool = configure_postgresql(&config).await;
    let redis_pool = configure_redis(&config).await;

    let user_store: UserStoreType = Arc::new(tokio::sync::RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RedisBannedTokenStore::new(Arc::new(redis_pool.clone())));
    let two_fa_code_store = auth_service::services::two_fa_code_store_factory::redis_two_fa_code_store(Arc::new(redis_pool));
    use auth_service::services::mock_email_client::MockEmailClient;
    let email_client = Arc::new(MockEmailClient);
    let config_arc = Arc::new(config);
    let app_state = Arc::new(AppState::new(user_store, banned_token_store, two_fa_code_store, email_client, config_arc.clone()));

    // Set up graceful shutdown signal (Ctrl+C)
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server_address = config_arc.server_address();
    let app = Application::build(app_state.clone(), &server_address, Some(shutdown_rx))
        .await
        .expect("Failed to build app");

    // gRPC server address
    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_service = grpc::grpc_service(app_state.clone());

    tracing::info!("Starting servers - REST: {}, gRPC: {}", server_address, grpc_addr);

    // Spawn a task to listen for Ctrl+C
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("Shutdown signal received");
            let _ = shutdown_tx.send(());
        }
    });

    // Run both REST and gRPC servers in parallel
    tokio::select! {
        res = app.run() => {
            if let Err(e) = res {
                tracing::error!("REST server error: {}", e);
            }
        }
        res = Server::builder().add_service(grpc_service).serve(grpc_addr) => {
            if let Err(e) = res {
                tracing::error!("gRPC server error: {}", e);
            }
        }
    }
}

async fn configure_postgresql(config: &AppConfig) -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&config.database.url)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database! 
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

async fn configure_redis(config: &AppConfig) -> bb8::Pool<bb8_redis::RedisConnectionManager> {
    // Create a new Redis connection pool using configuration
    get_redis_pool(config.redis.host.clone())
        .await
        .expect("Failed to create Redis connection pool!")
}