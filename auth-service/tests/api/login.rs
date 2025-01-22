use crate::helpers::{ get_random_email, TestApp };
use auth_service::{ utils::constants::JWT_COOKIE_NAME, ErrorResponse };

/* 
#[tokio::test]
async fn login_returns_200() {
    let app = TestApp::new().await;

    let login_body = serde_json::json!({
        "email": "test@example.com",
        "password": "password123",
    });
    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);
}
    */

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let test_case = [
        serde_json::json!({
        }),
        serde_json::json!({
            "email": random_email
        }),
        serde_json::json!({
            "password": random_email
        })
    ];

    for test in test_case.iter() {
        let response = app.post_login(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input {:?}",
            test
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.

    let app = TestApp::new().await;
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password1234",
        "requires2FA": false, 
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let test_cases = vec![
        ("invalid_email", "password123"),
        (random_email.as_str(), "invalid"),
        ("", "password123"),
        (random_email.as_str(), ""),
        ("", ""),
    ];
    for (email, password) in test_cases {
        let login_body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let response = app.post_login(&login_body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input {:?}",
            login_body
        );
    }
}

#[tokio::test]
async fn should_return_401_if_invalid_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message. 
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "password": "ieiekkdk"
        }),
        serde_json::json!({
            "email": random_email,
            "password": "jfdkdslaj"
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password3223"
        })
    ];

    for test in test_cases.iter() {
        let response = app.post_login(test).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            test
        );
    }
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}