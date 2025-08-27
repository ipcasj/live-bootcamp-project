//! Unit tests for the signup handler using a mock user store.
use axum::{extract::State, http::{StatusCode, Request}, response::Response, body::Body};
use auth_service::routes::signup::{signup, SignupRequest};
use auth_service::domain::UserStore;
use crate::mock_user_store::MockUserStore;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn signup_returns_201_on_success() {
    let mock_store = Arc::new(RwLock::new(MockUserStore::default()));
    let app_state = auth_service::app_state::AppState::new(mock_store);
    let req = SignupRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        requires_2fa: false,
    };
    let response = signup(State(app_state), axum::Json(req)).await.unwrap().into_response();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn signup_returns_409_on_duplicate() {
    let mut mock = MockUserStore::default();
    mock.add_user(auth_service::domain::User::new(
        "test@example.com".to_string(),
        "password123".to_string(),
        false,
    )).await.unwrap();
    let mock_store = Arc::new(RwLock::new(mock));
    let app_state = auth_service::app_state::AppState::new(mock_store);
    let req = SignupRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        requires_2fa: false,
    };
    let response = signup(State(app_state), axum::Json(req)).await;
    assert!(matches!(response, Err(auth_service::domain::AuthAPIError::UserAlreadyExists)));
}
