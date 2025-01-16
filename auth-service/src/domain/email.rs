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

