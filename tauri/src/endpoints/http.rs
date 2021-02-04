use tauri_api::http::{make_request as request, HttpRequestOptions};
use webview_official::WebviewMut;

/// Makes an HTTP request and resolves the response to the webview
pub async fn make_request(
  webview: &mut WebviewMut,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(webview, async move { request(options) }, callback, error).await;
}
