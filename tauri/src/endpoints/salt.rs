use crate::{Webview, WebviewMut};

/// Validates a salt.
pub fn validate<W: WebviewMut>(
  webview: &mut W,
  salt: String,
  callback: String,
  error: String,
) -> crate::Result<()> {
  let response = if crate::salt::is_valid(salt) {
    Ok("Valid")
  } else {
    Err("Invalid salt")
  };
  let callback_string = crate::api::rpc::format_callback_result(response, callback, error)?;
  webview.dispatch(move |w| {
    w.eval(callback_string.as_str());
  })?;
  Ok(())
}
