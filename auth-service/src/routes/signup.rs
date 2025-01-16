use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{app_state::AppState, domain::User};

pub async fn signup(State(state): State<Arc<AppState>>,Json(request): Json<SignupRequest>) -> impl IntoResponse {
    // Create a new `User` instance using data in the `request`
    let user = request.into_user();

    let mut user_store = state.user_store.write().await;

    // Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.

    let _ = user_store.add_user(user);
    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    (StatusCode::CREATED, response)
}

#[derive(Deserialize,PartialEq, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Debug, PartialEq,Deserialize)]
pub struct SignupResponse {
    pub message: String,
}

impl SignupRequest {
    fn into_user(&self) -> User {
	User::new(self.email.clone(), self.password.clone(), self.requires_2fa)
    }
}