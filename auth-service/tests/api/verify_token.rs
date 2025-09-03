#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    let response = app.verify_token("trigger500").await;
    assert_eq!(response.status(), 500);
}
use crate::helpers::TestApp;

#[tokio::test]
async fn test_verify_token() {
    let app = TestApp::new().await;

    let response = app.verify_token("token").await;

    assert_eq!(response.status(), 200);
}