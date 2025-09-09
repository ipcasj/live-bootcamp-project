#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    use crate::helpers::TestApp;
    use auth_service::domain::Email;
    use auth_service::domain::data_stores::TwoFACodeStore;
    use auth_service::routes::login::TwoFactorAuthResponse;

    let app = TestApp::new().await;
    let email = TestApp::get_random_email();
    let password = "password123";

    // Register user with 2FA enabled
    let _ = app.signup(&email, password, true).await;

    // Login
    let response = app.login(&email, password).await;
    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    // Assert that login_attempt_id is stored in the 2FA code store
    let (stored_login_attempt_id, _) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    assert_eq!(stored_login_attempt_id.as_ref(), json_body.login_attempt_id);
}
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
    let email = TestApp::get_random_email();
    let password = "password123";
    let response = app.signup(&email, password, false).await;
    assert_eq!(response.status().as_u16(), 201);
    let login_body = serde_json::json!({
        "email": email,
        "password": password,
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());
}