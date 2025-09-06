#[tokio::test]
async fn refresh_token_rotation_and_revocation() {
    // Assume test user exists and login endpoint issues refresh token
    let client = reqwest::Client::new();
    let email = "test_refresh@example.com";
    let password = "password123";

    // Signup
    let _ = client.post("http://localhost:3000/signup")
        .json(&json!({"email": email, "password": password}))
        .send().await;

    // Login to get refresh token
    let login_resp = client.post("http://localhost:3000/login")
        .json(&json!({"email": email, "password": password}))
        .send().await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_json: serde_json::Value = login_resp.json().await.unwrap();
    let refresh_token = login_json.get("refresh_token").unwrap().as_str().unwrap();

    // Use refresh token to get new tokens
    let refresh_resp = client.post("http://localhost:3000/refresh-token")
        .json(&json!({"refresh_token": refresh_token}))
        .send().await.unwrap();
    assert_eq!(refresh_resp.status(), StatusCode::OK);
    let refresh_json: serde_json::Value = refresh_resp.json().await.unwrap();
    let new_refresh_token = refresh_json.get("refresh_token").unwrap().as_str().unwrap();

    // Old refresh token should now be revoked (banned)
    let second_resp = client.post("http://localhost:3000/refresh-token")
        .json(&json!({"refresh_token": refresh_token}))
        .send().await.unwrap();
    assert_eq!(second_resp.status(), StatusCode::UNAUTHORIZED);

    // New refresh token should work
    let third_resp = client.post("http://localhost:3000/refresh-token")
        .json(&json!({"refresh_token": new_refresh_token}))
        .send().await.unwrap();
    assert_eq!(third_resp.status(), StatusCode::OK);
}
use super::utils::auth::generate_auth_token_from_str;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

#[tokio::test]
async fn test_refresh_token_success() {
    // Simulate a valid refresh token (for demo, use a valid access token)
    let email = "test@example.com";
    let refresh_token = generate_auth_token_from_str(email).unwrap();
    let app = crate::setup_test_app().await;
    let client = reqwest::Client::new();
    let res = client
        .post(&format!("{}/refresh-token", app.base_url))
        .json(&json!({"refresh_token": refresh_token}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("access_token").is_some());
}
