use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PingResponse {
  pub value: Option<String>,
}
