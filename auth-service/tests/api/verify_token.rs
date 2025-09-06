use crate::helpers::TestApp;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    // Missing token field
    let body = serde_json::json!({});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 422);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;
    let body = serde_json::json!({ "token": "invalid.jwt.token" });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), 401);
    let err: ErrorResponse = response.json().await.expect("Invalid JSON");
    assert_eq!(err.error, "Invalid or expired token");
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    // Register and login to get a valid token
    let email = TestApp::get_random_email();
    let signup_body = serde_json::json!({
        "email": email,
        "password": "password123",
        "requires2FA": false
    });
    let _ = app.signup(&signup_body).await;
    let login_body = serde_json::json!({
        "email": email,
        "password": "password123"
    });
    let login_response = app.post_login(&login_body).await;
    let cookie = login_response
        .headers()
        .get("set-cookie")
        .expect("No set-cookie header")
        .to_str()
        .unwrap();
    let token = cookie
        .split(';')
        .find(|s| s.trim_start().starts_with("jwt="))
        .unwrap()
        .trim_start_matches("jwt=");

    let verify_body = serde_json::json!({ "token": token });
    let response = app.post_verify_token(&verify_body).await;
    assert_eq!(response.status(), 200);
}