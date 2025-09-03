use reqwest::StatusCode;
use serde_json::json;

use crate::helpers::TestApp;

#[tokio::test]
async fn delete_existing_account_returns_200_and_removes_user() {
    let app = TestApp::spawn().await;
    // Create user
    let email = "delete_me@example.com";
    let password = "password123";
    let signup_body = json!({
        "email": email,
        "password": password,
        "requires2FA": false
    });
    let signup_res = app.post_signup(&signup_body).await;
    assert_eq!(signup_res.status(), StatusCode::CREATED);

    // Delete user

    let delete_res = app.delete_account(email).await;
    assert_eq!(delete_res.status(), StatusCode::OK);

    // Try to delete again (should return 404 or 400)
    let delete_res2 = app.delete_account(email).await;
    assert!(delete_res2.status() == StatusCode::BAD_REQUEST || delete_res2.status() == StatusCode::NOT_FOUND);

    // Try to login (should fail)
    let login_body = json!({ "email": email, "password": password });
    let login_res = app.post_login(&login_body).await;
    assert_eq!(login_res.status(), StatusCode::BAD_REQUEST);
}
