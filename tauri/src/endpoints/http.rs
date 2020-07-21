use tauri_api::http::{make_request as request, HttpRequestOptions};
use webview_official::Webview;

/// Makes an HTTP request and resolves the response to the webview
pub fn make_request(
  webview: &mut Webview,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(webview, move || request(options), callback, error);
}
