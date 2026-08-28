// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::cli::Args;
use anyhow::Error;
use futures_util::TryFutureExt;
use http_body_util::{BodyExt, Full};
use hyper::{
  body::{Bytes, Incoming},
  header::CONTENT_LENGTH,
  http::uri::Authority,
  service::service_fn,
  Method, Request, Response,
};
use hyper_util::{
  client::legacy::{connect::HttpConnector, Client},
  rt::{TokioExecutor, TokioIo},
  server::conn::auto,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use std::process::Child;
use tokio::net::TcpListener;

const TAURI_OPTIONS: &str = "tauri:options";

// WebDriver BiDi capability auto-injected by clients like WebdriverIO 9+.
// tauri-driver does not proxy the BiDi websocket and native drivers without
// BiDi support (WebKitGTK < 2.46) reject sessions requesting it, so it is
// stripped before forwarding. BiDi is additive; clients fall back to classic
// WebDriver.
const BIDI_CAPABILITY: &str = "webSocketUrl";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TauriOptions {
  application: PathBuf,
  #[serde(default)]
  args: Vec<String>,
  #[cfg(target_os = "windows")]
  #[serde(default)]
  webview_options: Option<Value>,
}

impl TauriOptions {
  #[cfg(target_os = "linux")]
  fn into_native_object(self) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert(
      "webkitgtk:browserOptions".into(),
      json!({"binary": self.application, "args": self.args}),
    );
    map
  }

  #[cfg(target_os = "windows")]
  fn into_native_object(self) -> Map<String, Value> {
    let mut ms_edge_options = Map::new();
    ms_edge_options.insert(
      "binary".into(),
      json!(self.application.with_extension("exe")),
    );
    ms_edge_options.insert("args".into(), self.args.into());

    if let Some(webview_options) = self.webview_options {
      ms_edge_options.insert("webviewOptions".into(), webview_options);
    }

    let mut map = Map::new();
    map.insert("ms:edgeChromium".into(), json!(true));
    map.insert("browserName".into(), json!("webview2"));
    map.insert("ms:edgeOptions".into(), ms_edge_options.into());
    map
  }
}

async fn handle(
  client: Client<HttpConnector, Full<Bytes>>,
  req: Request<Incoming>,
  args: Args,
) -> Result<Response<Incoming>, Error> {
  // manipulate a new session to convert options to the native driver format
  let new_req: Request<Full<Bytes>> =
    if let (&Method::POST, "/session") = (req.method(), req.uri().path()) {
      let (mut parts, body) = req.into_parts();

      // get the body from the future stream and parse it as json
      let body = body.collect().await?.to_bytes().to_vec();
      let json: Value = serde_json::from_slice(&body)?;

      // manipulate the json to convert from tauri option to native driver options
      let json = map_capabilities(json);

      // serialize json and update the content-length header to be accurate
      let bytes = serde_json::to_vec(&json)?;
      parts.headers.insert(CONTENT_LENGTH, bytes.len().into());

      Request::from_parts(parts, Full::new(bytes.into()))
    } else if is_element_send_keys(req.method(), req.uri().path()) {
      let (mut parts, body) = req.into_parts();

      let mut bytes = body.collect().await?.to_bytes().to_vec();
      if let Ok(mut json) = serde_json::from_slice::<Value>(&bytes) {
        if ensure_send_keys_text(&mut json) {
          bytes = serde_json::to_vec(&json)?;
        }
      }
      parts.headers.insert(CONTENT_LENGTH, bytes.len().into());

      Request::from_parts(parts, Full::new(bytes.into()))
    } else {
      let (parts, body) = req.into_parts();

      let body = body.collect().await?.to_bytes().to_vec();

      Request::from_parts(parts, Full::new(body.into()))
    };

  client
    .request(forward_to_native_driver(new_req, args)?)
    .err_into()
    .await
}

