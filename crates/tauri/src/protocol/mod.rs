// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Custom protocol handlers

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use url::Url;

#[cfg(feature = "protocol-asset")]
pub mod asset;
#[cfg(feature = "isolation")]
pub mod isolation;
pub mod tauri;

/// Percent-encoding set matching JavaScript's `encodeURIComponent`,
/// which leaves `A-Z a-z 0-9 - _ . ! ~ * ' ( )` unescaped.
pub(crate) const ENCODE_URI_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
  .remove(b'-')
  .remove(b'_')
  .remove(b'.')
  .remove(b'!')
  .remove(b'~')
  .remove(b'*')
  .remove(b'\'')
  .remove(b'(')
  .remove(b')');

/// The origin (`scheme://host`, without a trailing slash) a custom protocol is served from.
///
/// On Windows and Android custom protocols are served over `http(s)://{scheme}.localhost`;
/// everywhere else they are served over `{scheme}://localhost`.
pub(crate) fn origin(scheme: &str, use_https: bool) -> String {
  if cfg!(windows) || cfg!(target_os = "android") {
    let http = if use_https { "https" } else { "http" };
    format!("{http}://{scheme}.localhost")
  } else {
    format!("{scheme}://localhost")
  }
}

/// Rewrites a `tauri://localhost` URL to the custom app origin, matching the URL wry loads.
pub(crate) fn with_custom_app_origin(url: Url, custom_app_origin: &Url) -> Url {
  if url.scheme() != "tauri" || !url.authority().eq_ignore_ascii_case("localhost") {
    return url;
  }

  let mut app_url = custom_app_origin.clone();
  app_url.set_path(url.path());
  app_url.set_query(url.query());
  app_url.set_fragment(url.fragment());
  app_url
}

/// The protocol of an `http(s)://<protocol>.localhost` URL on Windows and Android,
/// skipping `tauri` when wry serves it from the custom app origin.
#[cfg(any(windows, target_os = "android", test))]
pub(crate) fn localhost_protocol(url: &Url, has_custom_app_origin: bool) -> Option<&str> {
  url
    .domain()
    .and_then(|domain| domain.strip_suffix(".localhost"))
    .filter(|protocol| !(has_custom_app_origin && *protocol == "tauri"))
}

#[cfg(test)]
mod tests {
  use url::Url;

  use super::{localhost_protocol, with_custom_app_origin};

  fn url(url: &str) -> Url {
    Url::parse(url).unwrap()
  }

  #[test]
  fn tauri_localhost_moves_to_custom_app_origin() {
    let origin = url("https://app.example.com");
    for (input, expected) in [
      ("tauri://localhost", "https://app.example.com/"),
      ("tauri://localhost/", "https://app.example.com/"),
      (
        "tauri://localhost/nested/page.html?q=tauri://localhost#hash",
        "https://app.example.com/nested/page.html?q=tauri://localhost#hash",
      ),
      ("tauri://LOCALHOST/page", "https://app.example.com/page"),
    ] {
      assert_eq!(
        with_custom_app_origin(url(input), &origin).as_str(),
        expected
      );
    }
    for unchanged in [
      "tauri://localhost:8080/",
      "tauri://user@localhost/",
      "tauri://other/",
      "https://tauri.localhost/",
      "https://example.com/",
      "custom://localhost/",
    ] {
      assert_eq!(
        with_custom_app_origin(url(unchanged), &origin).as_str(),
        unchanged
      );
    }
  }

  #[test]
  fn tauri_localhost_is_not_a_protocol_with_custom_app_origin() {
    for (input, unmapped, mapped) in [
      ("https://tauri.localhost/", Some("tauri"), None),
      ("http://tauri.localhost/", Some("tauri"), None),
      ("https://TAURI.localhost:8443/", Some("tauri"), None),
      ("https://custom.localhost/", Some("custom"), Some("custom")),
      ("https://app.example.com/", None, None),
    ] {
      let url = url(input);
      assert_eq!(localhost_protocol(&url, false), unmapped, "{input}");
      assert_eq!(localhost_protocol(&url, true), mapped, "{input}");
    }
  }

  #[test]
  fn encode_uri_component_matches_js() {
    // encodeURIComponent("aZ09-_.!~*'() /\\?#&%+é")
    let input = "aZ09-_.!~*'() /\\?#&%+é";
    let expected = "aZ09-_.!~*'()%20%2F%5C%3F%23%26%25%2B%C3%A9";
    assert_eq!(
      percent_encoding::utf8_percent_encode(input, super::ENCODE_URI_COMPONENT).to_string(),
      expected
    );
  }
}
