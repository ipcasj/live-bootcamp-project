use crate::helpers::TestApp;

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