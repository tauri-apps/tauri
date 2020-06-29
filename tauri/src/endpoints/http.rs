use tauri_api::http::{make_request as request, HttpRequestOptions};
use web_view::WebView;

/// Makes an HTTP request and resolves the response to the webview
pub fn make_request<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(webview, move || request(options), callback, error);
}
