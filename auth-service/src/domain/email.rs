use std::hash::Hash; // New!
use validator::validate_email;
use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret}; // New!

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

// New Implementation of PartialEq
impl PartialEq for Email {
  fn eq(&self, other: &Self) -> bool {
      self.0.expose_secret() == other.0.expose_secret()
  }
}

// New Implementation of Hash
impl Hash for Email {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.0.expose_secret().hash(state);
  }
}

// New Implementation of Eq
impl Eq for Email {}

impl Email {
  // Updated!
  pub fn parse(email: Secret<String>) -> Result<Email> {
    if validate_email(email.expose_secret()) {
      Ok(Self(email))
    }
    else {
      Err(eyre!(format!(
        "{} is not a valid email", 
        email.expose_secret()
      )))
    }
  }
}

impl AsRef<Secret<String>> for Email {
  fn as_ref(&self) -> &Secret<String> {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::Email;

  use fake::faker::internet::en::SafeEmail;
  use fake::Fake;
  use secrecy::Secret; // New!

  #[test]
  fn empty_string_is_rejected() {
    let email = Secret::new("".to_string());
    assert!(Email::parse(email).is_err());
  }

  #[test]
  fn email_missing_at_symbol_is_rejected() {
    let email = Secret::new("ursuladomain.com".to_string());
    assert!(Email::parse(email).is_err());
  }

      #[test]
    fn email_missing_subject_is_rejected() {
        let email = Secret::new("@domain.com".to_string());
        assert!(Email::parse(email).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(Secret::new(valid_email.0)).is_ok()
    }
}