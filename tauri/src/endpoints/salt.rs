use web_view::WebView;

/// Validates a salt.
pub fn validate<T: 'static>(
  webview: &mut WebView<'_, T>,
  salt: String,
  callback: String,
  error: String,
) {
  let response = if crate::salt::is_valid(salt) {
    Ok("'VALID'".to_string())
  } else {
    Err("'INVALID SALT'".to_string())
  };
  let callback_string = crate::api::rpc::format_callback_result(response, callback, error);
  webview
    .eval(callback_string.as_str())
    .expect("Failed to eval JS from validate()");
}
