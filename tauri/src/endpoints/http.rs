use tauri_api::http::{make_request as request, HttpRequestOptions, ResponseType};
use web_view::WebView;

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
      request(options)
        .map_err(|e| crate::ErrorKind::Http(e.to_string()).into())
        .map(|response| {
          match response_type.unwrap_or(ResponseType::Json) {
            ResponseType::Text => format!(r#""{}""#, response),
            _ => response
          }
        })
    },
    callback,
    error,
  );
}
