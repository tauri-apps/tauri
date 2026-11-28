// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! URL helpers.

use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

/// A pattern to match URLs, used to scope URL-based options such as [`crate::config::OnNewWindow`].
///
/// By default it is a glob pattern matched against the URL string:
/// - `*` matches any sequence of characters except `/`, so a host wildcard such as
///   `https://*.tauri.app/**` cannot match `https://evil.com/.tauri.app/`;
/// - `**` matches any number of path segments;
/// - `?` matches a single character, `[...]` a character class.
///
/// A pattern without a path such as `https://tauri.app` matches any path on that origin.
/// The URL username and password are ignored when matching.
///
/// When the `url-pattern` feature is enabled, the pattern follows the
/// [URLPattern](https://developer.mozilla.org/en-US/docs/Web/API/URLPattern) spec instead.
/// This pulls the `urlpattern` and `regex` crates into the binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlScope(UrlScopeMatcher);

#[cfg(feature = "url-pattern")]
type UrlScopeMatcher = UrlPattern;
#[cfg(not(feature = "url-pattern"))]
type UrlScopeMatcher = GlobUrlPattern;

/// Error returned when a [`UrlScope`] fails to parse.
#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid URL scope `{pattern}`: {error}")]
pub struct UrlScopeError {
  pattern: String,
  error: String,
}

impl UrlScope {
  /// The pattern string this scope was parsed from.
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// Test if a given URL is matched by the scope.
  pub fn test(&self, url: &Url) -> bool {
    self.0.test(url)
  }
}

impl FromStr for UrlScope {
  type Err = UrlScopeError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    s.parse().map(Self).map_err(|e| UrlScopeError {
      pattern: s.to_string(),
      error: e.to_string(),
    })
  }
}

impl Serialize for UrlScope {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.as_str())
  }
}

impl<'de> Deserialize<'de> for UrlScope {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Self::from_str(&s).map_err(serde::de::Error::custom)
  }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for UrlScope {
  fn schema_name() -> std::borrow::Cow<'static, str> {
    "UrlScope".into()
  }

  fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    String::json_schema(generator)
  }
}

/// Glob pattern matched against a URL string. See [`UrlScope`].
#[cfg(not(feature = "url-pattern"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct GlobUrlPattern {
  pattern: glob::Pattern,
  raw: String,
}

#[cfg(not(feature = "url-pattern"))]
impl FromStr for GlobUrlPattern {
  type Err = glob::PatternError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    // a pattern with only a scheme and host matches any path on that origin
    let has_path = s
      .split_once("://")
      .map(|(_, rest)| rest.contains('/'))
      .unwrap_or(true);
    let pattern = if has_path {
      glob::Pattern::new(s)?
    } else {
      glob::Pattern::new(&format!("{s}/**"))?
    };
    Ok(Self {
      pattern,
      raw: s.to_string(),
    })
  }
}

#[cfg(not(feature = "url-pattern"))]
impl GlobUrlPattern {
  const MATCH_OPTIONS: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
  };

  fn as_str(&self) -> &str {
    &self.raw
  }

  fn test(&self, url: &Url) -> bool {
    let mut url = url.clone();
    // ignore the userinfo so `https://tauri.app@evil.com` cannot be matched as `tauri.app`
    let _ = url.set_username("");
    let _ = url.set_password(None);
    self.pattern.matches_with(url.as_str(), Self::MATCH_OPTIONS)
  }
}

/// UrlPattern to match URLs.
#[derive(Debug, Clone)]
pub struct UrlPattern(Arc<urlpattern::UrlPattern>, String);

#[cfg(feature = "schema")]
impl schemars::JsonSchema for UrlPattern {
  fn schema_name() -> std::borrow::Cow<'static, str> {
    "UrlPattern".into()
  }

  fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    String::json_schema(generator)
  }
}

impl Serialize for UrlPattern {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.1)
  }
}

