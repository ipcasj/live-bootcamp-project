//! gRPC regression tests for AuthService


use crate::helpers::TestApp;
use auth_service::generated::auth_service_client::AuthServiceClient;
use auth_service::generated::{SignupRequest, LoginRequest};

#[tokio::test]
async fn grpc_signup_and_login() {
    let _ = dotenvy::dotenv();
    let app = TestApp::new().await;
    let mut client = AuthServiceClient::connect(app.grpc_addr.clone())
        .await
        .expect("Failed to connect to gRPC server");

    let signup_request = tonic::Request::new(SignupRequest {
        email: "grpc_test@example.com".to_string(),
        password: "password".to_string(),
        requires2fa: false,
    });
    let signup_response = client.signup(signup_request).await.expect("Signup failed");
    assert_eq!(signup_response.get_ref().message, "Signup successful");

    let login_request = tonic::Request::new(LoginRequest {
        email: "grpc_test@example.com".to_string(),
        password: "password".to_string(),
    });
    let login_response = client.login(login_request).await.expect("Login failed");
    assert_eq!(login_response.get_ref().message, "Login successful");
    assert!(!login_response.get_ref().token.is_empty());
}
