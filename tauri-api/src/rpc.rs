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
  use quickcheck_macros::quickcheck;

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn qc_formating(f: String, a: String) -> bool {
    // can not accept empty strings
    if f != "" && a != "" {
      // get length of function and argument
      let alen = &a.len();
      let flen = &f.len();
      // call format callback
      let fc = format_callback(f, a);
      // get length of the resulting string
      let fclen = fc.len();

      // if formatted string equals the length of the argument and the function plus 12 then its correct.
      fclen == alen + flen + 12
    } else {
      true
    }
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn qc_format_res(result: Result<String, String>, c: String, ec: String) -> bool {
    // match on result to decide how to call the function.
    match result {
      // if ok, get length of result and callback strings.
      Ok(r) => {
        let rlen = r.len();
        let clen = c.len();

        // take the ok string from result and pass it into format_callback_result as an ok.
        let resp = format_callback_result(Ok(r), c, ec);
        // get response string length
        let reslen = resp.len();

        // if response string length equals result and callback length plus 12 characters then it is correct.
        reslen == rlen + clen + 12
      }
      // If Err, get length of Err and Error callback
      Err(err) => {
        let eclen = ec.len();
        let errlen = err.len();
        // pass err as Err into format_callback_result with callback and error callback
        let resp = format_callback_result(Err(err), c, ec);
        // get response string length
        let reslen = resp.len();

        // if length of response string equals the error length and the error callback length plus 14 characters then its is correct.
        reslen == eclen + errlen + 14
      }
    }
  }
}
