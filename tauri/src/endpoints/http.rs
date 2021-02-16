use serde::Deserialize;
use serde_json::Value as JsonValue;
use tauri_api::http::{make_request as request, HttpRequestOptions};

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The HTTP request API.
  HttpRequest { options: Box<HttpRequestOptions> },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<JsonValue> {
    match self {
      Self::HttpRequest { options } => {
        #[cfg(http_request)]
        return request(*options)
          .map_err(Into::into)
          .and_then(super::to_value);
        #[cfg(not(http_request))]
        Err(crate::Error::ApiNotAllowlisted("httpRequest".to_string()))
      }
    }
  }
}
