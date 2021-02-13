use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The get CLI matches API.
  CliMatches { callback: String, error: String },
}

impl Cmd {
  #[allow(unused_variables)]
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
    context: &crate::app::Context,
  ) {
    match self {
      #[allow(unused_variables)]
      Self::CliMatches { callback, error } => {
        #[cfg(cli)]
        {
          let matches = tauri_api::cli::get_matches(&context.config).map_err(|e| e.into());
          crate::execute_promise(webview_manager, async move { matches }, callback, error).await;
        }
        #[cfg(not(cli))]
          super::api_error(
            webview_manager,
            error,
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri.cli)",
          );
      }
    }
  }
}
