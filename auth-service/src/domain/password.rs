
#[derive(Debug, PartialEq, Clone)]
pub struct Password(String);

impl Password {
  pub fn parse(password: String) -> Result<Password, String> {
    if password.len() >= 8 {
      Ok(Self(password))
    }
    else {
      Err(format!("Password must be at least 8 characters long, but was {} characters long", password.len()))
    }
  }
}

impl AsRef<str> for Password {
  fn as_ref(&self) -> &str {
    &self.0
  }
}