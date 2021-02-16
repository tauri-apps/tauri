use serde::Deserialize;
use serde_json::Value as JsonValue;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  ValidateSalt { salt: String },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<JsonValue> {
    match self {
      Self::ValidateSalt { salt } => validate_salt(salt),
    }
  }
}

/// Validates a salt.
pub fn validate_salt(salt: String) -> crate::Result<JsonValue> {
  Ok(JsonValue::Bool(crate::salt::is_valid(salt)))
}
