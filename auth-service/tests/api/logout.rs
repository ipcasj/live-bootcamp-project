use crate::helpers::TestApp;

#[tokio::test]
async fn test_logout() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status(), 200);
}