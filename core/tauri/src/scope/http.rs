// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_utils::config::HttpAllowlistScope;
use url::Url;

/// Scope for filesystem access.
#[derive(Debug, Clone)]
pub struct Scope {
  allowed_urls: Vec<Url>,
}

impl Scope {
  /// Creates a new scope from the allowlist's `http` scope configuration.
  pub fn for_http_api(scope: &HttpAllowlistScope) -> Self {
    Self {
      allowed_urls: scope.0.clone(),
    }
  }

  /// Determines if the given URL is allowed on this scope.
  pub fn is_allowed(&self, url: &Url) -> bool {
    self.allowed_urls.iter().any(|allowed| {
      allowed.scheme() == url.scheme()
        && allowed.host() == url.host()
        && allowed.port() == url.port()
    })
  }
}
