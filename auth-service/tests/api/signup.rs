use crate::helpers::TestApp;
use auth_service::ErrorResponse;

#[tokio::test]
async fn test_signup() {
    let app = TestApp::new().await;

    let signup_body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": false
    });
    let response = app.signup(&signup_body).await;
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let test_cases = vec![
        serde_json::json!({
            // missing email
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            // missing password
            "email": "malformed1@example.com",
            "requires2FA": false
        }),
        serde_json::json!({
            // missing both email and password
            "requires2FA": true
        }),
        serde_json::json!({
            // completely empty object
        }),
    ];
    for test_case in &test_cases {
        let response = app.signup(test_case).await;
            assert_eq!(
                response.status().as_u16(),
                422,
                "Failed for input: {:?}",
                test_case
            );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let signup_body = serde_json::json!({
        "email": "valid@example.com",
        "password": "validpassword",
        "requires2FA": false
    });
    let response = app.signup(&signup_body).await;
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
        serde_json::json!({
            "email": "invalidemail",
            "password": "short",
            "requires2FA": false
        }),
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "test@example.com",
            "password": "short",
            "requires2FA": false
        }),
    ];
    for input in invalid_inputs {
        let response = app.signup(&input).await;
    assert_eq!(response.status(), 400);
    let body: ErrorResponse = response.json().await.expect("Invalid JSON");
    assert_eq!(body.error, "Invalid credentials");
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let app = TestApp::new().await;

    let signup_body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": false
    });
    let response = app.signup(&signup_body).await;
    assert_eq!(response.status(), 201);

    // Try to sign up the same user again
    let response = app.signup(&signup_body).await;
    assert_eq!(response.status(), 409);
    let body: ErrorResponse = response.json().await.expect("Invalid JSON");
    assert_eq!(body.error, "User already exists");
}