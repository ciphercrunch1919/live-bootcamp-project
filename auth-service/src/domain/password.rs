use color_eyre::eyre::{eyre, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct Password(String);

impl Password {
  pub fn parse(password: String) -> Result<Password> {
    if password.len() >= 8 {
      Ok(Self(password))
    }
    else {
      Err(eyre!(format!("Password must be at least 8 characters long, but was {} characters long", password.len())))
    }
  }
}

impl AsRef<str> for Password {
  fn as_ref(&self) -> &str {
    &self.0
  }
}
#[cfg(test)]
mod tests {
    use super::Password;

    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;

    #[test]
    fn empty_string_is_rejected() {
        let password = "".to_owned();
        assert!(Password::parse(password).is_err());
    }
    #[test]
    fn string_less_than_8_characters_is_rejected() {
        let password = "1234567".to_owned();
        assert!(Password::parse(password).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let password = FakePassword(8..30).fake_with_rng(g);
            Self(password)
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}