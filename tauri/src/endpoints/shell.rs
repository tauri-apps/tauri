use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
  /// The open URL in browser API
  Open { uri: String },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) {
    match self {
      Self::Execute {
        command,
        args,
        callback,
        error,
      } => {
        #[cfg(execute)]
        crate::call(webview_manager, command, args, callback, error).await;
        #[cfg(not(execute))]
        super::throw_allowlist_error(webview_manager, "execute");
      }
      Self::Open { uri } => {
        #[cfg(open)]
        open_browser(uri);
        #[cfg(not(open))]
        super::throw_allowlist_error(webview_manager, "open");
      }
    }
  }
}

#[cfg(open)]
pub fn open_browser(uri: String) {
  #[cfg(test)]
  assert!(uri.contains("http://"));

  #[cfg(not(test))]
  webbrowser::open(&uri).expect("Failed to open webbrowser with uri");
}
