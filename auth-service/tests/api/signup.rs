#[tokio::test]
async fn should_return_500_if_user_store_fails() {
    let app = TestApp::new().await;
    // This email will trigger a 500 error in the handler
    let email = "trigger500@example.com";
    let password = "password123";
    let response = app.signup(email, password, false).await;
    assert_eq!(response.status(), 500);
    let body: ErrorResponse = response.json().await.expect("Invalid JSON");
    assert_eq!(body.code, "internal_server_error");
    assert!(body.error.contains("Unexpected error"), "error message: {}", body.error);
}
use crate::helpers::TestApp;
use auth_service::ErrorResponse;

#[tokio::test]
async fn test_signup() {
    let app = TestApp::new().await;

    let email = "test@example.com";
    let password = "password";
    let response = app.signup(email, password, false).await;
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    // Malformed input tests: call with empty/invalid email or password
    let cases = vec![
        ("", "password123", true), // missing email
        ("malformed1@example.com", "", false), // missing password
        ("", "", true), // missing both
    ];
    for (email, password, requires_2fa) in cases {
        let response = app.signup(email, password, requires_2fa).await;
        assert_eq!(response.status().as_u16(), 422, "Failed for input: email={}, password={}", email, password);
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let email = "valid@example.com";
    let password = "validpassword";
    let response = app.signup(email, password, false).await;
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters

    // Create an array of invalid inputs. Then, iterate through the array and 
    // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
    let invalid_inputs = vec![
        ("invalidemail", "short", false),
        ("", "password123", true),
        ("test@example.com", "short", false),
    ];
    for (email, password, requires_2fa) in invalid_inputs {
        let response = app.signup(email, password, requires_2fa).await;
        assert_eq!(response.status(), 422);
        let body: ErrorResponse = response.json().await.expect("Invalid JSON");
        assert_eq!(body.error, "Malformed credentials");
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let app = TestApp::new().await;

    let email = "test@example.com";
    let password = "password";
    let response = app.signup(email, password, false).await;
    assert_eq!(response.status(), 201);

    // Try to sign up the same user again
    let response = app.signup(email, password, false).await;
    assert_eq!(response.status(), 409);
    let body: ErrorResponse = response.json().await.expect("Invalid JSON");
    assert_eq!(body.error, "User already exists");
}