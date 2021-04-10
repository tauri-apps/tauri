// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;
use serde_json::value::RawValue;

/// The information about this is quite limited. On Chrome/Edge and Firefox, [the maximum string size is approximately 1 GB](https://stackoverflow.com/a/34958490).
///
/// [From MDN:](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length#description)
///
/// ECMAScript 2016 (ed. 7) established a maximum length of 2^53 - 1 elements. Previously, no maximum length was specified.
///
/// In Firefox, strings have a maximum length of 2\*\*30 - 2 (~1GB). In versions prior to Firefox 65, the maximum length was 2\*\*28 - 1 (~256MB).
const MAX_JSON_STR_LEN: usize = usize::pow(2, 30) - 2;

/// Minimum size JSON needs to be in order to convert it to JSON.parse with [`escape_json_parse`].
// todo: this number should be benchmarked and checked for optimal range, I set 10 KiB arbitrarily
// we don't want to lose the gained object parsing time to extra allocations preparing it
const MIN_JSON_PARSE_LEN: usize = 10_240;

/// Transforms & escapes a JSON String -> JSON.parse('{json}')
///
/// Single quotes chosen because double quotes are already used in JSON. With single quotes, we only
/// need to escape strings that include backslashes or single quotes. If we used double quotes, then
/// there would be no cases that a string doesn't need escaping.
///
/// # Safety
///
/// The ability to safely escape JSON into a JSON.parse('{json}') relies entirely on 2 things.
///
/// 1. `serde_json`'s ability to correctly escape and format json into a string.
/// 2. JavaScript engines not accepting anything except another unescaped, literal single quote
///     character to end a string that was opened with it.
fn escape_json_parse(json: &RawValue) -> String {
  let json = json.get();

  // 14 chars in JSON.parse('')
  // todo: should we increase the 14 by x to allow x amount of escapes before another allocation?
  let mut s = String::with_capacity(json.len() + 14);
  s.push_str("JSON.parse('");

  // insert a backslash before any backslash or single quote characters.
  let mut last = 0;
  for (idx, _) in json.match_indices(|c| c == '\\' || c == '\'') {
    s.push_str(&json[last..idx]);
    s.push('\\');
    last = idx;
  }

  // finish appending the trailing characters that don't need escaping
  s.push_str(&json[last..]);
  s.push_str("')");
  s
}

/// Formats a function name and argument to be evaluated as callback.
///
/// This will serialize primitive JSON types (e.g. booleans, strings, numbers, etc.) as JavaScript literals,
/// but will serialize arrays and objects whose serialized JSON string is smaller than 1 GB and larger
/// than 10 KiB with `JSON.parse('...')`.
/// https://github.com/GoogleChromeLabs/json-parse-benchmark
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback;
/// // callback with a string argument
/// let cb = format_callback("callback-function-name", &"the string response").expect("failed to serialize");
/// assert!(cb.contains(r#"window["callback-function-name"]("the string response")"#));
/// ```
///
/// ```
/// use tauri_api::rpc::format_callback;
/// use serde::Serialize;
///
/// // callback with large JSON argument
/// #[derive(Serialize)]
/// struct MyResponse {
///   value: String
/// }
///
/// let cb = format_callback("callback-function-name", &MyResponse { value: String::from_utf8(vec![b'X'; 10_240]).unwrap()})
///   .expect("failed to serialize");
///
/// assert!(cb.contains(r#"window["callback-function-name"](JSON.parse('{"value":"XXXXXXXXX"#));
/// ```
pub fn format_callback<T: Serialize, S: AsRef<str>>(
  function_name: S,
  arg: &T,
) -> crate::Result<String> {
  macro_rules! format_callback {
    ( $arg:expr ) => {
      format!(
        r#"
          if (window["{fn}"]) {{
            window["{fn}"]({arg})
          }} else {{
            console.warn("[TAURI] Couldn't find callback id {fn} in window. This happens when the app is reloaded while Rust is running an asynchronous operation.")
          }}
        "#,
        fn = function_name.as_ref(),
        arg = $arg
      )
    }
  }

  // get a raw &str representation of a serialized json value.
  let string = serde_json::to_string(arg)?;
  let raw = RawValue::from_string(string)?;

  // from here we know json.len() > 1 because an empty string is not a valid json value.
  let json = raw.get();
  let first = json.as_bytes()[0];

  #[cfg(debug_assertions)]
  if first == b'"' {
    debug_assert!(
      json.len() < MAX_JSON_STR_LEN,
      "passing a callback string larger than the max JavaScript literal string size"
    )
  }

  // only use JSON.parse('{arg}') for arrays and objects less than the limit
  // smaller literals do not benefit from being parsed from json
  Ok(
    if json.len() > MIN_JSON_PARSE_LEN && (first == b'{' || first == b'[') {
      let escaped = escape_json_parse(&raw);
      if escaped.len() < MAX_JSON_STR_LEN {
        format_callback!(escaped)
      } else {
        format_callback!(json)
      }
    } else {
      format_callback!(json)
    },
  )
}

