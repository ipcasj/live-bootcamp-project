#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    let response = app.verify_2fa("trigger500").await;
    assert_eq!(response.status(), 500);
}
use crate::helpers::TestApp;

#[tokio::test]
async fn test_verify_2fa() {
    let app = TestApp::new().await;

    let response = app.verify_2fa("123456").await;

    assert_eq!(response.status(), 200);
}