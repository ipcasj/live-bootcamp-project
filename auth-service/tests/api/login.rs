use crate::helpers::TestApp;

#[tokio::test]
async fn test_login() {
    let app = TestApp::new().await;

    let response = app.login("test@example.com", "password").await;

    assert_eq!(response.status(), 200);
}