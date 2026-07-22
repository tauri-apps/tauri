// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Custom URI scheme protocols. A host registers a scheme on the builder; every
//! request the webviews make to it is delivered on the event queue as a
//! `{"type":"uri-scheme-request",...}` message carrying an opaque request handle,
//! which the host answers with [`tauri_uri_scheme_respond`] from any thread —
//! the same split as invokes and their resolvers.
//!
//! Backed by `Builder::register_asynchronous_uri_scheme_protocol`, so the
//! webview thread is never blocked waiting for the host to produce a response.

use std::collections::BTreeMap;
use std::os::raw::c_char;

use base64::Engine;
use tauri::{http, UriSchemeContext, UriSchemeResponder};

use crate::error::{catch, fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::state::{self, Entry};
use crate::try_cstr;
use crate::Rt as TauriRuntime;

/// Registers a custom URI scheme served to every webview of the app.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_builder_register_uri_scheme_protocol(
  builder: u64,
  scheme: *const c_char,
) -> i32 {
  catch(|| {
    let scheme = try_cstr!(scheme);
    if scheme.is_empty() {
      return fail(ERR_INVALID_ARG, "uri scheme is empty");
    }
    state::with_builder(builder, |b| {
      b.uri_scheme_protocols.insert(scheme.to_string());
      OK
    })
  })
}

/// Answers a pending custom-protocol request, consuming its handle. `status` is
/// an HTTP status code, `headers_json` a JSON object mapping header names to a
/// string or an array of strings (may be null or empty), and `body`/`body_len`
/// the raw response body (`body` may be null for an empty body).
///
/// A request that is never answered leaves the webview waiting forever, so every
/// `uri-scheme-request` message must end here (an error status is still an
/// answer). Callable from any thread.
#[no_mangle]
pub unsafe extern "C" fn tauri_uri_scheme_respond(
  request: u64,
  status: u32,
  headers_json: *const c_char,
  body: *const u8,
  body_len: u32,
) -> i32 {
  catch(|| {
    // Everything is validated before the handle is consumed, so a malformed
    // response can be corrected and retried with the same request handle.
    let status = match u16::try_from(status)
      .ok()
      .and_then(|status| http::StatusCode::from_u16(status).ok())
    {
      Some(status) => status,
      None => return fail(ERR_INVALID_ARG, format!("invalid HTTP status `{status}`")),
    };
    let headers = match unsafe { parse_headers(headers_json) } {
      Ok(headers) => headers,
      Err(code) => return code,
    };
    let body = if body.is_null() {
      Vec::new()
    } else {
      unsafe { std::slice::from_raw_parts(body, body_len as usize) }.to_vec()
    };

    let Some(Entry::UriSchemeResponder(responder)) = state::remove(request) else {
      return fail(ERR_INVALID_HANDLE, "invalid uri scheme request handle");
    };
    let mut response = http::Response::builder().status(status);
    for (name, value) in headers {
      response = response.header(name, value);
    }
    match response.body(body) {
      Ok(response) => {
        responder.respond(response);
        OK
      }
      // The responder is dropped here: the request stays unanswered, but its
      // handle is already gone, so surface the reason to the host.
      Err(e) => fail(ERR_GENERIC, format!("invalid response: {e}")),
    }
  })
}

/// `{"a":"b","c":["d","e"]}` → `[("a","b"), ("c","d"), ("c","e")]`. A null or
/// empty string means no headers.
unsafe fn parse_headers(json: *const c_char) -> Result<Vec<(String, String)>, i32> {
  if json.is_null() {
    return Ok(Vec::new());
  }
  let json = unsafe { crate::cstr(json) }?;
  if json.trim().is_empty() || json.trim() == "null" {
    return Ok(Vec::new());
  }
  let map: BTreeMap<String, serde_json::Value> = serde_json::from_str(json)
    .map_err(|e| fail(ERR_INVALID_ARG, format!("invalid response headers: {e}")))?;

  let mut headers = Vec::new();
  for (name, value) in map {
    match value {
      serde_json::Value::String(value) => headers.push((name, value)),
      serde_json::Value::Array(values) => {
        for value in values {
          match value {
            serde_json::Value::String(value) => headers.push((name.clone(), value)),
            _ => {
              return Err(fail(
                ERR_INVALID_ARG,
                format!("header `{name}` has a non-string value"),
              ))
            }
          }
        }
      }
      _ => {
        return Err(fail(
          ERR_INVALID_ARG,
          format!("header `{name}` must be a string or an array of strings"),
        ))
      }
    }
  }
  Ok(headers)
}

/// Registers the request's responder and serializes the request into a queue
/// message. The body is base64 — the queue carries JSON only, and protocol
/// requests (unlike responses) can arrive with arbitrary bytes.
pub(crate) fn uri_scheme_request_json(
  scheme: &str,
  ctx: &UriSchemeContext<'_, TauriRuntime>,
  request: http::Request<Vec<u8>>,
  responder: UriSchemeResponder,
) -> String {
  let (parts, body) = request.into_parts();
  let headers: serde_json::Map<String, serde_json::Value> = parts
    .headers
    .iter()
    .map(|(name, value)| {
      (
        name.as_str().to_string(),
        serde_json::Value::String(value.to_str().unwrap_or_default().to_string()),
      )
    })
    .collect();
  let request_id = state::insert(Entry::UriSchemeResponder(responder));
  serde_json::json!({
    "type": "uri-scheme-request",
    "id": request_id,
    "scheme": scheme,
    "webview": ctx.webview_label(),
    "method": parts.method.as_str(),
    "uri": parts.uri.to_string(),
    "headers": headers,
    "body": base64::engine::general_purpose::STANDARD.encode(&body),
  })
  .to_string()
}
