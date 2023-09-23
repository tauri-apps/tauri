// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::borrow::Cow;

use http::{
  header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
  HeaderValue, Method, StatusCode,
};

use crate::{
  manager::WindowManager,
  window::{InvokeRequest, UriSchemeProtocolHandler},
  Runtime,
};

use super::{CallbackFn, InvokeBody, InvokeResponse};

const TAURI_CALLBACK_HEADER_NAME: &str = "Tauri-Callback";
const TAURI_ERROR_HEADER_NAME: &str = "Tauri-Error";

#[cfg(any(target_os = "macos", target_os = "ios", not(ipc_custom_protocol)))]
pub fn message_handler<R: Runtime>(
  manager: WindowManager<R>,
) -> crate::runtime::webview::WebviewIpcHandler<crate::EventLoopMessage, R> {
  Box::new(move |window, request| handle_ipc_message(request, &manager, &window.label))
}

pub fn get<R: Runtime>(manager: WindowManager<R>, label: String) -> UriSchemeProtocolHandler {
  Box::new(move |request, responder| {
    let manager = manager.clone();
    let label = label.clone();

    let respond = move |mut response: http::Response<Cow<'static, [u8]>>| {
      response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
      responder.respond(response);
    };

    match *request.method() {
      Method::POST => {
        if let Some(window) = manager.get_window(&label) {
          match parse_invoke_request(&manager, request) {
            Ok(request) => {
              window.on_message(
                request,
                Box::new(move |_window, _cmd, response, _callback, _error| {
                  let (mut response, mime_type) = match response {
                    InvokeResponse::Ok(InvokeBody::Json(v)) => (
                      http::Response::new(serde_json::to_vec(&v).unwrap().into()),
                      mime::APPLICATION_JSON,
                    ),
                    InvokeResponse::Ok(InvokeBody::Raw(v)) => (
                      http::Response::new(v.into()),
                      mime::APPLICATION_OCTET_STREAM,
                    ),
                    InvokeResponse::Err(e) => {
                      let mut response =
                        http::Response::new(serde_json::to_vec(&e.0).unwrap().into());
                      *response.status_mut() = StatusCode::BAD_REQUEST;
                      (response, mime::TEXT_PLAIN)
                    }
                  };

                  response.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.essence_str()).unwrap(),
                  );

                  respond(response);
                }),
              );
            }
            Err(e) => {
              respond(
                http::Response::builder()
                  .status(StatusCode::BAD_REQUEST)
                  .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
                  .body(e.as_bytes().to_vec().into())
                  .unwrap(),
              );
            }
          }
        } else {
          respond(
            http::Response::builder()
              .status(StatusCode::BAD_REQUEST)
              .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
              .body(
                "failed to acquire window reference"
                  .as_bytes()
                  .to_vec()
                  .into(),
              )
              .unwrap(),
          );
        }
      }

      Method::OPTIONS => {
        let mut r = http::Response::new(Vec::new().into());
        r.headers_mut().insert(
          ACCESS_CONTROL_ALLOW_HEADERS,
          HeaderValue::from_static("Content-Type, Tauri-Callback, Tauri-Error, Tauri-Channel-Id"),
        );
        respond(r);
      }

      _ => {
        let mut r = http::Response::new(
          "only POST and OPTIONS are allowed"
            .as_bytes()
            .to_vec()
            .into(),
        );
        *r.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        r.headers_mut().insert(
          CONTENT_TYPE,
          HeaderValue::from_str(mime::TEXT_PLAIN.essence_str()).unwrap(),
        );
        respond(r);
      }
    }
  })
}

