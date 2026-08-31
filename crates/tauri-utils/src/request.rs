// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Helpers to interpret requests served by Tauri.

use http::{header::ACCEPT, HeaderMap};

const SEC_FETCH_DEST: &str = "sec-fetch-dest";

/// Whether the request is a document navigation, i.e. the webview is loading a
/// page rather than a subresource of one.
///
/// A navigation may resolve to the SPA `index.html` fallback, so the frontend
/// router can react to any URL.
/// A subresource (script, style, image, font, `fetch`) must not, because
/// serving an HTML document in its place fails with a misleading error that
/// names neither the URL nor the cause.
///
/// Chromium based webviews send [`Sec-Fetch-Dest`], which answers this
/// directly.
/// WebKit does not send fetch metadata for custom protocols, but its `Accept`
/// header names `text/html` for navigations and never for subresources, which
/// send `*/*`, `image/...` or `text/css` instead.
/// When neither header is present nothing is assumed and the request counts as
/// a navigation, preserving the fallback.
///
/// [`Sec-Fetch-Dest`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-Fetch-Dest
pub fn is_navigation(headers: &HeaderMap) -> bool {
  if let Some(destination) = headers.get(SEC_FETCH_DEST).and_then(|v| v.to_str().ok()) {
    return matches!(
      destination.trim().to_ascii_lowercase().as_str(),
      "document" | "iframe" | "frame"
    );
  }

  match headers.get(ACCEPT).and_then(|v| v.to_str().ok()) {
    Some(accept) => accept.to_ascii_lowercase().contains("text/html"),
    None => true,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn headers(entries: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in entries {
      headers.insert(
        http::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
        value.parse().unwrap(),
      );
    }
    headers
  }

  #[test]
  fn fetch_metadata_answers_directly() {
    for destination in ["document", "iframe", "frame", "DOCUMENT"] {
      assert!(is_navigation(&headers(&[("sec-fetch-dest", destination)])));
    }
    for destination in ["script", "style", "image", "font", "empty", "worker"] {
      assert!(!is_navigation(&headers(&[("sec-fetch-dest", destination)])));
    }
  }

  #[test]
  fn fetch_metadata_wins_over_accept() {
    assert!(!is_navigation(&headers(&[
      ("sec-fetch-dest", "script"),
      ("accept", "text/html,*/*"),
    ])));
  }

  #[test]
  fn accept_distinguishes_webkit_requests() {
    // values observed on WebKitGTK 2.52.3 for `tauri://` requests
    assert!(is_navigation(&headers(&[(
      "accept",
      "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
    )])));
    assert!(!is_navigation(&headers(&[("accept", "*/*")])));
    assert!(!is_navigation(&headers(&[(
      "accept",
      "image/webp,image/avif,image/jxl,video/*;q=0.8,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5"
    )])));
    assert!(!is_navigation(&headers(&[(
      "accept",
      "text/css,*/*;q=0.1"
    )])));
  }

  #[test]
  fn without_evidence_the_fallback_is_kept() {
    assert!(is_navigation(&headers(&[])));
    assert!(is_navigation(&headers(&[("user-agent", "whatever")])));
  }
}
