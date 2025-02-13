use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)] // Updated
pub struct Password(Secret<String>); //  Updated

impl PartialEq for Password {
  fn eq(&self, other: &Self) -> bool {
    // We can use the expose_secret methoid to expose the secret in a controlled manner when needed!
    self.0.expose_secret() == other.0.expose_secret() //update
  }
}

impl Password {
  pub fn parse(password: Secret<String>) -> Result<Password> {
    if password.expose_secret().len() >= 8 {
      Ok(Self(password))
    }
    else {
      Err(eyre!("Failed to parse string to a Password type"))
    }
  }
}

impl AsRef<Secret<String>> for Password {
  fn as_ref(&self) -> &Secret<String> {
    &self.0
  }
}
#[cfg(test)]
mod tests {
    use super::Password;

    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;
    use secrecy::Secret;

    #[test]
    fn empty_string_is_rejected() {
        let password = Secret::new("".to_owned());
        assert!(Password::parse(password).is_err());
    }
    #[test]
    fn string_less_than_8_characters_is_rejected() {
        let password = Secret::new("1234567".to_owned());
        assert!(Password::parse(password).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let password = FakePassword(8..30).fake_with_rng(g);
            Self(Secret::new(password))
        }
    }
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}