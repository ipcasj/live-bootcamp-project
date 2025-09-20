use auth_service::domain::data_stores::BannedTokenStore;
use crate::helpers::TestApp;

// use reqwest::cookie::CookieStore; // unused
#[tokio::test]
async fn should_return_500_if_internal_error() {
    let app = TestApp::new().await;
    // Set the jwt cookie to trigger500
    let url = app.address.parse().unwrap();
    app.cookie_jar.add_cookie_str("jwt=trigger500", &url);
    let response = app.logout().await;
    assert_eq!(response.status(), 500);
}

// 400 Bad Request: No cookie/token present
#[tokio::test]
async fn logout_should_return_400_if_no_cookie() {
    let app = TestApp::new().await;
    // No login, so no cookie is set
    let response = app.logout().await;
    assert_eq!(response.status(), 400);
}

// 401 Unauthorized: Invalid/expired session (simulate by tampering cookie)
#[tokio::test]
async fn logout_should_return_401_if_invalid_cookie() {
    let app = TestApp::new().await;
    // Set an invalid cookie manually
    let url = app.address.parse().unwrap();
    app.cookie_jar.add_cookie_str("jwt=invalid.jwt.token", &url);
    let response = app.logout().await;
    assert_eq!(response.status(), 401);
}

// 200 OK: Successful logout, cookie cleared
#[tokio::test]
async fn logout_should_clear_cookie_on_success() {
    let mut app = TestApp::new().await;
    
    // Use unique email for each test to avoid token collisions
    let email = TestApp::get_random_email();
    let password = "password123";
    
    // First, signup and login to set a valid cookie
    let _ = app.signup(&email, password, false).await;
    let login_response = app.login(&email, password).await;
    // Extract token from Set-Cookie
    let set_cookie = login_response.headers().get("set-cookie").expect("No set-cookie header").to_str().unwrap();
    let token = set_cookie.split(';').find(|s| s.trim_start().starts_with("jwt=")).unwrap().trim_start_matches("jwt=");
    // Now logout
    let response = app.logout().await;
    assert_eq!(response.status(), 200);
    // Check that the Set-Cookie header clears the cookie
    let cookies: Vec<_> = response.headers().get_all("set-cookie").iter().collect();
    assert!(cookies.iter().any(|c| c.to_str().unwrap().contains("jwt=;")));
    // Check that the token is banned
    assert!(app.banned_token_store.is_banned(token).await);
    
    app.cleanup().await;
}

#[tokio::test]
async fn test_logout() {
    let mut app = TestApp::new().await;
    
    // Use unique email for each test to avoid token collisions
    let email = TestApp::get_random_email();
    let password = "password123";
    
    // First, signup and login to set a valid cookie
    let signup_response = app.signup(&email, password, false).await;
    assert_eq!(signup_response.status(), 201);
    
    let login_response = app.login(&email, password).await;
    assert_eq!(login_response.status(), 200);
    
    // Verify that a JWT cookie was set
    let set_cookie_header = login_response.headers().get("set-cookie");
    assert!(set_cookie_header.is_some(), "Login should set JWT cookie");
    
    // Small delay to ensure JWT processing is complete
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    let logout_response = app.logout().await;
    let status = logout_response.status();
    
    // Debug on failure
    if status != 200 {
        eprintln!("Logout failed with status: {}", status);
        eprintln!("Response text: {:?}", logout_response.text().await);
    }
    
    assert_eq!(status, 200);
    
    app.cleanup().await;
}