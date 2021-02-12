use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The set webview title API.
  SetTitle { title: String },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<()> {
    match self {
      Self::SetTitle { title } => {
        webview_manager.current_webview()?.set_title(&title);
        #[cfg(not(set_title))]
        throw_allowlist_error(webview_manager, "title");
      }
    }
    Ok(())
  }
}
