use crate::helpers::TestApp;

#[tokio::test]
async fn test_signup() {
    let app = TestApp::new().await;

    let response = app.signup("test@example.com", "password").await;

    assert_eq!(response.status(), 200);
}