impl<'de> Deserialize<'de> for UrlPattern {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Self::from_str(&s)
      .map_err(|e| serde::de::Error::custom(format!("invalid URLPattern `{s}`: {e}")))
  }
}

impl FromStr for UrlPattern {
  type Err = urlpattern::quirks::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    let mut init = urlpattern::UrlPatternInit::parse_constructor_string::<regex::Regex>(s, None)?;
    if init.search.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
      init.search.replace("*".to_string());
    }
    if init.hash.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
      init.hash.replace("*".to_string());
    }
    if init
      .pathname
      .as_ref()
      .map(|p| p.is_empty() || p == "/")
      .unwrap_or(true)
    {
      init.pathname.replace("*".to_string());
    }
    let pattern = urlpattern::UrlPattern::parse(init, Default::default())?;
    Ok(Self(Arc::new(pattern), s.to_string()))
  }
}

impl UrlPattern {
  #[doc(hidden)]
  pub fn as_str(&self) -> &str {
    &self.1
  }

  /// Test if a given URL is matched by the pattern.
  pub fn test(&self, url: &Url) -> bool {
    self
      .0
      .test(urlpattern::UrlPatternMatchInput::Url(url.clone()))
      .unwrap_or_default()
  }
}

impl PartialEq for UrlPattern {
  fn eq(&self, other: &Self) -> bool {
    self.0.protocol() == other.0.protocol()
      && self.0.username() == other.0.username()
      && self.0.password() == other.0.password()
      && self.0.hostname() == other.0.hostname()
      && self.0.port() == other.0.port()
      && self.0.pathname() == other.0.pathname()
      && self.0.search() == other.0.search()
      && self.0.hash() == other.0.hash()
  }
}

impl Eq for UrlPattern {}

#[cfg(test)]
mod tests {
  use super::UrlPattern;

