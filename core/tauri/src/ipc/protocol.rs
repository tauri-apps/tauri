// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use http::{
  header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN},
  HeaderValue, Method, StatusCode,
};

use crate::{
  manager::WindowManager,
  runtime::http::{Request as HttpRequest, Response as HttpResponse},
  window::{InvokeRequest, UriSchemeProtocolHandler},
  Runtime,
};

use super::{CallbackFn, InvokeBody, InvokeResponse};

const TAURI_CALLBACK_HEADER_NAME: &str = "Tauri-Callback";
const TAURI_ERROR_HEADER_NAME: &str = "Tauri-Error";

#[cfg(any(target_os = "macos", not(ipc_custom_protocol)))]
pub fn message_handler<R: Runtime>(
  manager: WindowManager<R>,
) -> crate::runtime::webview::WebviewIpcHandler<crate::EventLoopMessage, R> {
  Box::new(move |window, request| handle_ipc_message(request, &manager, &window.label))
}

pub fn get<R: Runtime>(manager: WindowManager<R>, label: String) -> UriSchemeProtocolHandler {
  Box::new(move |request| {
    let mut response = match *request.method() {
      Method::POST => {
        let (mut response, content_type) = match handle_ipc_request(request, &manager, &label) {
          Ok(data) => match data {
            InvokeResponse::Ok(InvokeBody::Json(v)) => (
              HttpResponse::new(serde_json::to_vec(&v)?.into()),
              mime::APPLICATION_JSON,
            ),
            InvokeResponse::Ok(InvokeBody::Raw(v)) => {
              (HttpResponse::new(v.into()), mime::APPLICATION_OCTET_STREAM)
            }
            InvokeResponse::Err(e) => {
              let mut response = HttpResponse::new(serde_json::to_vec(&e.0)?.into());
              response.set_status(StatusCode::BAD_REQUEST);
              (response, mime::TEXT_PLAIN)
            }
          },
          Err(e) => {
            let mut response = HttpResponse::new(e.as_bytes().to_vec().into());
            response.set_status(StatusCode::BAD_REQUEST);
            (response, mime::TEXT_PLAIN)
          }
        };

        response.set_mimetype(Some(content_type.essence_str().into()));

        response
      }

      Method::OPTIONS => {
        let mut r = HttpResponse::new(Vec::new().into());
        r.headers_mut().insert(
          ACCESS_CONTROL_ALLOW_HEADERS,
          HeaderValue::from_static("Content-Type, Tauri-Callback, Tauri-Error, Tauri-Channel-Id"),
        );
        r
      }

      _ => {
        let mut r = HttpResponse::new(
          "only POST and OPTIONS are allowed"
            .as_bytes()
            .to_vec()
            .into(),
        );
        r.set_status(StatusCode::METHOD_NOT_ALLOWED);
        r.set_mimetype(Some(mime::TEXT_PLAIN.essence_str().into()));
        r
      }
    };

    response
      .headers_mut()
      .insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));

    Ok(response)
  })
}

#[cfg(any(target_os = "macos", not(ipc_custom_protocol)))]
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
        let _ = window.on_message(InvokeRequest {
          cmd: message.cmd,
          callback: message.callback,
          error: message.error,
          body: message.payload.into(),
          headers: message.options.map(|o| o.headers.0).unwrap_or_default(),
        });
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

fn handle_ipc_request<R: Runtime>(
  request: &HttpRequest,
  manager: &WindowManager<R>,
  label: &str,
) -> std::result::Result<InvokeResponse, String> {
  if let Some(window) = manager.get_window(label) {
    // TODO: consume instead
    #[allow(unused_mut)]
    let mut body = request.body().clone();

    let cmd = request
      .uri()
      .strip_prefix("ipc://localhost/")
      .map(|c| c.to_string())
      // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
      // where `$P` is not `localhost/*`
      // in this case the IPC call is considered invalid
      .unwrap_or_else(|| "".to_string());
    let cmd = percent_encoding::percent_decode(cmd.as_bytes())
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
      request
        .headers()
        .get(TAURI_CALLBACK_HEADER_NAME)
        .ok_or("missing Tauri-Callback header")?
        .to_str()
        .map_err(|_| "Tauri callback header value must be a string")?
        .parse()
        .map_err(|_| "Tauri callback header value must be a numeric string")?,
    );
    let error = CallbackFn(
      request
        .headers()
        .get(TAURI_ERROR_HEADER_NAME)
        .ok_or("missing Tauri-Error header")?
        .to_str()
        .map_err(|_| "Tauri error header value must be a string")?
        .parse()
        .map_err(|_| "Tauri error header value must be a numeric string")?,
    );

    let content_type = request
      .headers()
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
      headers: request.headers().clone(),
    };

    let rx = window.on_message(payload);
    Ok(rx.recv().unwrap())
  } else {
    Err("window not found".into())
  }
}
