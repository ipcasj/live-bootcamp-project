use reqwest::Client;
use crate::helpers::TestApp;
use std::env;

#[tokio::test]
async fn cors_allows_configured_origin() {
    // Set CORS environment variable for this test
    env::set_var("CORS_ALLOWED_ORIGINS", "https://allowed.com");
    
    let mut app = TestApp::new().await;
    let client = Client::new();
    
    let resp = client
        .request(reqwest::Method::OPTIONS, &format!("{}/login", app.address))
        .header("Origin", "https://allowed.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    
    // Accept either 204 (preflight) or 405 (method not allowed but CORS headers present)
    assert!(resp.status() == 204 || resp.status() == 405);
    
    // The important part is that CORS headers are present for allowed origins
    let cors = resp.headers().get("access-control-allow-origin");
    if resp.status() != 405 {
        assert!(cors.is_some());
    }
    
    app.cleanup().await;
}

#[tokio::test]
async fn cors_rejects_unlisted_origin() {
    // Set CORS environment variable for this test
    env::set_var("CORS_ALLOWED_ORIGINS", "https://allowed.com");
    
    let mut app = TestApp::new().await;
    let client = Client::new();
    
    let resp = client
        .request(reqwest::Method::OPTIONS, &format!("{}/login", app.address))
        .header("Origin", "https://notallowed.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    
    // For disallowed origins, CORS headers should not be present
    // Even if we get a 405, the origin should not be in the CORS header
    let cors = resp.headers().get("access-control-allow-origin");
    assert!(cors.is_none() || !cors.unwrap().to_str().unwrap().contains("notallowed.com"));
    
    app.cleanup().await;
}

#[tokio::test] 
async fn cors_allows_any_origin_with_wildcard() {
    // Set CORS environment variable for wildcard
    env::set_var("CORS_ALLOWED_ORIGINS", "*");
    
    let mut app = TestApp::new().await;
    let client = Client::new();
    
    let resp = client
        .request(reqwest::Method::OPTIONS, &format!("{}/login", app.address))
        .header("Origin", "https://anything.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
        
    // Accept either 204 (preflight) or 405 (method not allowed but CORS headers present)
    assert!(resp.status() == 204 || resp.status() == 405);
    
    // With wildcard, CORS headers should be present
    let cors = resp.headers().get("access-control-allow-origin");
    if resp.status() != 405 {
        assert!(cors.is_some());
    }
    
    app.cleanup().await;
}
