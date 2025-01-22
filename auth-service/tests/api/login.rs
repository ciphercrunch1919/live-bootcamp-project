use crate::helpers::{ get_random_email, TestApp };
use auth_service::ErrorResponse;

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

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup = serde_json::json!({
        "email": random_email,
        "password": "password",
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
        serde_json::json!({
            "email": "example@email.com",
            "password": "dsjkfae23"
        })
    ];

    for test_case in test_cases {
        let response = app.post_login(&test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input {:?}",
            test_case
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
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password1234",
        "requires2FA": false,
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let test_cases = vec![
        (random_email.as_str(), "wrong-password"),
        ("wrong@email.com", "password123"),
        ("wrong@email.com", "wrong-password"),
    ];

    for (email, password) in test_cases {
        let login_body = serde_json::json!({
            "email": email,
            "password": password,
        });
        let response = app.post_login(&login_body).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input {:?}",
            login_body
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Incorrect credentials".to_owned()
        );
    }
}