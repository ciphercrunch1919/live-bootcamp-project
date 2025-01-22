use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use serde_json::json;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "token": true
        }),
        serde_json::json!({}),
        serde_json::json!({
            "token": 123
        })
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_token(test_case);
        assert_eq!(
            response.await.status().as_u16(),
            422,
            "Failed for input {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup).await;
    assert_eq!(response.status().as_u16(), 201);

    let login = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Auth cookie not found");
    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let verify_token_body = serde_json::json!({
        "token": &token,
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let response = app.post_verify_token(&json!({
        "token": "invalid_token"
    })).await;

    assert_eq!(response.status().as_u16(), 401);
}