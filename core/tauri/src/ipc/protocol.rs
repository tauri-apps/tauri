// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, sync::Arc};

use crate::{
  manager::AppManager,
  webview::{InvokeRequest, UriSchemeProtocolHandler},
  Runtime,
};
use http::{
  header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
  HeaderValue, Method, StatusCode,
};

use super::{CallbackFn, InvokeResponse, InvokeResponseBody};

const TAURI_CALLBACK_HEADER_NAME: &str = "Tauri-Callback";
const TAURI_ERROR_HEADER_NAME: &str = "Tauri-Error";

pub fn message_handler<R: Runtime>(
  manager: Arc<AppManager<R>>,
) -> crate::runtime::webview::WebviewIpcHandler<crate::EventLoopMessage, R> {
  Box::new(move |webview, request| handle_ipc_message(request, &manager, &webview.label))
}

pub fn get<R: Runtime>(manager: Arc<AppManager<R>>, label: String) -> UriSchemeProtocolHandler {
  Box::new(move |request, responder| {
    #[cfg(feature = "tracing")]
    let span = tracing::trace_span!(
      "ipc::request",
      kind = "custom-protocol",
      request = tracing::field::Empty
    )
    .entered();

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
        if let Some(webview) = manager.get_webview(&label) {
          match parse_invoke_request(&manager, request) {
            Ok(request) => {
              #[cfg(feature = "tracing")]
              span.record(
                "request",
                match &request.body {
                  InvokeBody::Json(j) => serde_json::to_string(j).unwrap(),
                  InvokeBody::Raw(b) => serde_json::to_string(b).unwrap(),
                },
              );
              #[cfg(feature = "tracing")]
              let request_span = tracing::trace_span!("ipc::request::handle", cmd = request.cmd);

              webview.on_message(
                request,
                Box::new(move |_webview, _cmd, response, _callback, _error| {
                  #[cfg(feature = "tracing")]
                  let _respond_span = tracing::trace_span!(
                    parent: &request_span,
                    "ipc::request::respond"
                  )
                  .entered();

                  #[cfg(feature = "tracing")]
                  let response_span = tracing::trace_span!(
                    "ipc::request::response",
                    response = serde_json::to_string(&response).unwrap(),
                    mime_type = tracing::field::Empty
                  )
                  .entered();

                  let (mut response, mime_type) = match response {
                    InvokeResponse::Ok(InvokeResponseBody::Json(v)) => (
                      http::Response::new(serde_json::to_vec(&v).unwrap().into()),
                      mime::APPLICATION_JSON,
                    ),
                    InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => (
                      http::Response::new(v.into()),
                      mime::APPLICATION_OCTET_STREAM,
                    ),
                    InvokeResponse::Err(e) => {
                      let mut response =
                        http::Response::new(serde_json::to_vec(&e.0).unwrap().into());
                      *response.status_mut() = StatusCode::BAD_REQUEST;
                      (response, mime::APPLICATION_JSON)
                    }
                  };

                  #[cfg(feature = "tracing")]
                  response_span.record("mime_type", mime_type.essence_str());

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
                "failed to acquire webview reference"
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

fn handle_ipc_message<R: Runtime>(message: String, manager: &AppManager<R>, label: &str) {
  if let Some(webview) = manager.get_webview(label) {
    #[cfg(feature = "tracing")]
    let _span =
      tracing::trace_span!("ipc::request", kind = "post-message", request = message).entered();

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
            http::header::HeaderName::from_bytes(key.as_bytes()),
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

      if let crate::Pattern::Isolation { crypto_keys, .. } = &*manager.pattern {
        #[cfg(feature = "tracing")]
        let _span = tracing::trace_span!("ipc::request::decrypt_isolation_payload").entered();

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

    let message = invoke_message.unwrap_or_else(|| {
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("ipc::request::deserialize").entered();
      serde_json::from_str::<Message>(&message).map_err(Into::into)
    });

    match message {
      Ok(message) => {
        let request = InvokeRequest {
          cmd: message.cmd,
          callback: message.callback,
          error: message.error,
          body: message.payload.into(),
          headers: message.options.map(|o| o.headers.0).unwrap_or_default(),
        };

        #[cfg(feature = "tracing")]
        let request_span = tracing::trace_span!("ipc::request::handle", cmd = request.cmd);

        webview.on_message(
          request,
          Box::new(move |webview, cmd, response, callback, error| {
            use crate::ipc::{
              format_callback::{
                format as format_callback, format_result as format_callback_result,
              },
              Channel,
            };
            use crate::sealed::ManagerBase;

            #[cfg(feature = "tracing")]
            let _respond_span = tracing::trace_span!(
              parent: &request_span,
              "ipc::request::respond"
            )
            .entered();

            // the channel data command is the only command that uses a custom protocol on Linux
            if webview.manager().webview.invoke_responder.is_none()
              && cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
            {
              fn responder_eval<R: Runtime>(
                webview: &crate::Webview<R>,
                js: crate::Result<String>,
                error: CallbackFn,
              ) {
                let eval_js = match js {
                  Ok(js) => js,
                  Err(e) => format_callback(error, &e.to_string())
                    .expect("unable to serialize response error string to json"),
                };

                let _ = webview.eval(&eval_js);
              }

              #[cfg(feature = "tracing")]
              let _response_span = tracing::trace_span!(
                "ipc::request::response",
                response = serde_json::to_string(&response).unwrap(),
                mime_type = match &response {
                  InvokeResponse::Ok(InvokeResponseBody::Json(_)) => mime::APPLICATION_JSON,
                  InvokeResponse::Ok(InvokeResponseBody::Raw(_)) => mime::APPLICATION_OCTET_STREAM,
                  InvokeResponse::Err(_) => mime::APPLICATION_JSON,
                }
                .essence_str()
              )
              .entered();

              match response {
                InvokeResponse::Ok(InvokeResponseBody::Json(v)) => {
                  if !(cfg!(target_os = "macos") || cfg!(target_os = "ios")) && v.len() > 4000 {
                    let _ = Channel::from_callback_fn(webview, callback).send(&v);
                  } else {
                    responder_eval(
                      &webview,
                      format_callback_result(Result::<_, ()>::Ok(v), callback, error),
                      error,
                    )
                  }
                }
                InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => {
                  if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                    responder_eval(
                      &webview,
                      format_callback_result(Result::<_, ()>::Ok(v), callback, error),
                      error,
                    );
                  } else {
                    let _ = Channel::from_callback_fn(webview, callback).send(v);
                  }
                }
                InvokeResponse::Err(e) => responder_eval(
                  &webview,
                  format_callback_result(Result::<(), _>::Err(&e.0), callback, error),
                  error,
                ),
              }
            }
          }),
        );
      }
      Err(e) => {
        #[cfg(feature = "tracing")]
        tracing::trace!("ipc.request.error {}", e);

        let _ = webview.eval(&format!(
          r#"console.error({})"#,
          serde_json::Value::String(e.to_string())
        ));
      }
    }
  }
}

fn parse_invoke_request<R: Runtime>(
  #[allow(unused_variables)] manager: &AppManager<R>,
  request: http::Request<Vec<u8>>,
) -> std::result::Result<InvokeRequest, String> {
  #[allow(unused_mut)]
  let (parts, mut body) = request.into_parts();

  // skip leading `/`
  let cmd = percent_encoding::percent_decode(parts.uri.path()[1..].as_bytes())
    .decode_utf8_lossy()
    .to_string();

  // on Android and on Linux (without the linux-ipc-protocol Cargo feature) we cannot read the request body
  // so we must ignore it because some commands use the IPC for faster response
  let has_payload = !body.is_empty();

  #[cfg(feature = "isolation")]
  if let crate::Pattern::Isolation { crypto_keys, .. } = &*manager.pattern {
    // if the platform does not support request body, we ignore it
    if has_payload {
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("ipc::request::decrypt_isolation_payload").entered();

      body = crate::utils::pattern::isolation::RawIsolationPayload::try_from(&body)
        .and_then(|raw| crypto_keys.decrypt(raw))
        .map_err(|e| e.to_string())?;
    }
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

  #[cfg(feature = "tracing")]
  let span = tracing::trace_span!("ipc::request::deserialize").entered();

  let body = if content_type == mime::APPLICATION_OCTET_STREAM {
    body.into()
  } else if content_type == mime::APPLICATION_JSON {
    // if the platform does not support request body, we ignore it
    if has_payload {
      serde_json::from_slice::<serde_json::Value>(&body)
        .map_err(|e| e.to_string())?
        .into()
    } else {
      serde_json::Value::Object(Default::default()).into()
    }
  } else {
    return Err(format!("content type {content_type} is not implemented"));
  };

  #[cfg(feature = "tracing")]
  drop(span);

  let payload = InvokeRequest {
    cmd,
    callback,
    error,
    body,
    headers: parts.headers,
  };

  Ok(payload)
}
