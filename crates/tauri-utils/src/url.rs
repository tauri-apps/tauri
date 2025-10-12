// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! URL helpers.

use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

/// UrlPattern to match URLs.
#[derive(Debug, Clone)]
pub struct UrlPattern(Arc<urlpattern::UrlPattern>, String);

#[cfg(feature = "schema")]
impl schemars::JsonSchema for UrlPattern {
  fn schema_name() -> String {
    "UrlPattern".to_string()
  }

  fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    String::json_schema(_gen)
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
    Self::from_str(&s).map_err(serde::de::Error::custom)
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

  /// Test if a given URL matches the pattern.
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
}
