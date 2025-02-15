// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Assets;
use http::header::CONTENT_TYPE;
use serialize_to_javascript::Template;
use tauri_utils::{
  assets::EmbeddedAssets,
  config::{Csp, HeaderAddition},
};

use std::sync::Arc;

use crate::{
  manager::{set_csp, webview::PROCESS_IPC_MESSAGE_FN, AppManager},
  webview::UriSchemeProtocolHandler,
  Runtime,
};

pub fn get<R: Runtime>(
  manager: Arc<AppManager<R>>,
  schema: &str,
  assets: Arc<EmbeddedAssets>,
  aes_gcm_key: [u8; 32],
  window_origin: String,
  use_https_scheme: bool,
) -> UriSchemeProtocolHandler {
  let frame_src = if cfg!(any(windows, target_os = "android")) {
    let https = if use_https_scheme { "https" } else { "http" };
    format!("{https}://{schema}.localhost")
  } else {
    format!("{schema}:")
  };

  let assets = assets as Arc<dyn Assets<R>>;

  Box::new(move |_, request, responder| {
    let response = match request_to_path(&request).as_str() {
      "index.html" => match assets.get(&"index.html".into()) {
        Some(asset) => {
          let mut asset = String::from_utf8_lossy(asset.as_ref()).into_owned();
          let csp_map = set_csp(
            &mut asset,
            &assets,
            &"index.html".into(),
            &manager,
            Csp::Policy(format!("default-src 'none'; frame-src {}", frame_src)),
          );
          let csp = Csp::DirectiveMap(csp_map).to_string();

          let template = tauri_utils::pattern::isolation::IsolationJavascriptRuntime {
            runtime_aes_gcm_key: &aes_gcm_key,
            origin: window_origin.clone(),
            process_ipc_message_fn: PROCESS_IPC_MESSAGE_FN,
          };
          match template.render(asset.as_ref(), &Default::default()) {
            Ok(asset) => http::Response::builder()
              .add_configured_headers(manager.config.app.security.headers.as_ref())
              .header(CONTENT_TYPE, mime::TEXT_HTML.as_ref())
              .header("Content-Security-Policy", csp)
              .body(asset.into_string().as_bytes().to_vec()),
            Err(_) => http::Response::builder()
              .status(http::StatusCode::INTERNAL_SERVER_ERROR)
              .header(CONTENT_TYPE, mime::TEXT_PLAIN.as_ref())
              .body(Vec::new()),
          }
        }

        None => http::Response::builder()
          .status(http::StatusCode::NOT_FOUND)
          .header(CONTENT_TYPE, mime::TEXT_PLAIN.as_ref())
          .body(Vec::new()),
      },
      _ => http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, mime::TEXT_PLAIN.as_ref())
        .body(Vec::new()),
    };

    if let Ok(r) = response {
      responder.respond(r);
    } else {
      responder.respond(
        http::Response::builder()
          .status(http::StatusCode::INTERNAL_SERVER_ERROR)
          .body("failed to get response".as_bytes().to_vec())
          .unwrap(),
      );
    }
  })
}

fn request_to_path(request: &http::Request<Vec<u8>>) -> String {
  let path = request
    .uri()
    .path()
    .trim_start_matches('/')
    .trim_end_matches('/');

  let path = percent_encoding::percent_decode(path.as_bytes())
    .decode_utf8_lossy()
    .to_string();

  if path.is_empty() {
    // if the url has no path, we should load `index.html`
    "index.html".to_string()
  } else {
    // skip leading `/`
    path.chars().skip(1).collect()
  }
}
