use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
};

pub async fn verify_2fa(
    State(state): State<AppState>, // New!
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?; // Validate the email in `request`

    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?; // Validate the login attempt ID in `request`

    let two_fa_code = TwoFACode::parse(request.two_fa_code.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?; // Validate the 2FA code in `request`

    // New!
    let two_fa_code_store = state.two_fa_code_store.write().await;

    // Call `two_fa_code_store.get_code`. If the call fails
    // return a `AuthAPIError::IncorrectCredentials`.
    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(tuple) => tuple,
        Err(_) => return Err(AuthAPIError::IncorrectCredentials),
    };

    // Validate that the `login_attempt_id` and `two_fa_code`
    // in the request body matches values in the `code_tuple`. 
    // If not, return a `AuthAPIError::IncorrectCredentials`.
    if !(code_tuple.1 == two_fa_code && code_tuple.0 == login_attempt_id) {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    Ok(StatusCode::OK.into_response())
}

// implement the Verify2FARequest struct. See the verify-2fa route contract in step 1 for the expected JSON body.
#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}