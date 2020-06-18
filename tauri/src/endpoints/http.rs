use tauri_api::http::{make_request as request, HttpRequestOptions, ResponseType};
use web_view::WebView;

/// Makes a HTTP request and resolves the response to the webview
pub fn make_request<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: HttpRequestOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let response_type = options.response_type.clone();
      request(options).map(
        |response| match response_type.unwrap_or(ResponseType::Json) {
          ResponseType::Text => format!(r#""{}""#, response),
          _ => response,
        },
      )
    },
    callback,
    error,
  );
}
