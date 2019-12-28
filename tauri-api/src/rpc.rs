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

#[cfg(test)]
mod test {
  use crate::rpc::*;

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn test_formating(f: String, a: String) -> bool {
    // can not accept empty strings
    if f != "" && a != "" {
      // get length of function and argument
      let alen = &a.len();
      let flen = &f.len();
      // call format callback
      let fc = format_callback(f, a);
      // get length of the resulting string
      let fclen = fc.len();

      println!("{}", fclen);

      // if formatted string equals the length of the argument and the function plus 12 then its correct.
      fclen == alen + flen + 12
    } else {
      true
    }
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn test_format_res(result: Result<String, String>, c: String, ec: String) -> bool {
    match result {
      Ok(r) => {
        let rlen = r.len();
        let clen = c.len();

        if r != "" && c != "" {
          let resp = format_callback_result(Ok(r), c, ec);
          let reslen = resp.len();
          reslen == rlen + clen + 12
        } else {
          true
        }
      }
      Err(err) => {
        if ec != "" {
          let eclen = ec.len();
          let errlen = err.len();
          let resp = format_callback_result(Err(err), c, ec);
          let reslen = resp.len();

          reslen == eclen + errlen + 14
        } else {
          true
        }
      }
    }
  }
}
