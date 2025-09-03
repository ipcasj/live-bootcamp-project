#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    // The logout helper does not take a token, so we need to call the endpoint manually
    let response = app.http_client
        .post(&format!("{}/logout", &app.address))
        .json(&serde_json::json!({ "token": "trigger500" }))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), 500);
}
use crate::helpers::TestApp;

#[tokio::test]
async fn test_logout() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status(), 200);
}