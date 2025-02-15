// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, sync::Arc};

use crate::{
  ipc::InvokeResponseBody,
  manager::AppManager,
  webview::{InvokeRequest, UriSchemeProtocolHandler},
  Runtime,
};
use http::{
  header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
    CONTENT_TYPE,
  },
  HeaderValue, Method, Request, StatusCode,
};
use url::Url;

use super::{CallbackFn, InvokeResponse};

const TAURI_CALLBACK_HEADER_NAME: &str = "Tauri-Callback";
const TAURI_ERROR_HEADER_NAME: &str = "Tauri-Error";
const TAURI_INVOKE_KEY_HEADER_NAME: &str = "Tauri-Invoke-Key";

const TAURI_RESPONSE_HEADER_NAME: &str = "Tauri-Response";
const TAURI_RESPONSE_HEADER_ERROR: &str = "error";
const TAURI_RESPONSE_HEADER_OK: &str = "ok";

pub fn message_handler<R: Runtime>(
  manager: Arc<AppManager<R>>,
) -> crate::runtime::webview::WebviewIpcHandler<crate::EventLoopMessage, R> {
  Box::new(move |webview, request| handle_ipc_message(request, &manager, &webview.label))
}

pub fn get<R: Runtime>(manager: Arc<AppManager<R>>) -> UriSchemeProtocolHandler {
  Box::new(move |label, request, responder| {
    #[cfg(feature = "tracing")]
    let span = tracing::trace_span!(
      "ipc::request",
      kind = "custom-protocol",
      request = tracing::field::Empty
    )
    .entered();

    let manager = manager.clone();

    let respond = move |mut response: http::Response<Cow<'static, [u8]>>| {
      response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
      response.headers_mut().insert(
        ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static(TAURI_RESPONSE_HEADER_NAME),
      );
      responder.respond(response);
    };

    match *request.method() {
      Method::POST => {
        if let Some(webview) = manager.get_webview(label) {
          match parse_invoke_request(&manager, request) {
            Ok(request) => {
              #[cfg(feature = "tracing")]
              span.record(
                "request",
                match &request.body {
                  super::InvokeBody::Json(j) => serde_json::to_string(j).unwrap(),
                  super::InvokeBody::Raw(b) => serde_json::to_string(b).unwrap(),
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
                  let response_span = match &response {
                    InvokeResponse::Ok(InvokeResponseBody::Json(v)) => tracing::trace_span!(
                      "ipc::request::response",
                      response = v,
                      mime_type = tracing::field::Empty
                    )
                    .entered(),
                    InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => tracing::trace_span!(
                      "ipc::request::response",
                      response = format!("{v:?}"),
                      mime_type = tracing::field::Empty
                    )
                    .entered(),
                    InvokeResponse::Err(e) => tracing::trace_span!(
                      "ipc::request::response",
                      error = format!("{e:?}"),
                      mime_type = tracing::field::Empty
                    )
                    .entered(),
                  };

                  let response_header = match &response {
                    InvokeResponse::Ok(_) => TAURI_RESPONSE_HEADER_OK,
                    InvokeResponse::Err(_) => TAURI_RESPONSE_HEADER_ERROR,
                  };

                  let (mut response, mime_type) = match response {
                    InvokeResponse::Ok(InvokeResponseBody::Json(v)) => (
                      http::Response::new(v.as_bytes().to_vec().into()),
                      mime::APPLICATION_JSON,
                    ),
                    InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => (
                      http::Response::new(v.into()),
                      mime::APPLICATION_OCTET_STREAM,
                    ),
                    InvokeResponse::Err(e) => (
                      http::Response::new(serde_json::to_vec(&e.0).unwrap().into()),
                      mime::APPLICATION_JSON,
                    ),
                  };

                  response
                    .headers_mut()
                    .insert(TAURI_RESPONSE_HEADER_NAME, response_header.parse().unwrap());

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
                  .status(StatusCode::INTERNAL_SERVER_ERROR)
                  .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
                  .body(e.as_bytes().to_vec().into())
                  .unwrap(),
              );
            }
          }
        } else {
          respond(
            http::Response::builder()
              .status(StatusCode::INTERNAL_SERVER_ERROR)
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
        r.headers_mut()
          .insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));

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

fn handle_ipc_message<R: Runtime>(request: Request<String>, manager: &AppManager<R>, label: &str) {
  if let Some(webview) = manager.get_webview(label) {
    #[cfg(feature = "tracing")]
    let _span = tracing::trace_span!(
      "ipc::request",
      kind = "post-message",
      uri = request.uri().to_string(),
      request = request.body()
    )
    .entered();

    use serde::{Deserialize, Deserializer};

    #[derive(Default)]
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

    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct RequestOptions {
      #[serde(default)]
      headers: HeaderMap,
      #[serde(default)]
      custom_protocol_ipc_blocked: bool,
    }

    #[derive(Deserialize)]
    struct Message {
      cmd: String,
      callback: CallbackFn,
      error: CallbackFn,
      payload: serde_json::Value,
      options: Option<RequestOptions>,
      #[serde(rename = "__TAURI_INVOKE_KEY__")]
      invoke_key: String,
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
        #[serde(rename = "__TAURI_INVOKE_KEY__")]
        invoke_key: String,
      }

      if let crate::Pattern::Isolation { crypto_keys, .. } = &*manager.pattern {
        #[cfg(feature = "tracing")]
        let _span = tracing::trace_span!("ipc::request::decrypt_isolation_payload").entered();

        invoke_message.replace(
          serde_json::from_str::<IsolationMessage<'_>>(request.body())
            .map_err(Into::into)
            .and_then(|message| {
              let is_raw =
                message.payload.content_type() == &mime::APPLICATION_OCTET_STREAM.to_string();
              let payload = crypto_keys.decrypt(message.payload)?;
              Ok(Message {
                cmd: message.cmd,
                callback: message.callback,
                error: message.error,
                payload: if is_raw {
                  payload.into()
                } else {
                  serde_json::from_slice(&payload)?
                },
                options: message.options,
                invoke_key: message.invoke_key,
              })
            }),
        );
      }
    }

    let message = invoke_message.unwrap_or_else(|| {
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("ipc::request::deserialize").entered();
      serde_json::from_str::<Message>(request.body()).map_err(Into::into)
    });

    match message {
      Ok(message) => {
        let options = message.options.unwrap_or_default();

        let request = InvokeRequest {
          cmd: message.cmd,
          callback: message.callback,
          error: message.error,
          url: Url::parse(&request.uri().to_string()).expect("invalid IPC request URL"),
          body: message.payload.into(),
          headers: options.headers.0,
          invoke_key: message.invoke_key,
        };

        #[cfg(feature = "tracing")]
        let request_span = tracing::trace_span!("ipc::request::handle", cmd = request.cmd);

        webview.on_message(
          request,
          Box::new(move |webview, cmd, response, callback, error| {
            use crate::ipc::Channel;

            #[cfg(feature = "tracing")]
            let _respond_span = tracing::trace_span!(
              parent: &request_span,
              "ipc::request::respond"
            )
            .entered();

            fn responder_eval<R: Runtime>(
              webview: &crate::Webview<R>,
              js: crate::Result<String>,
              error: CallbackFn,
            ) {
              let eval_js = match js {
                Ok(js) => js,
                Err(e) => crate::ipc::format_callback::format(error, &e.to_string())
                  .expect("unable to serialize response error string to json"),
              };

              let _ = webview.eval(&eval_js);
            }

            let can_use_channel_for_response = cmd
              != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
              && !options.custom_protocol_ipc_blocked;

            #[cfg(feature = "tracing")]
            let mime_type = match &response {
              InvokeResponse::Ok(InvokeResponseBody::Json(_)) => mime::APPLICATION_JSON,
              InvokeResponse::Ok(InvokeResponseBody::Raw(_)) => mime::APPLICATION_OCTET_STREAM,
              InvokeResponse::Err(_) => mime::APPLICATION_JSON,
            };

            #[cfg(feature = "tracing")]
            let _response_span = match &response {
              InvokeResponse::Ok(InvokeResponseBody::Json(v)) => tracing::trace_span!(
                "ipc::request::response",
                response = v,
                mime_type = mime_type.essence_str()
              )
              .entered(),
              InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => tracing::trace_span!(
                "ipc::request::response",
                response = format!("{v:?}"),
                mime_type = mime_type.essence_str()
              )
              .entered(),
              InvokeResponse::Err(e) => tracing::trace_span!(
                "ipc::request::response",
                response = format!("{e:?}"),
                mime_type = mime_type.essence_str()
              )
              .entered(),
            };

            match response {
              InvokeResponse::Ok(InvokeResponseBody::Json(v)) => {
                if !(cfg!(target_os = "macos") || cfg!(target_os = "ios"))
                  && (v.starts_with('{') || v.starts_with('['))
                  && can_use_channel_for_response
                {
                  let _ =
                    Channel::from_callback_fn(webview, callback).send(InvokeResponseBody::Json(v));
                } else {
                  responder_eval(
                    &webview,
                    crate::ipc::format_callback::format_result_raw(
                      Result::<_, String>::Ok(v),
                      callback,
                      error,
                    ),
                    error,
                  )
                }
              }
              InvokeResponse::Ok(InvokeResponseBody::Raw(v)) => {
                if cfg!(target_os = "macos")
                  || cfg!(target_os = "ios")
                  || !can_use_channel_for_response
                {
                  responder_eval(
                    &webview,
                    crate::ipc::format_callback::format_result(
                      Result::<_, ()>::Ok(v),
                      callback,
                      error,
                    ),
                    error,
                  );
                } else {
                  let _ = Channel::from_callback_fn(webview, callback)
                    .send(InvokeResponseBody::Raw(v.clone()));
                }
              }
              InvokeResponse::Err(e) => responder_eval(
                &webview,
                crate::ipc::format_callback::format_result(
                  Result::<(), _>::Err(&e.0),
                  callback,
                  error,
                ),
                error,
              ),
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

  // on Android we cannot read the request body
  // so we must ignore it because some commands use the IPC for faster response
  let has_payload = !body.is_empty();

  #[allow(unused_mut)]
  let mut content_type = parts
    .headers
    .get(http::header::CONTENT_TYPE)
    .and_then(|h| h.to_str().ok())
    .map(|mime| mime.parse())
    .unwrap_or(Ok(mime::APPLICATION_OCTET_STREAM))
    .map_err(|_| "unknown content type")?;

  #[cfg(feature = "isolation")]
  if let crate::Pattern::Isolation { crypto_keys, .. } = &*manager.pattern {
    // if the platform does not support request body, we ignore it
    if has_payload {
      #[cfg(feature = "tracing")]
      let _span = tracing::trace_span!("ipc::request::decrypt_isolation_payload").entered();

      (body, content_type) = crate::utils::pattern::isolation::RawIsolationPayload::try_from(&body)
        .and_then(|raw| {
          let content_type = raw.content_type().clone();
          crypto_keys.decrypt(raw).map(|decrypted| {
            (
              decrypted,
              content_type
                .parse()
                .unwrap_or(mime::APPLICATION_OCTET_STREAM),
            )
          })
        })
        .map_err(|e| e.to_string())?;
    }
  }

  let invoke_key = parts
    .headers
    .get(TAURI_INVOKE_KEY_HEADER_NAME)
    .ok_or("missing Tauri-Invoke-Key header")?
    .to_str()
    .map_err(|_| "Tauri invoke key header value must be a string")?
    .to_owned();

  let url = Url::parse(
    parts
      .headers
      .get("Origin")
      .ok_or("missing Origin header")?
      .to_str()
      .map_err(|_| "Origin header value must be a string")?,
  )
  .map_err(|_| "Origin header is not a valid URL")?;

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
    url,
    body,
    headers: parts.headers,
    invoke_key,
  };

  Ok(payload)
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;
  use crate::{ipc::InvokeBody, manager::AppManager, plugin::PluginStore, StateManager, Wry};
  use http::header::*;
  use serde_json::json;
  use tauri_macros::generate_context;

  #[test]
  fn parse_invoke_request() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate, test = true);
    let manager: AppManager<Wry> = AppManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| false),
      None,
      Default::default(),
      StateManager::new(),
      Default::default(),
      #[cfg(all(desktop, feature = "tray-icon"))]
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      "".into(),
      None,
      crate::generate_invoke_key().unwrap(),
    );

    let cmd = "write_something";
    let url = "tauri://localhost";
    let invoke_key = "1234ahdsjkl123";
    let callback = 12378123;
    let error = 6243;
    let headers = HeaderMap::from_iter(vec![
      (
        CONTENT_TYPE,
        HeaderValue::from_str(mime::APPLICATION_OCTET_STREAM.as_ref()).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_INVOKE_KEY_HEADER_NAME).unwrap(),
        HeaderValue::from_str(invoke_key).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_CALLBACK_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&callback.to_string()).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_ERROR_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&error.to_string()).unwrap(),
      ),
      (ORIGIN, HeaderValue::from_str("tauri://localhost").unwrap()),
    ]);

    let mut request = Request::builder().uri(format!("ipc://localhost/{cmd}"));
    *request.headers_mut().unwrap() = headers.clone();

    let body = vec![123, 31, 45];
    let request = request.body(body.clone()).unwrap();
    let invoke_request = super::parse_invoke_request(&manager, request).unwrap();

    assert_eq!(invoke_request.cmd, cmd);
    assert_eq!(invoke_request.callback.0, callback);
    assert_eq!(invoke_request.error.0, error);
    assert_eq!(invoke_request.invoke_key, invoke_key);
    assert_eq!(invoke_request.url, url.parse().unwrap());
    assert_eq!(invoke_request.headers, headers);
    assert_eq!(invoke_request.body, InvokeBody::Raw(body));

    let body = json!({
      "key": 1,
      "anotherKey": "asda",
    });

    let mut headers = headers.clone();
    headers.insert(
      CONTENT_TYPE,
      HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
    );

    let mut request = Request::builder().uri(format!("ipc://localhost/{cmd}"));
    *request.headers_mut().unwrap() = headers.clone();

    let request = request.body(serde_json::to_vec(&body).unwrap()).unwrap();
    let invoke_request = super::parse_invoke_request(&manager, request).unwrap();

    assert_eq!(invoke_request.headers, headers);
    assert_eq!(invoke_request.body, InvokeBody::Json(body));
  }

  #[test]
  #[cfg(feature = "isolation")]
  fn parse_invoke_request_isolation() {
    let context = generate_context!(
      "test/fixture/isolation/src-tauri/tauri.conf.json",
      crate,
      test = false
    );

    let crate::pattern::Pattern::Isolation { crypto_keys, .. } = &context.pattern else {
      unreachable!()
    };

    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce).unwrap();

    let body_raw = vec![1, 41, 65, 12, 78];
    let body_bytes = crypto_keys.aes_gcm().encrypt(&nonce, &body_raw).unwrap();
    let isolation_payload_raw = json!({
      "nonce": nonce,
      "payload": body_bytes,
      "contentType":  mime::APPLICATION_OCTET_STREAM.to_string(),
    });

    let body_json = json!({
      "key": 1,
      "anotherKey": "string"
    });
    let body_bytes = crypto_keys
      .aes_gcm()
      .encrypt(&nonce, &serde_json::to_vec(&body_json).unwrap())
      .unwrap();
    let isolation_payload_json = json!({
      "nonce": nonce,
      "payload": body_bytes,
      "contentType":  mime::APPLICATION_JSON.to_string(),
    });

    let manager: AppManager<Wry> = AppManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| false),
      None,
      Default::default(),
      StateManager::new(),
      Default::default(),
      #[cfg(all(desktop, feature = "tray-icon"))]
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      "".into(),
      None,
      crate::generate_invoke_key().unwrap(),
    );

    let cmd = "write_something";
    let url = "tauri://localhost";
    let invoke_key = "1234ahdsjkl123";
    let callback = 12378123;
    let error = 6243;

    let headers = HeaderMap::from_iter(vec![
      (
        CONTENT_TYPE,
        HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_INVOKE_KEY_HEADER_NAME).unwrap(),
        HeaderValue::from_str(invoke_key).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_CALLBACK_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&callback.to_string()).unwrap(),
      ),
      (
        HeaderName::from_str(TAURI_ERROR_HEADER_NAME).unwrap(),
        HeaderValue::from_str(&error.to_string()).unwrap(),
      ),
      (ORIGIN, HeaderValue::from_str("tauri://localhost").unwrap()),
    ]);

    let mut request = Request::builder().uri(format!("ipc://localhost/{cmd}"));
    *request.headers_mut().unwrap() = headers.clone();
    let body = serde_json::to_vec(&isolation_payload_raw).unwrap();
    let request = request.body(body).unwrap();
    let invoke_request = super::parse_invoke_request(&manager, request).unwrap();

    assert_eq!(invoke_request.cmd, cmd);
    assert_eq!(invoke_request.callback.0, callback);
    assert_eq!(invoke_request.error.0, error);
    assert_eq!(invoke_request.invoke_key, invoke_key);
    assert_eq!(invoke_request.url, url.parse().unwrap());
    assert_eq!(invoke_request.headers, headers);
    assert_eq!(invoke_request.body, InvokeBody::Raw(body_raw));

    let mut request = Request::builder().uri(format!("ipc://localhost/{cmd}"));
    *request.headers_mut().unwrap() = headers.clone();
    let body = serde_json::to_vec(&isolation_payload_json).unwrap();
    let request = request.body(body).unwrap();
    let invoke_request = super::parse_invoke_request(&manager, request).unwrap();

    assert_eq!(invoke_request.headers, headers);
    assert_eq!(invoke_request.body, InvokeBody::Json(body_json));
  }
}
