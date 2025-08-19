use crate::helpers::TestApp;

// Tokio's test macro is used to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn test_signup() {
    let app = TestApp::new().await;

    let response = app.signup("test@example.com", "password").await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_login() {
    let app = TestApp::new().await;

    let response = app.login("test@example.com", "password").await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_logout() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_verify_2fa() {
    let app = TestApp::new().await;

    let response = app.verify_2fa("123456").await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_verify_token() {
    let app = TestApp::new().await;

    let response = app.verify_token("token").await;

    assert_eq!(response.status(), 200);
}