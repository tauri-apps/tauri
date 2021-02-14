use crate::ApplicationDispatcherExt;

use serde::Deserialize;
use tauri_api::http::{make_request as request, HttpRequestOptions};

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The HTTP request API.
  HttpRequest {
    options: Box<HttpRequestOptions>,
    callback: String,
    error: String,
  },
}

impl Cmd {
  pub async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) {
    match self {
      Self::HttpRequest {
        options,
        callback,
        error,
      } => {
        #[cfg(http_request)]
        make_request(webview_manager, *options, callback, error).await;
        #[cfg(not(http_request))]
        allowlist_error(webview_manager, error, "httpRequest");
      }
    }
  }
}

/// Makes an HTTP request and resolves the response to the webview
pub async fn make_request<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview_manager,
    async move { request(options).map_err(|e| e.into()) },
    callback,
    error,
  )
  .await;
}
