#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    let response = app.login("trigger500@example.com", "password").await;
    assert_eq!(response.status(), 500);
}

use crate::helpers::TestApp;

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