use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    // Implement all required methods. Note that you will need to make SQL queries against our PostgreSQL instance inside these methods.

    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
      let password_hash = compute_password_hash(user.password.as_ref().to_string())
          .await
          .map_err(|_| UserStoreError::InvalidCredentials)?;

      let query = sqlx::query!(
          "
          INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)
          ",
          user.email.as_ref(),
          password_hash,
          user.requires_2fa
      );

      query
          .execute(&self.pool)
          .await
          .map_err(|_| UserStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) ->Result<User, UserStoreError> {
      let user = sqlx::query!(
        "
        SELECT email, password_hash, requires_2fa FROM users WHERE email = $1
        ",
        email.as_ref()
      )
      .fetch_optional(&self.pool)
      .await
      .map_err(|_| UserStoreError::UnexpectedError)?
      .ok_or(UserStoreError::UserNotFound)?;

      Ok(User {
        email: Email::parse(user.email)
          .map_err(|_| UserStoreError::InvalidCredentials)?,
        password: Password::parse(user.password_hash)
          .map_err(|_| UserStoreError::InvalidCredentials)?,
        requires_2fa: user.requires_2fa
      })
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
      let row = sqlx::query!(
        "
        SELECT password_hash FROM users WHERE email = $1
        ",
        email.as_ref()
      )
      .fetch_one(&self.pool)
      .await
      .map_err(|_| {
          UserStoreError::UnexpectedError
      })?;

      verify_password_hash(row.password_hash, password.as_ref().to_string())
        .await
        .map_err(|_| UserStoreError::InvalidCredentials)?;

      Ok(())
    }
}

// Helper function to verify if a given password matches an expected hash
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
pub async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let thread = tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| e.into())
    })
    .await;

    thread?
}

// Helper function to hash passwords before persisting them in the database.
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
pub async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let thread = tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

        Ok(password_hash)
    })
    .await;

    thread?
}