use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME, // New!
    ErrorResponse,
};

use serde_json::json;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();

    let test_cases = [
        serde_json::json!({
            "loginAttemptId": login_attempt_id.parse::<String>().unwrap(),
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "2FACode": "123456",
        }),
        serde_json::json!({
            "password": "password123",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;

        assert_eq!(
            response.status().as_u16(), 
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let two_fa_code = TwoFACode::default().as_ref().to_owned();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": login_attempt_id.parse::<String>().unwrap(),
            "2FACode": "bad 2fa code".to_string()
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "bad login attempt".to_string(),
            "2FACode": two_fa_code.parse::<String>().unwrap()
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "bad login attempt",
            "2FACode": "bad 2fa code",
        }),
    ];

    for test_case in test_cases {
        let response = app.post_verify_2fa(&test_case).await;

        assert_eq!(response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
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
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email.clone(), 
        "password": "password123", 
        "requires2FA": true
    });

    let response = app.post_signup(&signup).await;

    assert_eq!(response.status().as_u16(), 201);

    let wrong_email = get_random_email();
    let wrong_login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let wrong_two_fa_code = TwoFACode::default().as_ref().to_owned();

    let test_case = vec![
        serde_json::json!({
            "email": wrong_email,
            "loginAttemptId": wrong_login_attempt_id,
            "2FACode": wrong_two_fa_code
        }),
    ];

    for test in test_case.iter() {
        let request_body = serde_json::json!({
            "email": test["email"],
            "loginAttemptId": test["loginAttemptId"],
            "2FACode": test["2FACode"]
        });

        let response = app.post_verify_2fa(&request_body).await;

        assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        request_body
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
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;
    let email = get_random_email();

    // Step 1: Sign up a user
    let signup_payload = json!({
        "email": email.clone(),
        "password": "password123",
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    // Step 2: Attempt login (should require 2FA)
    let login_payload = json!({
        "email": email.clone(),
        "password": "password123",
    });
    let login_response = app.post_login(&login_payload).await;
    println!("Login response: {:?}", login_response);
    assert_eq!(login_response.status().as_u16(), 206);

    let login_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Failed to deserialize TwoFactorAuthResponse");

    assert_eq!(login_body.message, "2FA required");
    assert!(!login_body.login_attempt_id.is_empty());

    // Step 3: Retrieve the stored 2FA code
    let two_fa_code = {
        let two_fa_store = app.two_fa_code_store.write().await;
        let email_obj = Email::parse(email.clone()).unwrap();
        two_fa_store
            .get_code(&email_obj)
            .await
            .expect("Failed to retrieve 2FA code")
    };

    let first_code = two_fa_code.1.as_ref();

    // Step 4: Attempt login again with the same credentials
    let second_login_response = app.post_login(&login_payload).await;
    println!("Second login response: {:?}", second_login_response);

    // TODO: passes 500 for some reason failing the test 
    assert_eq!(second_login_response.status().as_u16(), 206);

    // Step 5: Verify 2FA with the retrieved code (expecting 401 Unauthorized)
    let verify_2fa_payload = json!({
        "email": email,
        "loginAttemptId": login_body.login_attempt_id,
        "2FACode": first_code,
    });
    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_payload).await;
    println!("Verify 2FA response: {:?}", verify_2fa_response);

    assert_eq!(verify_2fa_response.status().as_u16(), 401);
    assert_eq!(
        verify_2fa_response
            .json::<ErrorResponse>()
            .await
            .expect("Failed to deserialize ErrorResponse")
            .error,
        "Incorrect Credentials"
    );
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    // Make sure to assert the auth cookie gets set
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let response = app
        .post_signup(&serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }))
        .await;
    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .post_login(&serde_json::json!({
            "email": random_email,
            "password": "password123"
        }))
        .await;
    assert_eq!(response.status().as_u16(), 206);
    
    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": code
    });

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {    
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(response_body.message, "2FA required".to_owned());
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": code
    });

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 401);
}