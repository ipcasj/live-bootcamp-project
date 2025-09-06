use reqwest::Client;

#[tokio::test]
async fn cors_allows_configured_origin() {
    // Set up: assume server is running with CORS_ALLOWED_ORIGINS=https://allowed.com
    let client = Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, "http://localhost:3000/login")
        .header("Origin", "https://allowed.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204); // No Content for preflight
    let cors = resp.headers().get("access-control-allow-origin").unwrap();
    assert_eq!(cors, "https://allowed.com");
}

#[tokio::test]
async fn cors_rejects_unlisted_origin() {
    // Set up: assume server is running with CORS_ALLOWED_ORIGINS=https://allowed.com
    let client = Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, "http://localhost:3000/login")
        .header("Origin", "https://notallowed.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    // Should NOT have the CORS header
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn cors_allows_any_origin_with_wildcard() {
    // Set up: assume server is running with CORS_ALLOWED_ORIGINS=*
    let client = Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, "http://localhost:3000/login")
        .header("Origin", "https://anything.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let cors = resp.headers().get("access-control-allow-origin").unwrap();
    assert_eq!(cors, "*");
}
