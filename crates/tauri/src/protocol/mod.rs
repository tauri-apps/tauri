// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Custom protocol handlers

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

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

#[cfg(test)]
mod tests {
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
