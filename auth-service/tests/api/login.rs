use crate::helpers::{ get_random_email, TestApp };
use secrecy::ExposeSecret;
use auth_service::{
    domain::Email,
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};
use wiremock::{matchers::{method, path}, Mock, ResponseTemplate};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup).await;

    assert_eq!(response.status().as_u16(), 201);

    let test_cases = [
        serde_json::json!({
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases {
        let response = app.post_login(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup).await;

    assert_eq!(response.status().as_u16(), 201);

    let test_cases = vec![
        ("invalid_email", "password123"),
        (random_email.as_str(), "invalid"),
        ("", "password123"),
        (random_email.as_str(), ""),
        ("", ""),
    ];

    for (email, password) in test_cases {
        let login = serde_json::json!({
            "email": email,
            "password": password
        });
        let response = app.post_login(&login).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            login
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid Credentials".to_owned()
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup).await;

    assert_eq!(response.status().as_u16(), 201);

    let test_cases = vec![
        (random_email.as_str(), "wrong-password"),
        ("wrong@email.com", "password123"),
        ("wrong@email.com", "wrong-password"),
    ];

    for (email, password) in test_cases {
        let login = serde_json::json!({
            "email": email,
            "password": password
        });
        let response = app.post_login(&login).await;

        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            login
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Incorrect Credentials".to_owned()
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;

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
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup).await;

    assert_eq!(response.status().as_u16(), 201);

    // Define an expectation for the mock server
    Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
        .and(method("POST")) // Expect the HTTP method to be POST
        .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status
        .expect(1) // Expect this request to be made exactly once
        .mount(&app.email_server) // Mount this expectation on the mock email server
        .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding

    let login = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    

    let response = app.post_login(&login).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    // assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
    let login_attempt_id = app.two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.into()).unwrap())
        .await
        .unwrap().0.expose_secret().to_string();

    assert_eq!(login_attempt_id, json_body.login_attempt_id);

    app.clean_up().await;
}