/// Transform the request to a request for the native webdriver server.
fn forward_to_native_driver(
  mut req: Request<Full<Bytes>>,
  args: Args,
) -> Result<Request<Full<Bytes>>, Error> {
  let host: Authority = {
    let headers = req.headers_mut();
    headers.remove("host").expect("hyper request has host")
  }
  .to_str()?
  .parse()?;

  let path = req
    .uri()
    .path_and_query()
    .expect("hyper request has uri")
    .clone();

  let uri = format!(
    "http://{}:{}{}",
    host.host(),
    args.native_port,
    path.as_str()
  );

  let (mut parts, body) = req.into_parts();
  parts.uri = uri.parse()?;
  Ok(Request::from_parts(parts, body))
}

/// only happy path for now, no errors
fn map_capabilities(mut json: Value) -> Value {
  let mut native = None;
  if let Some(capabilities) = json.get_mut("capabilities") {
    if let Some(always_match) = capabilities.get_mut("alwaysMatch") {
      if let Some(always_match) = always_match.as_object_mut() {
        if let Some(tauri_options) = always_match.remove(TAURI_OPTIONS) {
          if let Ok(options) = serde_json::from_value::<TauriOptions>(tauri_options) {
            native = Some(options.into_native_object());
          }
        }

        if let Some(native) = native.clone() {
          always_match.extend(native);
        }
      }
    }

    strip_bidi_capabilities(capabilities);
  }

  if let Some(native) = native {
    if let Some(desired) = json.get_mut("desiredCapabilities") {
      if let Some(desired) = desired.as_object_mut() {
        desired.remove(TAURI_OPTIONS);
        desired.extend(native);
      }
    }
  }

  json
}

/// Removes WebDriver BiDi capabilities the native driver may not honor,
/// from both `alwaysMatch` and every `firstMatch` entry.
fn strip_bidi_capabilities(capabilities: &mut Value) {
  if let Some(always_match) = capabilities
    .get_mut("alwaysMatch")
    .and_then(Value::as_object_mut)
  {
    always_match.remove(BIDI_CAPABILITY);
  }

  if let Some(first_match) = capabilities
    .get_mut("firstMatch")
    .and_then(Value::as_array_mut)
  {
    for entry in first_match.iter_mut() {
      if let Some(entry) = entry.as_object_mut() {
        entry.remove(BIDI_CAPABILITY);
      }
    }
  }
}

/// `true` for the Element Send Keys endpoint (`POST /session/{id}/element/{id}/value`).
fn is_element_send_keys(method: &Method, path: &str) -> bool {
  method == Method::POST && {
    let mut segments = path.trim_start_matches('/').split('/');
    matches!(
      (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
      ),
      (
        Some("session"),
        Some(_),
        Some("element"),
        Some(_),
        Some("value"),
        None
      )
    )
  }
}

/// Adds the W3C `text` field to an Element Send Keys body that only carries
/// the legacy JSON Wire Protocol `value`. WebKitWebDriver 2.52+ rejects
/// bodies without `text`. Returns `true` if the body was modified.
fn ensure_send_keys_text(json: &mut Value) -> bool {
  let Some(obj) = json.as_object_mut() else {
    return false;
  };
  if obj.contains_key("text") {
    return false;
  }
  let text = match obj.get("value") {
    Some(Value::Array(chunks)) => chunks.iter().filter_map(Value::as_str).collect::<String>(),
    Some(Value::String(s)) => s.clone(),
    _ => return false,
  };
  obj.insert("text".into(), Value::String(text));
  true
}

