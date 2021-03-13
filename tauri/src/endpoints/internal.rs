use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  ValidateSalt { salt: String },
}

impl Cmd {
  pub fn run(self) -> crate::Result<InvokeResponse> {
    match self {
      Self::ValidateSalt { salt } => validate_salt(salt),
    }
  }
}

/// Validates a salt.
pub fn validate_salt(salt: String) -> crate::Result<InvokeResponse> {
  Ok(crate::salt::is_valid(salt).into())
}
