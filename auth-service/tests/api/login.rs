use auth_service::utils::constants::JWT_COOKIE_NAME;
#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    let response = app.login("trigger500@example.com", "password").await;
    assert_eq!(response.status(), 500);
}

use crate::helpers::TestApp;

// use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse}; // unused

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    // Arrange
    let app = TestApp::new().await;
    // Malformed: missing password
    let body = serde_json::json!({ "email": TestApp::get_random_email() });
    let response = app.post_login(&body).await;
    // Assert
    assert_eq!(response.status(), 422);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;
    let random_email = TestApp::get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    let response = app.signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);
    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
}