#[tokio::main(flavor = "current_thread")]
pub async fn run(args: Args, mut _driver: Child) -> Result<(), Error> {
  #[cfg(unix)]
  let (signals_handle, signals_task) = {
    use futures_util::StreamExt;
    use signal_hook::consts::signal::*;

    let signals = signal_hook_tokio::Signals::new([SIGTERM, SIGINT, SIGQUIT])?;
    let signals_handle = signals.handle();
    let signals_task = tokio::spawn(async move {
      let mut signals = signals.fuse();
      #[allow(clippy::never_loop)]
      while let Some(signal) = signals.next().await {
        match signal {
          SIGTERM | SIGINT | SIGQUIT => {
            _driver
              .kill()
              .expect("unable to kill native webdriver server");
            std::process::exit(0);
          }
          _ => unreachable!(),
        }
      }
    });
    (signals_handle, signals_task)
  };

  let address = std::net::SocketAddr::from(([127, 0, 0, 1], args.port));

  // the client we use to proxy requests to the native webdriver
  let client = Client::builder(TokioExecutor::new())
    .http1_preserve_header_case(true)
    .http1_title_case_headers(true)
    .retry_canceled_requests(false)
    .build_http();

  // set up a http1 server that uses the service we just created
  let srv = async move {
    if let Ok(listener) = TcpListener::bind(address).await {
      loop {
        let client = client.clone();
        let args = args.clone();
        if let Ok((stream, _)) = listener.accept().await {
          let io = TokioIo::new(stream);

          tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
              .http1()
              .title_case_headers(true)
              .preserve_header_case(true)
              .serve_connection(
                io,
                service_fn(|request| handle(client.clone(), request, args.clone())),
              )
              .await
            {
              println!("Error serving connection: {err:?}");
            }
          });
        } else {
          println!("accept new stream fail, ignore here");
        }
      }
    } else {
      println!("can not listen to address: {address:?}");
    }
  };
  srv.await;

  #[cfg(unix)]
  {
    signals_handle.close();
    signals_task.await?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn strips_bidi_from_always_match_and_first_match() {
    let json = json!({
      "capabilities": {
        "alwaysMatch": { "browserName": "wry", "webSocketUrl": true },
        "firstMatch": [{ "webSocketUrl": true }, { "browserName": "wry" }]
      }
    });

    let mapped = map_capabilities(json);
    let capabilities = &mapped["capabilities"];

    assert!(capabilities["alwaysMatch"].get("webSocketUrl").is_none());
    assert_eq!(capabilities["alwaysMatch"]["browserName"], "wry");
    for entry in capabilities["firstMatch"].as_array().unwrap() {
      assert!(entry.get("webSocketUrl").is_none());
    }
  }

  #[test]
  fn strip_ignores_non_object_entries() {
    let mut capabilities = json!({
      "alwaysMatch": true,
      "firstMatch": [1, "x", { "webSocketUrl": true }]
    });
    strip_bidi_capabilities(&mut capabilities);
    assert!(capabilities["firstMatch"][2].get("webSocketUrl").is_none());
  }

  #[test]
  fn matches_element_send_keys_endpoint() {
    assert!(is_element_send_keys(
      &Method::POST,
      "/session/abc/element/def/value"
    ));
    assert!(!is_element_send_keys(
      &Method::GET,
      "/session/abc/element/def/value"
    ));
    assert!(!is_element_send_keys(
      &Method::POST,
      "/session/abc/element/def/value/extra"
    ));
    assert!(!is_element_send_keys(&Method::POST, "/session/abc/value"));
    assert!(!is_element_send_keys(&Method::POST, "/status"));
  }

  #[test]
  fn send_keys_text_synthesis() {
    // W3C body stays untouched
    let mut body = json!({ "text": "hello" });
    assert!(!ensure_send_keys_text(&mut body));
    assert_eq!(body["text"], "hello");

    // legacy value array gets a text field
    let mut body = json!({ "value": ["h", "i"] });
    assert!(ensure_send_keys_text(&mut body));
    assert_eq!(body["text"], "hi");
    assert_eq!(body["value"], json!(["h", "i"]));

    // both fields present stays untouched
    let mut body = json!({ "text": "hi", "value": ["h", "i"] });
    assert!(!ensure_send_keys_text(&mut body));

    // value as plain string
    let mut body = json!({ "value": "hello" });
    assert!(ensure_send_keys_text(&mut body));
    assert_eq!(body["text"], "hello");

    // nothing usable
    assert!(!ensure_send_keys_text(&mut json!({})));
    assert!(!ensure_send_keys_text(&mut json!([1, 2])));
  }
}