#[cfg(any(target_os = "macos", target_os = "ios", not(ipc_custom_protocol)))]
fn handle_ipc_message<R: Runtime>(message: String, manager: &WindowManager<R>, label: &str) {
  if let Some(window) = manager.get_window(label) {
    use serde::{Deserialize, Deserializer};

    pub(crate) struct HeaderMap(http::HeaderMap);

    impl<'de> Deserialize<'de> for HeaderMap {
      fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        let map = std::collections::HashMap::<String, String>::deserialize(deserializer)?;
        let mut headers = http::HeaderMap::default();
        for (key, value) in map {
          if let (Ok(key), Ok(value)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value),
          ) {
            headers.insert(key, value);
          } else {
            return Err(serde::de::Error::custom(format!(
              "invalid header `{key}` `{value}`"
            )));
          }
        }
        Ok(Self(headers))
      }
    }

    #[derive(Deserialize)]
    struct RequestOptions {
      headers: HeaderMap,
    }

    #[derive(Deserialize)]
    struct Message {
      cmd: String,
      callback: CallbackFn,
      error: CallbackFn,
      payload: serde_json::Value,
      options: Option<RequestOptions>,
    }

    #[allow(unused_mut)]
    let mut invoke_message: Option<crate::Result<Message>> = None;

    #[cfg(feature = "isolation")]
    {
      #[derive(serde::Deserialize)]
      struct IsolationMessage<'a> {
        cmd: String,
        callback: CallbackFn,
        error: CallbackFn,
        payload: crate::utils::pattern::isolation::RawIsolationPayload<'a>,
        options: Option<RequestOptions>,
      }

      if let crate::Pattern::Isolation { crypto_keys, .. } = manager.pattern() {
        invoke_message.replace(
          serde_json::from_str::<IsolationMessage<'_>>(&message)
            .map_err(Into::into)
            .and_then(|message| {
              Ok(Message {
                cmd: message.cmd,
                callback: message.callback,
                error: message.error,
                payload: serde_json::from_slice(&crypto_keys.decrypt(message.payload)?)?,
                options: message.options,
              })
            }),
        );
      }
    }

    match invoke_message
      .unwrap_or_else(|| serde_json::from_str::<Message>(&message).map_err(Into::into))
    {
      Ok(message) => {
        window.on_message(
          InvokeRequest {
            cmd: message.cmd,
            callback: message.callback,
            error: message.error,
            body: message.payload.into(),
            headers: message.options.map(|o| o.headers.0).unwrap_or_default(),
          },
          Box::new(move |window, cmd, response, callback, error| {
            use crate::ipc::{
              format_callback::{
                format as format_callback, format_result as format_callback_result,
              },
              Channel,
            };
            use serde_json::Value as JsonValue;

            // the channel data command is the only command that uses a custom protocol on Linux
            if window.manager.invoke_responder().is_none()
              && cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
            {
              fn responder_eval<R: Runtime>(
                window: &crate::Window<R>,
                js: crate::Result<String>,
                error: CallbackFn,
              ) {
                let eval_js = match js {
                  Ok(js) => js,
                  Err(e) => format_callback(error, &e.to_string())
                    .expect("unable to serialize response error string to json"),
                };

                let _ = window.eval(&eval_js);
              }

              match &response {
                InvokeResponse::Ok(InvokeBody::Json(v)) => {
                  if !(cfg!(target_os = "macos") || cfg!(target_os = "ios"))
                    && matches!(v, JsonValue::Object(_) | JsonValue::Array(_))
                  {
                    let _ = Channel::from_ipc(window, callback).send(v);
                  } else {
                    responder_eval(
                      &window,
                      format_callback_result(Result::<_, ()>::Ok(v), callback, error),
                      error,
                    )
                  }
                }
                InvokeResponse::Ok(InvokeBody::Raw(v)) => {
                  if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                    responder_eval(
                      &window,
                      format_callback_result(Result::<_, ()>::Ok(v), callback, error),
                      error,
                    );
                  } else {
                    let _ = Channel::from_ipc(window, callback).send(InvokeBody::Raw(v.clone()));
                  }
                }
                InvokeResponse::Err(e) => responder_eval(
                  &window,
                  format_callback_result(Result::<(), _>::Err(&e.0), callback, error),
                  error,
                ),
              }
            }
          }),
        );
      }
      Err(e) => {
        let _ = window.eval(&format!(
          r#"console.error({})"#,
          serde_json::Value::String(e.to_string())
        ));
      }
    }
  }
}

fn parse_invoke_request<R: Runtime>(
  #[allow(unused_variables)] manager: &WindowManager<R>,
  request: http::Request<Vec<u8>>,
) -> std::result::Result<InvokeRequest, String> {
  #[allow(unused_mut)]
  let (parts, mut body) = request.into_parts();

  // skip leading `/`
  let cmd = percent_encoding::percent_decode(parts.uri.path()[1..].as_bytes())
    .decode_utf8_lossy()
    .to_string();

  // the body is not set if ipc_custom_protocol is not enabled so we'll just ignore it
  #[cfg(all(feature = "isolation", ipc_custom_protocol))]
  if let crate::Pattern::Isolation { crypto_keys, .. } = manager.pattern() {
    body = crate::utils::pattern::isolation::RawIsolationPayload::try_from(&body)
      .and_then(|raw| crypto_keys.decrypt(raw))
      .map_err(|e| e.to_string())?;
  }

  let callback = CallbackFn(
    parts
      .headers
      .get(TAURI_CALLBACK_HEADER_NAME)
      .ok_or("missing Tauri-Callback header")?
      .to_str()
      .map_err(|_| "Tauri callback header value must be a string")?
      .parse()
      .map_err(|_| "Tauri callback header value must be a numeric string")?,
  );
  let error = CallbackFn(
    parts
      .headers
      .get(TAURI_ERROR_HEADER_NAME)
      .ok_or("missing Tauri-Error header")?
      .to_str()
      .map_err(|_| "Tauri error header value must be a string")?
      .parse()
      .map_err(|_| "Tauri error header value must be a numeric string")?,
  );

  let content_type = parts
    .headers
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|h| h.to_str().ok())
    .map(|mime| mime.parse())
    .unwrap_or(Ok(mime::APPLICATION_OCTET_STREAM))
    .map_err(|_| "unknown content type")?;
  let body = if content_type == mime::APPLICATION_OCTET_STREAM {
    body.into()
  } else if content_type == mime::APPLICATION_JSON {
    if cfg!(ipc_custom_protocol) {
      serde_json::from_slice::<serde_json::Value>(&body)
        .map_err(|e| e.to_string())?
        .into()
    } else {
      // the body is not set if ipc_custom_protocol is not enabled so we'll just ignore it
      serde_json::Value::Object(Default::default()).into()
    }
  } else {
    return Err(format!("content type {content_type} is not implemented"));
  };

  let payload = InvokeRequest {
    cmd,
    callback,
    error,
    body,
    headers: parts.headers,
  };

  Ok(payload)
}
