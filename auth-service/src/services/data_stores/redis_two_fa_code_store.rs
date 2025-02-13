use std::sync::Arc;

use redis::{Commands, Connection};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use color_eyre::eyre::Context;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {

    #[tracing::instrument(name = "Add Code", skip_all)]
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        let data = TwoFATuple(
            login_attempt_id.expose_secret().to_owned(),
            code.as_ref().expose_secret().to_owned(),
        );
        let serialized_data = serde_json::to_string(&data)
            .wrap_err("failed to serialize 2FA tuple") // New!
            .map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

        let _: () = self
            .conn
            .write()
            .await
            .set_ex(&key, serialized_data, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis") // New! 
            .map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

        Ok(())
    }

    #[tracing::instrument(name = "Remove Code", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);

        let _: () = self
            .conn
            .write()
            .await
            .del(&key)
            .wrap_err("failed to delete 2FA code from Redis") // New!
            .map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

        Ok(())
    }

    #[tracing::instrument(name = "Get Code", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);

        match self.conn.write().await.get::<_, String>(&key) {
            Ok(value) => {
                let data: TwoFATuple = serde_json::from_str(&value)
                    .wrap_err("failed to deserialize 2FA tuple") // New!
                    .map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

                let login_attempt_id =
                    LoginAttemptId::parse(data.0.into()).map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

                let email_code =
                    TwoFACode::parse(data.1.into()).map_err(TwoFACodeStoreError::UnexpectedError)?; // Updated!

                Ok((login_attempt_id, email_code))
            }
            Err(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}