/// Formats a Result type to its Promise response.
/// Useful for Promises handling.
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
///
/// * `result` the Result to check
/// * `success_callback` the function name of the Ok callback. Usually the `resolve` of the JS Promise.
/// * `error_callback` the function name of the Err callback. Usually the `reject` of the JS Promise.
///
/// Note that the callback strings are automatically generated by the `invoke` helper.
///
/// # Examples
/// ```
/// use tauri_api::rpc::format_callback_result;
/// let res: Result<u8, &str> = Ok(5);
/// let cb = format_callback_result(res, "success_cb", "error_cb").expect("failed to format");
/// assert!(cb.contains(r#"window["success_cb"](5)"#));
///
/// let res: Result<&str, &str> = Err("error message here");
/// let cb = format_callback_result(res, "success_cb", "error_cb").expect("failed to format");
/// assert!(cb.contains(r#"window["error_cb"]("error message here")"#));
/// ```
pub fn format_callback_result<T: Serialize, E: Serialize>(
  result: Result<T, E>,
  success_callback: impl AsRef<str>,
  error_callback: impl AsRef<str>,
) -> crate::Result<String> {
  match result {
    Ok(res) => format_callback(success_callback, &res),
    Err(err) => format_callback(error_callback, &err),
  }
}

#[cfg(test)]
mod test {
  use crate::rpc::*;
  use quickcheck_macros::quickcheck;

  #[test]
  fn test_escape_json_parse() {
    let dangerous_json = RawValue::from_string(
      r#"{"test":"don\\🚀🐱‍👤\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don't forget to escape me!","test3":"\\🚀🐱‍👤\\\\'''\\\\🚀🐱‍👤\\\\🚀🐱‍👤\\'''''"}"#.into()
    ).unwrap();

    let definitely_escaped_dangerous_json = format!(
      "JSON.parse('{}')",
      dangerous_json
        .get()
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
    );
    let escape_single_quoted_json_test = escape_json_parse(&dangerous_json);

    let result = r#"JSON.parse('{"test":"don\\\\🚀🐱‍👤\\\\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don\'t forget to escape me!","test3":"\\\\🚀🐱‍👤\\\\\\\\\'\'\'\\\\\\\\🚀🐱‍👤\\\\\\\\🚀🐱‍👤\\\\\'\'\'\'\'"}')"#;
    assert_eq!(definitely_escaped_dangerous_json, result);
    assert_eq!(escape_single_quoted_json_test, result);
  }

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn qc_formating(f: String, a: String) -> bool {
    // can not accept empty strings
    if !f.is_empty() && !a.is_empty() {
      // call format callback
      let fc = format_callback(f.clone(), &a).unwrap();
      fc.contains(&format!(
        r#"window["{}"](JSON.parse('{}'))"#,
        f,
        serde_json::Value::String(a.clone()),
      )) || fc.contains(&format!(
        r#"window["{}"]({})"#,
        f,
        serde_json::Value::String(a),
      ))
    } else {
      true
    }
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn qc_format_res(result: Result<String, String>, c: String, ec: String) -> bool {
    let resp = format_callback_result(result.clone(), c.clone(), ec.clone())
      .expect("failed to format callback result");
    let (function, value) = match result {
      Ok(v) => (c, v),
      Err(e) => (ec, e),
    };

    resp.contains(&format!(
      r#"window["{}"]({})"#,
      function,
      serde_json::Value::String(value),
    ))
  }
}