  #[test]
  fn url_pattern_serde() {
    let patterns: Vec<UrlPattern> =
      serde_json::from_str(r#"["https://*.tauri.app/*", "http://localhost:*"]"#).unwrap();
    let json = serde_json::to_value(&patterns).unwrap();
    assert_eq!(
      json,
      serde_json::json!(["https://*.tauri.app/*", "http://localhost:*"])
    );
    let roundtrip: Vec<UrlPattern> = serde_json::from_value(json).unwrap();
    assert_eq!(patterns, roundtrip);
  }

  #[test]
  fn url_pattern_error() {
    let err = serde_json::from_str::<UrlPattern>(r#""https://tauri.app/(a""#)
      .unwrap_err()
      .to_string();
    assert!(
      err.contains("invalid URLPattern `https://tauri.app/(a`"),
      "{err}"
    );
  }

  #[test]
  fn url_pattern_host_wildcard_does_not_cross_path() {
    let pattern: UrlPattern = "https://*.tauri.app/*".parse().unwrap();
    assert!(pattern.test(&"https://api.tauri.app/path".parse().unwrap()));
    assert!(!pattern.test(&"https://evil.com/.tauri.app/".parse().unwrap()));
    assert!(!pattern.test(&"https://api.tauri.app.evil.com/".parse().unwrap()));
  }

  #[test]
  fn url_pattern_origin_only() {
    let pattern: UrlPattern = "https://tauri.app".parse().unwrap();
    assert!(pattern.test(&"https://tauri.app".parse().unwrap()));
    assert!(pattern.test(&"https://tauri.app/a/b?q=1#h".parse().unwrap()));
    assert!(!pattern.test(&"https://tauri.app.evil.com/".parse().unwrap()));
  }

  #[test]
  fn url_pattern_domain_wildcard() {
    let pattern: UrlPattern = "http://*".parse().unwrap();

    assert!(pattern.test(&"http://tauri.app/path".parse().unwrap()));
    assert!(pattern.test(&"http://tauri.app/path?q=1".parse().unwrap()));

    assert!(pattern.test(&"http://localhost/path".parse().unwrap()));
    assert!(pattern.test(&"http://localhost/path?q=1".parse().unwrap()));

    let pattern: UrlPattern = "http://*.tauri.app".parse().unwrap();

    assert!(!pattern.test(&"http://tauri.app/path".parse().unwrap()));
    assert!(!pattern.test(&"http://tauri.app/path?q=1".parse().unwrap()));
    assert!(pattern.test(&"http://api.tauri.app/path".parse().unwrap()));
    assert!(pattern.test(&"http://api.tauri.app/path?q=1".parse().unwrap()));
    assert!(!pattern.test(&"http://localhost/path".parse().unwrap()));
    assert!(!pattern.test(&"http://localhost/path?q=1".parse().unwrap()));
  }

  #[test]
  fn url_pattern_path_wildcard() {
    let pattern: UrlPattern = "http://localhost/*".parse().unwrap();
    assert!(pattern.test(&"http://localhost/path".parse().unwrap()));
    assert!(pattern.test(&"http://localhost/path?q=1".parse().unwrap()));
  }

  #[test]
  fn url_pattern_scheme_wildcard() {
    let pattern: UrlPattern = "*://localhost".parse().unwrap();
    assert!(pattern.test(&"http://localhost/path".parse().unwrap()));
    assert!(pattern.test(&"https://localhost/path?q=1".parse().unwrap()));
    assert!(pattern.test(&"custom://localhost/path".parse().unwrap()));
  }

  mod scope {
    use crate::url::UrlScope;

    fn scope(s: &str) -> UrlScope {
      s.parse().unwrap()
    }

    fn matches(scope: &UrlScope, url: &str) -> bool {
      scope.test(&url.parse().unwrap())
    }

    #[test]
    fn origin_only() {
      let scope = scope("https://tauri.app");
      assert!(matches(&scope, "https://tauri.app"));
      assert!(matches(&scope, "https://tauri.app/a/b?q=1#h"));
      assert!(!matches(&scope, "https://tauri.app.evil.com/"));
      assert!(!matches(&scope, "http://tauri.app/"));
    }

    #[test]
    fn host_wildcard_does_not_cross_path() {
      let scope = scope("https://*.tauri.app");
      assert!(matches(&scope, "https://api.tauri.app/path"));
      assert!(!matches(&scope, "https://evil.com/.tauri.app/"));
      assert!(!matches(&scope, "https://api.tauri.app.evil.com/"));
    }

    #[test]
    fn userinfo_is_not_host() {
      let scope = scope("https://tauri.app");
      assert!(!matches(&scope, "https://tauri.app@evil.com/"));
      assert!(!matches(&scope, "https://tauri.app:pass@evil.com/"));
    }

    #[test]
    fn serde_roundtrip() {
      let scopes: Vec<UrlScope> =
        serde_json::from_str(r#"["https://*.tauri.app", "http://localhost:*"]"#).unwrap();
      assert_eq!(
        serde_json::to_value(&scopes).unwrap(),
        serde_json::json!(["https://*.tauri.app", "http://localhost:*"])
      );
    }

    #[cfg(not(feature = "url-pattern"))]
    #[test]
    fn glob_path_wildcards() {
      let single = scope("https://tauri.app/docs/*");
      assert!(matches(&single, "https://tauri.app/docs/start"));
      assert!(!matches(&single, "https://tauri.app/docs/start/install"));

      let any = scope("https://tauri.app/docs/**");
      assert!(matches(&any, "https://tauri.app/docs/start"));
      assert!(matches(&any, "https://tauri.app/docs/start/install?q=1"));
      assert!(!matches(&any, "https://tauri.app/blog/post"));
    }

    #[cfg(not(feature = "url-pattern"))]
    #[test]
    fn glob_invalid_pattern() {
      let err = "https://tauri.app/***".parse::<UrlScope>().unwrap_err();
      assert!(
        err
          .to_string()
          .contains("invalid URL scope `https://tauri.app/***`")
      );
    }
  }
}
