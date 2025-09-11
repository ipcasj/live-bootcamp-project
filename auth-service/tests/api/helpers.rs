// use auth_service::{Application, grpc}; // unused
use reqwest::cookie::Jar;
use uuid::Uuid;
// use tonic::transport::Server; // unused
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

// use auth_service::domain::data_stores::BannedTokenStore; // unused
use auth_service::services::hashset_banned_token_store::HashsetBannedTokenStore;
pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<HashsetBannedTokenStore>,
    pub two_fa_code_store: Arc<tokio::sync::RwLock<auth_service::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore>>,
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
        use auth_service::services::hashmap_user_store::HashmapUserStore;
        use auth_service::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
        use auth_service::routes;
        use axum::{Router, routing::post};
        use tower_http::services::ServeDir;
        use axum::Server;

        let user_store: UserStoreType = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_token_store = Arc::new(HashsetBannedTokenStore::default());
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
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_guard.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.grpc_shutdown_guard.take() {
            let _ = tx.send(());
        }
    }
}
// END OF FILE