    pub async fn delete_account(&self, email: &str) -> reqwest::Response {
        self.http_client
            .delete(&format!("{}/delete-account", &self.address))
            .header("x-user-email", email)
            .send()
            .await
            .expect("Failed to execute request")
    }

use auth_service::{Application, grpc};
use uuid::Uuid;
use tonic::transport::Server;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};


pub struct TestApp {
    pub address: String,
    pub grpc_addr: String,
    pub http_client: reqwest::Client,
    shutdown_guard: Option<oneshot::Sender<()>>,
    grpc_shutdown_guard: Option<oneshot::Sender<()>>,
}


impl TestApp {
    pub async fn new() -> Self {
        use auth_service::app_state::{AppState, UserStoreType};
        use auth_service::services::hashmap_user_store::HashmapUserStore;
        let user_store: UserStoreType = Arc::new(RwLock::new(HashmapUserStore::default()));
    let app_state = Arc::new(AppState::new(user_store.clone()));

        // REST server
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = Application::build((*app_state).clone(), "127.0.0.1:0", Some(shutdown_rx))
            .await
            .expect("Failed to build application");
        let address = format!("http://{}", app.address.clone());
        let http_client = reqwest::Client::new();
        tokio::spawn(app.run());

        // gRPC server
        let (grpc_shutdown_tx, grpc_shutdown_rx) = oneshot::channel();
        let grpc_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind gRPC");
        let grpc_addr = grpc_listener.local_addr().unwrap();
        let grpc_service = grpc::grpc_service(app_state.clone());
        let grpc_shutdown = async move {
            let _ = grpc_shutdown_rx.await;
        };
        tokio::spawn(async move {
            Server::builder()
                .add_service(grpc_service)
                .serve_with_incoming_shutdown(
                    tokio_stream::wrappers::TcpListenerStream::new(grpc_listener),
                    grpc_shutdown,
                )
                .await
                .expect("gRPC server failed");
        });

        Self {
            address,
            grpc_addr: format!("http://{}", grpc_addr),
            http_client,
            shutdown_guard: Some(shutdown_tx),
            grpc_shutdown_guard: Some(grpc_shutdown_tx),
        }
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