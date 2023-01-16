// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

#[cfg(not(feature = "reqwest-client"))]
use std::sync::Mutex;
use std::{
  collections::HashMap,
  fmt::{Display, Formatter},
  pin::Pin,
  sync::Arc,
};

use super::InvokeContext;
use crate::{
  api::{file::SafePathBuf, http::HeaderMap},
  endpoints::file_system::resolve_path,
  resources::ResourceId,
  Error, Runtime,
};
use futures_util::Future;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri_macros::{command_enum, module_command_handler, CommandModule};

#[cfg(not(feature = "reqwest-client"))]
impl crate::resources::Resource for FetchResponse {}

#[cfg(feature = "reqwest-client")]
type CancelableResponseResult = Result<super::Result<reqwest::Response>, Canceled>;
#[cfg(feature = "reqwest-client")]
pub type CancelableResponseFuture =
  Pin<Box<dyn Future<Output = CancelableResponseResult> + Send + Sync>>;
#[cfg(feature = "reqwest-client")]
use tokio::sync::Mutex;
#[cfg(feature = "reqwest-client")]
struct FetchRequestResource(Mutex<CancelableResponseFuture>);
#[cfg(feature = "reqwest-client")]
impl FetchRequestResource {
  fn new(f: CancelableResponseFuture) -> Self {
    Self(Mutex::new(f))
  }
}
#[cfg(feature = "reqwest-client")]
impl crate::resources::Resource for FetchRequestResource {}

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
  #[cmd(http_request, "http > request")]
  FetchSend { rid: ResourceId },
  #[cmd(http_request, "http > request")]
  FetchCancel { rid: ResourceId },
}

impl Cmd {
  #[module_command_handler(http_request)]
  async fn fetch<R: Runtime>(
    context: InvokeContext<R>,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    data: Option<Vec<u8>>,
  ) -> super::Result<(u32, Option<u32>)> {
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

        #[cfg(not(feature = "reqwest-client"))]
        {
          let data = std::fs::read(&resolved_path)
            .with_context(|| format!("path: {}", resolved_path.display()))?;

          let rid = context
            .resources_table
            .lock()
            .unwrap()
            .add(Mutex::new(Some(FetchResponse {
              status: 200,
              status_text: "OK".into(),
              headers: Vec::new(),
              url: url.to_string(),
              data,
            })));
          Ok((rid, None))
        }

        #[cfg(feature = "reqwest-client")]
        {
          // TODO
          panic!("FIX file:// protocol fetch on reqwest client");
          let rid = 0;
          Ok((rid, Some(rid)))
        }
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

            let rid =
              context
                .resources_table
                .lock()
                .unwrap()
                .add(Mutex::new(Some(FetchResponse {
                  status: status.as_u16(),
                  status_text: status.canonical_reason().unwrap_or_default().to_string(),
                  headers,
                  url: url.to_string(),
                  data: response.bytes().map_err(into_anyhow)?.to_vec(),
                })));

            Ok((rid, None))
          }

          #[cfg(feature = "reqwest-client")]
          {
            let mut request = reqwest::Client::new().request(method.clone(), url);

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

            let fut = async move { Ok(request.send().await.map_err(crate::error::into_anyhow)) };
            let rid = context
              .resources_table
              .lock()
              .unwrap()
              .add(FetchRequestResource::new(Box::pin(fut)));

            Ok((rid, Some(rid)))
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

        #[cfg(not(feature = "reqwest-client"))]
        {
          let status = http::StatusCode::OK;
          let rid = context
            .resources_table
            .lock()
            .unwrap()
            .add(Mutex::new(Some(FetchResponse {
              status: status.as_u16(),
              status_text: status.canonical_reason().unwrap_or_default().to_string(),
              headers: vec![(CONTENT_TYPE.to_string(), data_url.mime_type().to_string())],
              url: url.to_string(),
              data: body,
            })));
          Ok((rid, None))
        }

        #[cfg(feature = "reqwest-client")]
        {
          let response = http::Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, data_url.mime_type().to_string())
            .body(reqwest::Body::from(body))?;

          let fut = async move { Ok(Ok(reqwest::Response::from(response))) };
          let rid = context
            .resources_table
            .lock()
            .unwrap()
            .add(FetchRequestResource::new(Box::pin(fut)));

          Ok((rid, Some(rid)))
        }
      }
      _ => Err(into_anyhow(format!("scheme '{}' not supported", scheme))),
    }
  }

  #[module_command_handler(http_request)]
  async fn fetch_send<R: Runtime>(
    context: InvokeContext<R>,
    rid: ResourceId,
  ) -> super::Result<FetchResponse> {
    #[cfg(not(feature = "reqwest-client"))]
    {
      context
        .resources_table
        .lock()
        .unwrap()
        .take::<Mutex<Option<FetchResponse>>>(rid)
        .map(|a| a.lock().unwrap().take().unwrap())
        .map_err(Into::into)
    }

    #[cfg(feature = "reqwest-client")]
    {
      let req = context
        .resources_table
        .lock()
        .unwrap()
        .take::<FetchRequestResource>(rid)?;

      let res = match req.0.lock().await.as_mut().await {
        Ok(Ok(res)) => res,
        Ok(Err(err)) => return Err(err),
        Err(_) => return Err(crate::error::into_anyhow("request was cancelled")),
      };

      let status = res.status();
      let url = res.url().to_string();
      let mut headers = Vec::new();
      for (key, val) in res.headers().iter() {
        headers.push((
          key.as_str().into(),
          String::from_utf8(val.as_bytes().to_vec())?,
        ));
      }

      Ok(FetchResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or_default().to_string(),
        headers,
        url,
        data: res
          .bytes()
          .await
          .map_err(crate::error::into_anyhow)?
          .to_vec(),
      })
    }
  }
  #[module_command_handler(http_request)]
  async fn fetch_cancel<R: Runtime>(
    context: InvokeContext<R>,
    rid: ResourceId,
  ) -> super::Result<()> {
    #[cfg(not(feature = "reqwest-client"))]
    {
      // avoid unused warning
      let _ = rid;
      let _ = context;
      Err(crate::error::into_anyhow(
        "Cacelling fetch requests requires `reqwest-client` which is not enabled",
      ))
    }
    #[cfg(feature = "reqwest-client")]
    {
      context.resources_table.lock().unwrap().replace(
        rid,
        FetchRequestResource::new(Box::pin(async { Err(Canceled) })),
      );
      Ok(())
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

#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
pub struct Canceled;

impl Display for Canceled {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "operation canceled")
  }
}

impl std::error::Error for Canceled {}

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
