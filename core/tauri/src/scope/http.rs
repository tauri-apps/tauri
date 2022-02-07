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
      let origin_matches = allowed.scheme() == url.scheme()
        && allowed.host() == url.host()
        && allowed.port() == url.port();
      let allowed_path_pattern = glob::Pattern::new(allowed.path())
        .unwrap_or_else(|_| panic!("invalid glob pattern on URL `{}` path", allowed));
      origin_matches && allowed_path_pattern.matches(url.path())
    })
  }
}

#[cfg(test)]
mod tests {
  use tauri_utils::config::HttpAllowlistScope;

  #[test]
  fn is_allowed() {
    // plain URL
    let scope = super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080"
      .parse()
      .unwrap()]));
    assert!(scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
    assert!(scope.is_allowed(&"http://localhost:8080/".parse().unwrap()));

    assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
    assert!(!scope.is_allowed(&"http://localhost:8080/path/to/asset.png".parse().unwrap()));
    assert!(!scope.is_allowed(&"https://localhost:8080".parse().unwrap()));
    assert!(!scope.is_allowed(&"http://localhost:8081".parse().unwrap()));
    assert!(!scope.is_allowed(&"http://local:8080".parse().unwrap()));

    // URL with fixed path
    let scope =
      super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080/file.png"
        .parse()
        .unwrap()]));

    assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));

    assert!(!scope.is_allowed(&"http://localhost:8080".parse().unwrap()));
    assert!(!scope.is_allowed(&"http://localhost:8080/file".parse().unwrap()));
    assert!(!scope.is_allowed(&"http://localhost:8080/file.png/other.jpg".parse().unwrap()));

    // URL with glob pattern
    let scope =
      super::Scope::for_http_api(&HttpAllowlistScope(vec!["http://localhost:8080/*.png"
        .parse()
        .unwrap()]));

    assert!(scope.is_allowed(&"http://localhost:8080/file.png".parse().unwrap()));
    assert!(scope.is_allowed(&"http://localhost:8080/assets/file.png".parse().unwrap()));

    assert!(!scope.is_allowed(&"http://localhost:8080/file.jpeg".parse().unwrap()));
  }
}
