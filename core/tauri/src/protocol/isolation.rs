// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use http::header::CONTENT_TYPE;
use serialize_to_javascript::Template;
use tauri_utils::assets::{Assets, EmbeddedAssets};

use std::sync::Arc;

use crate::{manager::webview::PROCESS_IPC_MESSAGE_FN, webview::UriSchemeProtocolHandler};

pub fn get(assets: Arc<EmbeddedAssets>, aes_gcm_key: [u8; 32]) -> UriSchemeProtocolHandler {
  Box::new(move |request, responder| {
    let response = match request_to_path(&request).as_str() {
      "index.html" => match assets.get(&"index.html".into()) {
        Some(asset) => {
          let asset = String::from_utf8_lossy(asset.as_ref());
          let template = tauri_utils::pattern::isolation::IsolationJavascriptRuntime {
            runtime_aes_gcm_key: &aes_gcm_key,
            process_ipc_message_fn: PROCESS_IPC_MESSAGE_FN,
          };
          match template.render(asset.as_ref(), &Default::default()) {
            Ok(asset) => http::Response::builder()
              .header(CONTENT_TYPE, mime::TEXT_HTML.as_ref())
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
          .status(http::StatusCode::BAD_REQUEST)
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
