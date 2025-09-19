// use auth_service::{Application, grpc}; // unused
use reqwest::cookie::Jar;
use uuid::Uuid;
// use tonic::transport::Server; // unused
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};
use sqlx::{PgConnection, Connection, Executor};
use sqlx::postgres::PgConnectOptions;
use std::str::FromStr;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::{get_redis_pool, get_postgres_pool};
use auth_service::services::data_stores::redis_banned_token_store::{RedisBannedTokenStore, RedisPool};

// Helper function to delete a test database
async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to terminate active connections to the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

// Helper function to create a test database
async fn create_database(db_name: &str) -> sqlx::Pool<sqlx::Postgres> {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Now we configure the database URL to point to the new database.
    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // We create and return the connection pool for the test database.
    let pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool");

    // Run database migrations on the test database
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}

// Helper function to cleanup Redis test database
async fn cleanup_redis_test_database(redis_pool: &RedisPool, test_db: u32) {
    let mut conn = redis_pool.get().await.expect("Failed to get Redis connection for cleanup");
    let _: () = bb8_redis::redis::cmd("SELECT")
        .arg(test_db)
        .query_async(&mut *conn)
        .await
        .expect("Failed to select test database for cleanup");
    let _: () = bb8_redis::redis::cmd("FLUSHDB")
        .query_async(&mut *conn)
        .await
        .expect("Failed to flush test database");
}

// use auth_service::domain::data_stores::BannedTokenStore; // unused
pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<RedisBannedTokenStore>,
    pub two_fa_code_store: Arc<tokio::sync::RwLock<auth_service::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore>>,
    pub db_name: String,
    pub redis_db: u32,
    pub redis_pool: Arc<RedisPool>,
    pub cleanup_called: bool,
    shutdown_guard: Option<oneshot::Sender<()>>,
    grpc_shutdown_guard: Option<oneshot::Sender<()>>,
}

impl TestApp {
    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }
    pub async fn new() -> Self {
        use auth_service::app_state::{AppState, UserStoreType};
        use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
        use auth_service::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
        use auth_service::routes;
        use axum::{Router, routing::post};
        use tower_http::services::ServeDir;
        use axum::Server;

        // Create a unique database name for this test
        let db_name = Uuid::new_v4().to_string();
        
        // Create and connect to the test database
        let db_pool = create_database(&db_name).await;

        // Set up Redis for test isolation
        let redis_pool = Arc::new(get_redis_pool("localhost".to_string()).await.expect("Failed to create Redis pool"));
        let test_redis_db = 1; // Use database 1 for tests (0 is for production)
        
        let user_store: UserStoreType = Arc::new(RwLock::new(PostgresUserStore::new(db_pool)));
        let banned_token_store = Arc::new(RedisBannedTokenStore::new(redis_pool.clone()));
        let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        use auth_service::services::mock_email_client::MockEmailClient;
        let email_client = Arc::new(MockEmailClient);
        let app_state = Arc::new(AppState::new(user_store.clone(), banned_token_store.clone(), two_fa_code_store.clone(), email_client));

        // Build the router directly for the test
        use utoipa::OpenApi;
        use axum::routing::get;
        let openapi = auth_service::api_doc::ApiDoc::openapi();
        let openapi_json = serde_json::to_string(&openapi).unwrap();
        let router = Router::new()
            .route("/signup", post(routes::signup::signup))
            .route("/login", post(routes::login::login))
            .route("/logout", post(routes::logout::logout))
            .route("/verify-2fa", post(routes::verify_2fa::verify_2fa))
            .route("/verify-token", post(routes::verify_token::verify_token))
            .route("/audit-log", get(|| async { axum::Json(Vec::<serde_json::Value>::new()) }))
            .route("/health", get(routes::signup::health))
            .route("/openapi.json", get(|| async move { openapi_json.clone() }))
            .fallback_service(ServeDir::new("assets"))
            .with_state(app_state.clone());

        // Bind to a random port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        let address = format!("http://{}", listener.local_addr().unwrap());
        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        // Spawn the server for the duration of the test
        let std_listener = listener.into_std().unwrap();
        tokio::spawn(async move {
            Server::from_tcp(std_listener)
                .unwrap()
                .serve(router.into_make_service())
                .await
                .unwrap();
        });

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            db_name,
            redis_db: test_redis_db,
            redis_pool,
            cleanup_called: false,
            shutdown_guard: None,
            grpc_shutdown_guard: None,
        }
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn login(&self, email: &str, password: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn signup(&self, email: &str, password: &str, requires_2fa: bool) -> reqwest::Response {
        let body = serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": requires_2fa
        });
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }


    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    }

    /// Clean up the test database and Redis test data
    pub async fn cleanup(&mut self) {
        if !self.cleanup_called {
            delete_database(&self.db_name).await;
            cleanup_redis_test_database(&self.redis_pool, self.redis_db).await;
            self.cleanup_called = true;
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_guard.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.grpc_shutdown_guard.take() {
            let _ = tx.send(());
        }
        
        // Note: We can't call async functions in Drop, so we enforce cleanup
        // through other means. Users should call cleanup() manually.
        if !self.cleanup_called {
            eprintln!("Warning: TestApp dropped without calling cleanup(). Database '{}' and Redis test data may still exist.", self.db_name);
        }
    }
}
// END OF FILE