use crate::helpers::TestApp;

#[tokio::test]
async fn health_returns_200() {
    let app = TestApp::new().await;
    let response = app.http_client.get(format!("{}/health", app.address)).send().await.expect("Failed to execute request");
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn openapi_json_returns_200_and_valid_json() {
    let app = TestApp::new().await;
    let response = app.http_client.get(format!("{}/openapi.json", app.address)).send().await.expect("Failed to execute request");
    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await.expect("Response was not valid JSON");
    // Check that the OpenAPI version field exists
    assert!(json.get("openapi").is_some(), "Missing 'openapi' field in OpenAPI spec");
}
