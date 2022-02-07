// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to Inter Procedure Call(IPC).
//!
//! This module includes utilities to send messages to the JS layer of the webview.

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// The `Callback` type is the return value of the `transformCallback` JavaScript function.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct CallbackFn(pub usize);

/// The information about this is quite limited. On Chrome/Edge and Firefox, [the maximum string size is approximately 1 GB](https://stackoverflow.com/a/34958490).
///
/// [From MDN:](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length#description)
///
/// ECMAScript 2016 (ed. 7) established a maximum length of 2^53 - 1 elements. Previously, no maximum length was specified.
///
/// In Firefox, strings have a maximum length of 2\*\*30 - 2 (~1GB). In versions prior to Firefox 65, the maximum length was 2\*\*28 - 1 (~256MB).
const MAX_JSON_STR_LEN: usize = usize::pow(2, 30) - 2;

/// Minimum size JSON needs to be in order to convert it to JSON.parse with [`escape_json_parse`].
// TODO: this number should be benchmarked and checked for optimal range, I set 10 KiB arbitrarily
// we don't want to lose the gained object parsing time to extra allocations preparing it
const MIN_JSON_PARSE_LEN: usize = 10_240;

fn escape(json: &RawValue) -> String {
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
pub fn escape_json_parse<T: Serialize>(value: &T) -> crate::api::Result<String> {
  // get a raw &str representation of a serialized json value.
  let string = serde_json::to_string(value)?;
  let raw = RawValue::from_string(string)?;

  // from here we know json.len() > 1 because an empty string is not a valid json value.
  let json = raw.get();
  let first = json.as_bytes()[0];

  #[cfg(debug_assertions)]
  if first == b'"' {
    assert!(
      json.len() < MAX_JSON_STR_LEN,
      "passing a string larger than the max JavaScript literal string size"
    )
  }

  let escaped = if json.len() > MIN_JSON_PARSE_LEN && (first == b'{' || first == b'[') {
    let escaped = escape(&raw);
    // only use JSON.parse('{arg}') for arrays and objects less than the limit
    // smaller literals do not benefit from being parsed from json
    if escaped.len() < MAX_JSON_STR_LEN {
      escaped
    } else {
      json.into()
    }
  } else {
    json.into()
  };

  Ok(escaped)
}

/// Formats a function name and argument to be evaluated as callback.
///
/// This will serialize primitive JSON types (e.g. booleans, strings, numbers, etc.) as JavaScript literals,
/// but will serialize arrays and objects whose serialized JSON string is smaller than 1 GB and larger
/// than 10 KiB with `JSON.parse('...')`.
/// See [json-parse-benchmark](https://github.com/GoogleChromeLabs/json-parse-benchmark).
///
/// # Examples
/// - With string literals:
/// ```
/// use tauri::api::ipc::{CallbackFn, format_callback};
/// // callback with a string argument
/// let cb = format_callback(CallbackFn(12345), &"the string response").unwrap();
/// assert!(cb.contains(r#"window["_12345"]("the string response")"#));
/// ```
///
/// - With types implement [`serde::Serialize`]:
/// ```
/// use tauri::api::ipc::{CallbackFn, format_callback};
/// use serde::Serialize;
///
/// // callback with large JSON argument
/// #[derive(Serialize)]
/// struct MyResponse {
///   value: String
/// }
///
/// let cb = format_callback(
///   CallbackFn(6789),
///   &MyResponse { value: String::from_utf8(vec![b'X'; 10_240]).unwrap()
/// }).expect("failed to serialize");
///
/// assert!(cb.contains(r#"window["_6789"](JSON.parse('{"value":"XXXXXXXXX"#));
/// ```
pub fn format_callback<T: Serialize>(
  function_name: CallbackFn,
  arg: &T,
) -> crate::api::Result<String> {
  Ok(format!(
    r#"
    if (window["_{fn}"]) {{
      window["_{fn}"]({arg})
    }} else {{
      console.warn("[TAURI] Couldn't find callback id {fn} in window. This happens when the app is reloaded while Rust is running an asynchronous operation.")
    }}"#,
    fn = function_name.0,
    arg = escape_json_parse(arg)?
  ))
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
/// use tauri::api::ipc::{CallbackFn, format_callback_result};
/// let res: Result<u8, &str> = Ok(5);
/// let cb = format_callback_result(res, CallbackFn(145), CallbackFn(0)).expect("failed to format");
/// assert!(cb.contains(r#"window["_145"](5)"#));
///
/// let res: Result<&str, &str> = Err("error message here");
/// let cb = format_callback_result(res, CallbackFn(2), CallbackFn(1)).expect("failed to format");
/// assert!(cb.contains(r#"window["_1"]("error message here")"#));
/// ```
// TODO: better example to explain
pub fn format_callback_result<T: Serialize, E: Serialize>(
  result: Result<T, E>,
  success_callback: CallbackFn,
  error_callback: CallbackFn,
) -> crate::api::Result<String> {
  match result {
    Ok(res) => format_callback(success_callback, &res),
    Err(err) => format_callback(error_callback, &err),
  }
}

#[cfg(test)]
mod test {
  use crate::api::ipc::*;
  use quickcheck::{Arbitrary, Gen};
  use quickcheck_macros::quickcheck;

  impl Arbitrary for CallbackFn {
    fn arbitrary(g: &mut Gen) -> CallbackFn {
      CallbackFn(usize::arbitrary(g))
    }
  }

  #[test]
  fn test_escape_json_parse() {
    assert_eq!(escape_json_parse(&()).unwrap(), "null");
    assert_eq!(escape_json_parse(&5i32).unwrap(), "5");

    #[derive(serde::Serialize)]
    struct JsonObj {
      value: String,
    }

    let raw_str = "T".repeat(MIN_JSON_PARSE_LEN);
    assert_eq!(
      escape_json_parse(&raw_str).unwrap(),
      format!("\"{}\"", raw_str)
    );

    assert_eq!(
      escape_json_parse(&JsonObj {
        value: raw_str.clone()
      })
      .unwrap(),
      format!("JSON.parse('{{\"value\":\"{}\"}}')", raw_str)
    );

    assert_eq!(
      escape_json_parse(&JsonObj {
        value: format!("\"{}\"", raw_str)
      })
      .unwrap(),
      format!("JSON.parse('{{\"value\":\"\\\\\"{}\\\\\"\"}}')", raw_str)
    );

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
    let escape_single_quoted_json_test = escape(&dangerous_json);

    let result = r#"JSON.parse('{"test":"don\\\\🚀🐱‍👤\\\\\'t forget to escape me!🚀🐱‍👤","te🚀🐱‍👤st2":"don\'t forget to escape me!","test3":"\\\\🚀🐱‍👤\\\\\\\\\'\'\'\\\\\\\\🚀🐱‍👤\\\\\\\\🚀🐱‍👤\\\\\'\'\'\'\'"}')"#;
    assert_eq!(definitely_escaped_dangerous_json, result);
    assert_eq!(escape_single_quoted_json_test, result);
  }

  // check abritrary strings in the format callback function
  #[quickcheck]
  fn qc_formating(f: CallbackFn, a: String) -> bool {
    // call format callback
    let fc = format_callback(f, &a).unwrap();
    fc.contains(&format!(
      r#"window["_{}"](JSON.parse('{}'))"#,
      f.0,
      serde_json::Value::String(a.clone()),
    )) || fc.contains(&format!(
      r#"window["_{}"]({})"#,
      f.0,
      serde_json::Value::String(a),
    ))
  }

  // check arbitrary strings in format_callback_result
  #[quickcheck]
  fn qc_format_res(result: Result<String, String>, c: CallbackFn, ec: CallbackFn) -> bool {
    let resp =
      format_callback_result(result.clone(), c, ec).expect("failed to format callback result");
    let (function, value) = match result {
      Ok(v) => (c, v),
      Err(e) => (ec, e),
    };

    resp.contains(&format!(
      r#"window["_{}"]({})"#,
      function.0,
      serde_json::Value::String(value),
    ))
  }
}
