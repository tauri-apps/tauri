// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::borrow::Cow;

use http::{header::CONTENT_TYPE, Request, Response as HttpResponse, StatusCode};

use crate::{
  manager::{window::PROXY_DEV_SERVER, AppManager},
  window::{UriSchemeProtocolHandler, WebResourceRequestHandler},
  Runtime,
};

#[cfg(all(dev, mobile))]
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

#[cfg(all(dev, mobile))]
#[derive(Clone)]
struct CachedResponse {
  status: http::StatusCode,
  headers: http::HeaderMap,
  body: bytes::Bytes,
}

pub fn get<R: Runtime>(
  #[allow(unused_variables)] manager: &AppManager<R>,
  window_origin: &str,
  web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,
) -> UriSchemeProtocolHandler {
  #[cfg(all(dev, mobile))]
  let url = {
    let mut url = manager.get_url().as_str().to_string();
    if url.ends_with('/') {
      url.pop();
    }
    url
  };

  let manager = manager.clone();
  let window_origin = window_origin.to_string();

  #[cfg(all(dev, mobile))]
  let response_cache = Arc::new(Mutex::new(HashMap::new()));

  Box::new(move |request, responder| {
    match get_response(
      request,
      &manager,
      &window_origin,
      web_resource_request_handler.as_deref(),
      #[cfg(all(dev, mobile))]
      (&url, &response_cache),
    ) {
      Ok(response) => responder.respond(response),
      Err(e) => responder.respond(
        HttpResponse::builder()
          .status(StatusCode::BAD_REQUEST)
          .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
          .body(e.to_string().as_bytes().to_vec())
          .unwrap(),
      ),
    }
  })
}

fn get_response<R: Runtime>(
  request: Request<Vec<u8>>,
  #[allow(unused_variables)] manager: &AppManager<R>,
  window_origin: &str,
  web_resource_request_handler: Option<&WebResourceRequestHandler>,
  #[cfg(all(dev, mobile))] (url, response_cache): (
    &str,
    &Arc<Mutex<HashMap<String, CachedResponse>>>,
  ),
) -> Result<HttpResponse<Cow<'static, [u8]>>, Box<dyn std::error::Error>> {
  // use the entire URI as we are going to proxy the request
  let path = if PROXY_DEV_SERVER {
    request.uri().to_string()
  } else {
    // ignore query string and fragment
    request
      .uri()
      .to_string()
      .split(&['?', '#'][..])
      .next()
      .unwrap()
      .into()
  };

  let path = path
    .strip_prefix("tauri://localhost")
    .map(|p| p.to_string())
    // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
    // where `$P` is not `localhost/*`
    .unwrap_or_else(|| "".to_string());

  let mut builder = HttpResponse::builder().header("Access-Control-Allow-Origin", window_origin);

  #[cfg(all(dev, mobile))]
  let mut response = {
    let decoded_path = percent_encoding::percent_decode(path.as_bytes())
      .decode_utf8_lossy()
      .to_string();
    let url = format!("{url}{decoded_path}");
    #[allow(unused_mut)]
    let mut client_builder = reqwest::ClientBuilder::new();
    #[cfg(any(feature = "native-tls", feature = "rustls-tls"))]
    {
      client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    let mut proxy_builder = client_builder
      .build()
      .unwrap()
      .request(request.method().clone(), &url);
    for (name, value) in request.headers() {
      proxy_builder = proxy_builder.header(name, value);
    }
    match crate::async_runtime::block_on(proxy_builder.send()) {
      Ok(r) => {
        let mut response_cache_ = response_cache.lock().unwrap();
        let mut response = None;
        if r.status() == http::StatusCode::NOT_MODIFIED {
          response = response_cache_.get(&url);
        }
        let response = if let Some(r) = response {
          r
        } else {
          let status = r.status();
          let headers = r.headers().clone();
          let body = crate::async_runtime::block_on(r.bytes())?;
          let response = CachedResponse {
            status,
            headers,
            body,
          };
          response_cache_.insert(url.clone(), response);
          response_cache_.get(&url).unwrap()
        };
        for (name, value) in &response.headers {
          builder = builder.header(name, value);
        }
        builder
          .status(response.status)
          .body(response.body.to_vec().into())?
      }
      Err(e) => {
        tauri_utils::debug_eprintln!("Failed to request {}: {}", url.as_str(), e);
        return Err(Box::new(e));
      }
    }
  };

  #[cfg(not(all(dev, mobile)))]
  let mut response = {
    let asset = manager.get_asset(path)?;
    builder = builder.header(CONTENT_TYPE, &asset.mime_type);
    if let Some(csp) = &asset.csp_header {
      builder = builder.header("Content-Security-Policy", csp);
    }
    builder.body(asset.bytes.into())?
  };
  if let Some(handler) = &web_resource_request_handler {
    handler(request, &mut response);
  }
  // if it's an HTML file, we need to set the CSP meta tag on Linux
  #[cfg(all(not(dev), target_os = "linux"))]
  if let Some(response_csp) = response.headers().get("Content-Security-Policy") {
    let response_csp = String::from_utf8_lossy(response_csp.as_bytes());
    let html = String::from_utf8_lossy(response.body());
    let body = html.replacen(tauri_utils::html::CSP_TOKEN, &response_csp, 1);
    *response.body_mut() = body.as_bytes().to_vec().into();
  }

  Ok(response)
}
