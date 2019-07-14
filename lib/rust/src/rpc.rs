pub fn format_callback(function_name: String, arg: String) -> String {
  let formatted_string = &format!("window[\"{}\"]({})", function_name, arg);
  return formatted_string.to_string();
}

pub fn format_callback_result(
  result: Result<String, String>,
  callback: String,
  error_callback: String,
) -> String {
  match result {
    Ok(res) => return format_callback(callback, res),
    Err(err) => return format_callback(error_callback, format!("\"{}\"", err)),
  }
}
