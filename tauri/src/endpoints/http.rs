use crate::WebviewDispatcher;
use tauri_api::http::{make_request as request, HttpRequestOptions};

/// Makes an HTTP request and resolves the response to the webview
pub async fn make_request<W: WebviewDispatcher>(
  webview: &mut W,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(webview, async move { request(options) }, callback, error).await;
}
