use auth_service::Application;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
    use auth_service::app_state::{AppState, UserStoreType};
    use auth_service::services::hashmap_user_store::HashmapUserStore;
    let user_store: UserStoreType = std::sync::Arc::new(tokio::sync::RwLock::new(HashmapUserStore::default()));
    let app_state = AppState::new(user_store);

        let app = Application::build(app_state, "127.0.0.1:0")
            .await
            .expect("Failed to build application");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();
        Self { address, http_client }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
        .get(&format!("{}/", &self.address))
        .send()
        .await
        .expect("Failed to execute request")
    }

    // Implementation for helper functions for all other routes (signup, login, logout, verify-2fa, and verify-token)
    pub async fn signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
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

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_2fa(&self, code: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(&serde_json::json!({ "code": code }))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token(&self, token: &str) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(&serde_json::json!({ "token": token }))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    }
}