use crate::helpers::TestApp;

#[tokio::test]
async fn test_verify_2fa() {
    let app = TestApp::new().await;

    let response = app.verify_2fa("123456").await;

    assert_eq!(response.status(), 200);
}