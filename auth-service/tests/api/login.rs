#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    let response = app.login("trigger500@example.com", "password").await;
    assert_eq!(response.status(), 500);
}
use crate::helpers::TestApp;

#[tokio::test]
async fn test_login() {
    let app = TestApp::new().await;

    let response = app.login("test@example.com", "password").await;

    assert_eq!(response.status(), 200);
}