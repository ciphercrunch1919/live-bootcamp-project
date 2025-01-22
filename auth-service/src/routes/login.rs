use axum::{ extract::State, http::StatusCode, response::IntoResponse, Json };
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

use crate::{ app_state::AppState, domain::{ AuthAPIError, Email, Password } };

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email.clone()).map_err(|_| AuthAPIError::IncorrectCredentails)?;
    let password = Password::parse(request.password.clone()).map_err(|_| AuthAPIError::IncorrectCredentails)?;

    let user_store = state.user_store.read().await;

    // call `user_store.validate_user` and return
    // `AuthAPIError::IncorrectCredentials` if valudation fails.

    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentails);
    };

    // call `user_store.get_user`. Return AuthAPIError::IncorrectCredentials if the operation fails.

    if user_store.get_user(&email).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentails);
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}