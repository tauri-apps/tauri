// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use super::InvokeContext;
use crate::{
  api::{file::SafePathBuf, http::HeaderMap},
  endpoints::file_system::resolve_path,
  Runtime,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri_macros::{command_enum, module_command_handler, CommandModule};

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[cmd(async)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The HTTP request API.
  #[cmd(http_request, "http > request")]
  Fetch {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    data: Option<Vec<u8>>,
  },
}

impl Cmd {
  #[module_command_handler(http_request)]
  async fn fetch<R: Runtime>(
    context: InvokeContext<R>,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    data: Option<Vec<u8>>,
  ) -> super::Result<FetchResponse> {
    use crate::{error::into_anyhow, Manager};
    use anyhow::Context;
    use http::{header::*, Method};

    let url = url::Url::parse(&url)?;
    let scheme = url.scheme();
    let method = Method::from_bytes(method.as_bytes()).map_err(into_anyhow)?;

    match scheme {
      "file" => {
        let path = url
          .to_file_path()
          .map_err(|_| into_anyhow("Failed to get path from `file:` url"))?;
        let path = SafePathBuf::new(path).map_err(into_anyhow)?;

        let resolved_path = resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          path,
          None,
          true,
        )?;

        if method != Method::GET {
          return Err(into_anyhow(format!(
            "Fetching files only supports the GET method. Received {}.",
            method
          )));
        }

        let data = std::fs::read(&resolved_path)
          .with_context(|| format!("path: {}", resolved_path.display()))?;

        Ok(FetchResponse {
          status: 200,
          status_text: "OK".into(),
          headers: Vec::new(),
          url: url.to_string(),
          data,
        })
      }
      "http" | "https" => {
        let scopes = &(context.window).state::<crate::Scopes>();
        if scopes.http.is_allowed(&url) {
          #[cfg(not(feature = "reqwest-client"))]
          {
            let mut request = attohttpc::RequestBuilder::try_new(method.clone(), &url)?;

            for (key, value) in headers {
              let name = HeaderName::from_bytes(key.as_bytes()).map_err(into_anyhow)?;
              let v = HeaderValue::from_bytes(value.as_bytes()).map_err(into_anyhow)?;
              if !matches!(name, HOST | CONTENT_LENGTH) {
                request = request.header(name, v);
              }
            }

            // POST and PUT requests should always have a 0 length content-length,
            // if there is no body. https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            if data.is_none() && matches!(method, Method::POST | Method::PUT) {
              request = request.header(CONTENT_LENGTH, HeaderValue::from(0));
            }

            let response = if let Some(data) = data {
              request.body(attohttpc::body::Bytes(data)).send()?
            } else {
              request.send()?
            };

            let status = response.status();
            let mut headers = Vec::new();

            for (key, val) in response.headers().iter() {
              headers.push((
                key.as_str().into(),
                String::from_utf8(val.as_bytes().to_vec())?,
              ));
            }

            Ok(FetchResponse {
              status: status.as_u16(),
              status_text: status.canonical_reason().unwrap_or_default().to_string(),
              headers,
              url: url.to_string(),
              data: response.bytes().map_err(into_anyhow)?.to_vec(),
            })
          }
          #[cfg(feature = "reqwest-client")]
          {
            let mut request = reqwest::Client::default().request(method.clone(), url);

            for (key, value) in headers {
              let name = HeaderName::from_bytes(key.as_bytes()).map_err(into_anyhow)?;
              let v = HeaderValue::from_bytes(value.as_bytes()).map_err(into_anyhow)?;
              if !matches!(name, HOST | CONTENT_LENGTH) {
                request = request.header(name, v);
              }
            }

            // POST and PUT requests should always have a 0 length content-length,
            // if there is no body. https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            if data.is_none() && matches!(method, Method::POST | Method::PUT) {
              request = request.header(CONTENT_LENGTH, HeaderValue::from(0));
            }

            if let Some(data) = data {
              request = request.body(data);
            }

            let response = request.send().await?;

            let status = response.status();
            let mut headers = Vec::new();

            for (key, val) in response.headers().iter() {
              headers.push((
                key.as_str().into(),
                String::from_utf8(val.as_bytes().to_vec())?,
              ));
            }

            Ok(FetchResponse {
              status: status.as_u16(),
              status_text: status.canonical_reason().unwrap_or_default().to_string(),
              headers,
              url: response.url().to_string(),
              data: response.bytes().await.map_err(into_anyhow)?.to_vec(),
            })
          }
        } else {
          Err(crate::Error::UrlNotAllowed(url).into_anyhow())
        }
      }
      "data" => {
        let data_url = data_url::DataUrl::process(url.as_str())
          .map_err(|_| into_anyhow("Failed to process data url"))?;
        let (body, _) = data_url
          .decode_to_vec()
          .map_err(|_| into_anyhow("Failed to decode data url to vec"))?;

        let status = http::StatusCode::OK;
        Ok(FetchResponse {
          status: status.as_u16(),
          status_text: status.canonical_reason().unwrap_or_default().to_string(),
          headers: vec![(CONTENT_TYPE.to_string(), data_url.mime_type().to_string())],
          url: url.to_string(),
          data: body,
        })
      }
      _ => Err(into_anyhow(format!("scheme '{}' not supported", scheme))),
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
  status: u16,
  status_text: String,
  headers: Vec<(String, String)>,
  url: String,
  data: Vec<u8>,
}

#[cfg(test)]
mod tests {
  #[tauri_macros::module_command_test(http_request, "http > request")]
  #[quickcheck_macros::quickcheck]
  fn fetch(method: String, url: String, headers: Vec<(String, String)>, data: Option<Vec<u8>>) {
    assert!(crate::async_runtime::block_on(super::Cmd::fetch(
      crate::test::mock_invoke_context(),
      method,
      url,
      headers,
      data
    ))
    .is_err());
  }
}
