use validator::validate_email;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Email(String);

impl Email {
  pub fn parse(email: String) -> Result<Email, String> {
    if validate_email(&email) {
      Ok(Self(email))
    }
    else {
      Err(format!("{} is not a valid email", email))
    }
  }
}

impl AsRef<str> for Email {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::Email;

  use fake::faker::internet::en::SafeEmail;
  use fake::Fake;

  #[test]
  fn empty_string_is_rejected() {
    let email = "".to_string();
    assert!(Email::parse(email).is_err());
  }

  #[test]
  fn email_missing_at_symbol_is_rejected() {
    let email = "ursuladomain.com".to_string();
    assert!(Email::parse(email).is_err());
  }

      #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
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
        Email::parse(valid_email.0).is_ok()
    }
}