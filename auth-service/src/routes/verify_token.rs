use axum::{ extract::State, http::StatusCode, response::IntoResponse, Json };
use serde::Deserialize;

use crate::{ app_state::AppState, domain::AuthAPIError, utils::auth::validate_token };

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let banned_token_store = state.banned_token_store.read().await;
    validate_token(&request.token, &*banned_token_store).await.map_err(|_| AuthAPIError::InvalidToken)?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}