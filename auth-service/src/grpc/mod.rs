pub mod generated {
    tonic::include_proto!("auth");
}

use generated::auth_service_server::{AuthService, AuthServiceServer};
use generated::{SignupRequest, SignupResponse, LoginRequest, LoginResponse};
use tonic::{Request, Response, Status};
use async_trait::async_trait;

use crate::app_state::AppState;
use crate::domain::{Email, Password, User};


use std::sync::Arc;

pub struct MyAuthService {
    pub state: Arc<AppState>,
}


#[async_trait]
impl AuthService for MyAuthService {
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<SignupResponse>, Status> {
        let req = request.into_inner();
        // Validate email and password
        let email = Email::parse(&req.email).map_err(|_| Status::invalid_argument("Invalid email"))?;
        let password = Password::parse(&req.password).map_err(|_| Status::invalid_argument("Invalid password"))?;
        let user = User::new(email.clone(), password, req.requires2_fa);
        let mut user_store = self.state.user_store.write().await;
        if user_store.get_user(&email).await.is_ok() {
            return Err(Status::already_exists("User already exists"));
        }
        if let Err(_) = user_store.add_user(user).await {
            return Err(Status::internal("Failed to add user"));
        }
        Ok(Response::new(SignupResponse {
            message: "User created successfully!".to_string(),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let email = Email::parse(&req.email).map_err(|_| Status::invalid_argument("Invalid email"))?;
        let password = Password::parse(&req.password).map_err(|_| Status::invalid_argument("Invalid password"))?;
        let user_store = self.state.user_store.read().await;
        match user_store.validate_user(&email, &password).await {
            Ok(()) => Ok(Response::new(LoginResponse {
                message: "Login successful".to_string(),
                token: "dummy-token".to_string(),
            })),
            Err(_) => Err(Status::unauthenticated("Invalid credentials")),
        }
    }
}

pub fn grpc_service(state: Arc<AppState>) -> AuthServiceServer<MyAuthService> {
    AuthServiceServer::new(MyAuthService { state })
}
