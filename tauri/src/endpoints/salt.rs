use crate::ApplicationDispatcherExt;

/// Validates a salt.
pub fn validate<D: ApplicationDispatcherExt>(
  dispatcher: &mut D,
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
  dispatcher.eval(callback_string.as_str());
  Ok(())
}
