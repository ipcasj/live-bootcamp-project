// Only use generated types for gRPC, avoid REST type name conflicts
mod default_impls;
use async_trait::async_trait;

use crate::app_state::AppState;
use crate::domain::{Email, Password, User};


use std::sync::Arc;

pub struct MyAuthService {
    pub state: Arc<AppState>,
}


#[async_trait]
impl crate::auth::auth_service_server::AuthService for MyAuthService {
    async fn signup(
        &self,
        request: tonic::Request<crate::auth::SignupRequest>,
    ) -> Result<tonic::Response<crate::auth::SignupResponse>, tonic::Status> {
        let req = request.into_inner();
        let trace_id = uuid::Uuid::new_v4();
        // Validate email and password
        let email = Email::parse(&req.email).map_err(|_| {
            tracing::error!(%trace_id, "Invalid email");
            tonic::Status::invalid_argument("Invalid email")
        })?;
        let password = Password::parse(&req.password).map_err(|_| {
            tracing::error!(%trace_id, "Invalid password");
            tonic::Status::invalid_argument("Invalid password")
        })?;
        let user = User::new(email.clone(), password, req.requires2_fa);
        let mut user_store = self.state.user_store.write().await;
        if user_store.get_user(&email).await.is_ok() {
            tracing::error!(%trace_id, "User already exists");
            return Err(tonic::Status::already_exists("User already exists"));
        }
        if let Err(e) = user_store.add_user(user).await {
            tracing::error!(%trace_id, error = ?e, "Internal error adding user");
            return Err(tonic::Status::internal(format!("Internal server error (trace_id: {trace_id})")));
        }
        Ok(tonic::Response::new(crate::auth::SignupResponse {
            message: "User created successfully!".to_string(),
        }))
    }

    async fn login(
        &self,
        request: tonic::Request<crate::auth::LoginRequest>,
    ) -> Result<tonic::Response<crate::auth::LoginResponse>, tonic::Status> {
        let req = request.into_inner();
        let trace_id = uuid::Uuid::new_v4();
        let email = Email::parse(&req.email).map_err(|_| {
            tracing::error!(%trace_id, "Invalid email");
            tonic::Status::invalid_argument("Invalid email")
        })?;
    let password_str = &req.password;
    // Optionally: validate password length/format here if needed
    let user_store = self.state.user_store.read().await;
    match user_store.validate_user(&email, password_str).await {
            Ok(()) => Ok(tonic::Response::new(crate::auth::LoginResponse {
                message: "Login successful".to_string(),
                token: "dummy-token".to_string(),
            })),
            Err(e) => {
                tracing::error!(%trace_id, error = ?e, "Invalid credentials or internal error");
                Err(tonic::Status::unauthenticated(format!("Invalid credentials (trace_id: {trace_id})")))
            }
        }
    }
}

pub fn grpc_service(state: Arc<AppState>) -> crate::auth::auth_service_server::AuthServiceServer<MyAuthService> {
    crate::auth::auth_service_server::AuthServiceServer::new(MyAuthService { state })
}
