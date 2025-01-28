use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

// implement TwoFACodeStore for HashmapTwoFACodeStore
#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
  async fn add_code(&mut self, email: Email, login_attempt_id: LoginAttemptId, two_fa_code: TwoFACode) -> Result<(), TwoFACodeStoreError> {
    if self.codes.contains_key(&email) {
      return Err(TwoFACodeStoreError::UnexpectedError);
    }
    self.codes.insert(email, (login_attempt_id, two_fa_code));
    Ok(())
  }

  async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
    match self.codes.remove(email) {
      Some(_) => Ok(()),
      None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
    }
  }

  async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
    match self.codes.get(email) {
      Some(value) => Ok(value.clone()),
      None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
    }
  }
} 

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_add_code() {
    let mut code_store = HashmapTwoFACodeStore::default();
    let two_fa_code = TwoFACode::default();
    let email = Email::parse("test@example.com".to_owned()).unwrap();
    let login_attempt_id = LoginAttemptId::default();

    // Test adding a new two_fa_code
    let result = code_store.add_code(email.clone(),login_attempt_id.clone(),two_fa_code.clone()).await;
    assert!(result.is_ok());

    // Test adding an existing two_fa_code
    let result = code_store.add_code(email,login_attempt_id,two_fa_code).await;
    assert_eq!(result, Err(TwoFACodeStoreError::UnexpectedError));
  }

  #[tokio::test]
  async fn test_get_code() {
    let mut code_store = HashmapTwoFACodeStore::default();
    let email = Email::parse("test@example.com".to_owned()).unwrap();
    let bogus = Email::parse("nonexistent@bogus.com".to_owned()).unwrap();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    // Test getting a two_fa_code that exists
    let result = code_store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await;
    assert!(result.is_ok());
    let result = code_store.get_code(&email).await;
    assert_eq!(result, Ok((login_attempt_id,two_fa_code)));

    // Test getting a two_fa_code that doesn't exist
    let result = code_store
      .get_code(&bogus)
      .await;

    assert_eq!(result, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
  }

  #[tokio::test]
  async fn test_remove_code() {
    let mut code_store = HashmapTwoFACodeStore::default();
    let email = Email::parse("test@example.com".to_owned()).unwrap();
    let bogus = Email::parse("nonexistent@bogus.com".to_owned()).unwrap();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let result = code_store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await;
    assert!(result.is_ok());
    let result = code_store.get_code(&email).await;
    assert_eq!(result, Ok((login_attempt_id, two_fa_code)));

    // Remove existing code
    let result = code_store.remove_code(&bogus).await;
    assert_eq!(result, Err(TwoFACodeStoreError::LoginAttemptIdNotFound));
  }
}