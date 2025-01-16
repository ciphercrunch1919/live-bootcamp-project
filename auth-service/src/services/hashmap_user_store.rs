use std::collections::HashMap;

use crate::domain::{User, UserStoreError, UserStore, Password, Email};
#[derive(Default, Clone)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.clone(), user);
        return Ok(());
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                if user.password.eq(password) {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

// Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("email@example.com".to_owned()).unwrap();
        let password = Password::parse("password".to_owned()).unwrap();
        let user = User::new(email, password, false);
        user_store.add_user(user.clone()).await.unwrap();
        let get_user = user_store.get_user(&user.email).await;

        assert_eq!(get_user, Ok(user.clone()));

        assert_eq!(
            user_store.add_user(user).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("email@example.com".to_owned()).unwrap();
        let password = Password::parse("password".to_owned()).unwrap();
        let user = User::new(email, password, false);
        user_store.add_user(user.clone()).await.unwrap();
        let get_user = user_store.get_user(&user.email).await;

        assert_eq!(get_user.unwrap().email, user.email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("example@email.com".to_owned()).unwrap();
        let password = Password::parse("password".to_owned()).unwrap();
        let user = User::new(email, password, false);
        user_store.add_user(user.clone()).await.unwrap();
        assert_eq!(user_store.validate_user(&user.email, &user.password).await, Ok(()));
    }
}