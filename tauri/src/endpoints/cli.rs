use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The get CLI matches API.
  CliMatches,
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run(self, context: &crate::app::Context) -> crate::Result<InvokeResponse> {
    match self {
      #[allow(unused_variables)]
      Self::CliMatches => {
        #[cfg(cli)]
        return tauri_api::cli::get_matches(&context.config)
          .map_err(Into::into)
          .map(Into::into);
        #[cfg(not(cli))]
          Err(crate::Error::ApiNotEnabled(
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri.cli)".to_string(),
          ))
      }
    }
  }
}
