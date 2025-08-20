use crate::helpers::TestApp;

#[tokio::test]
async fn test_verify_token() {
    let app = TestApp::new().await;

    let response = app.verify_token("token").await;

    assert_eq!(response.status(), 200);
}