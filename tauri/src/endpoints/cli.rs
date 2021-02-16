use serde::Deserialize;
use serde_json::Value as JsonValue;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The get CLI matches API.
  CliMatches,
}

impl Cmd {
  #[allow(unused_variables)]
  pub async fn run(self, context: &crate::app::Context) -> crate::Result<JsonValue> {
    match self {
      #[allow(unused_variables)]
      Self::CliMatches => {
        #[cfg(cli)]
        return tauri_api::cli::get_matches(&context.config)
          .map_err(Into::into)
          .and_then(super::to_value);
        #[cfg(not(cli))]
          Err(crate::Error::ApiNotEnabled(
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri.cli)".to_string(),
          ))
      }
    }
  }
}
