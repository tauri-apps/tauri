// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, sync::Arc};

use http::{header::CONTENT_TYPE, Request, Response as HttpResponse, StatusCode};
use tauri_utils::config::HeaderAddition;

use crate::{
  manager::{webview::PROXY_DEV_SERVER, AppManager},
  webview::{UriSchemeProtocolHandler, WebResourceRequestHandler},
  Runtime,
};

#[cfg(all(dev, mobile))]
use std::{collections::HashMap, sync::Mutex};

#[cfg(all(dev, mobile))]
#[derive(Clone)]
struct CachedResponse {
  status: http::StatusCode,
  headers: http::HeaderMap,
  body: bytes::Bytes,
}

pub fn get<R: Runtime>(
  #[allow(unused_variables)] manager: Arc<AppManager<R>>,
  window_origin: &str,
  web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,
) -> UriSchemeProtocolHandler {
  #[cfg(all(dev, mobile))]
  let url = {
    let mut url = manager
      .get_url(window_origin.starts_with("https"))
      .as_str()
      .to_string();
    if url.ends_with('/') {
      url.pop();
    }
    url
  };

  let window_origin = window_origin.to_string();

  #[cfg(all(dev, mobile))]
  let response_cache = Arc::new(Mutex::new(HashMap::new()));

  Box::new(move |_, request, responder| {
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
          .status(StatusCode::INTERNAL_SERVER_ERROR)
          .header(CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
          .header("Access-Control-Allow-Origin", &window_origin)
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

  let mut builder = HttpResponse::builder()
    .add_configured_headers(manager.config.app.security.headers.as_ref())
    .header("Access-Control-Allow-Origin", window_origin);

  #[cfg(all(dev, mobile))]
  let mut response = {
    let decoded_path = percent_encoding::percent_decode(path.as_bytes())
      .decode_utf8_lossy()
      .to_string();
    let url = format!(
      "{}/{}",
      url.trim_end_matches('/'),
      decoded_path.trim_start_matches('/')
    );

    let mut proxy_builder = reqwest::ClientBuilder::new()
      .use_rustls_tls()
      .build()
      .unwrap()
      .request(request.method().clone(), &url);
    for (name, value) in request.headers() {
      proxy_builder = proxy_builder.header(name, value);
    }
    match crate::async_runtime::safe_block_on(proxy_builder.send()) {
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
          let body = crate::async_runtime::safe_block_on(r.bytes())?;
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
        log::error!("Failed to request {}: {}", url.as_str(), e);
        return Err(Box::new(e));
      }
    }
  };

  #[cfg(not(all(dev, mobile)))]
  let mut response = {
    let use_https_scheme = request.uri().scheme() == Some(&http::uri::Scheme::HTTPS);
    let asset = manager.get_asset(path, use_https_scheme)?;
    builder = builder.header(CONTENT_TYPE, &asset.mime_type);
    if let Some(csp) = &asset.csp_header {
      builder = builder.header("Content-Security-Policy", csp);
    }
    builder.body(asset.bytes.into())?
  };
  if let Some(handler) = &web_resource_request_handler {
    handler(request, &mut response);
  }

  Ok(response)
}
