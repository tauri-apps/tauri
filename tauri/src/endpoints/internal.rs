use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  ValidateSalt {
    salt: String,
    callback: String,
    error: String,
  },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<()> {
    match self {
      Self::ValidateSalt {
        salt,
        callback,
        error,
      } => {
        validate_salt(webview_manager, salt, callback, error)?;
      }
    }
    Ok(())
  }
}

/// Validates a salt.
pub fn validate_salt<D: crate::ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
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
  webview_manager
    .current_webview()?
    .eval(callback_string.as_str());
  Ok(())
}
