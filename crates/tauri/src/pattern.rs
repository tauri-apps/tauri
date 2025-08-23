// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "isolation")]
use std::sync::Arc;

use serde::Serialize;
use serialize_to_javascript::{default_template, Template};

/// The domain of the isolation iframe source.
#[cfg(feature = "isolation")]
pub const ISOLATION_IFRAME_SRC_DOMAIN: &str = "localhost";

/// An application pattern.
#[derive(Debug)]
pub enum Pattern {
  /// The brownfield pattern.
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  #[cfg(feature = "isolation")]
  Isolation {
    /// The HTML served on `isolation://index.html`.
    assets: Arc<tauri_utils::assets::EmbeddedAssets>,

    /// The schema used for the isolation frames.
    schema: String,

    /// A random string used to ensure that the message went through the isolation frame.
    ///
    /// This should be regenerated at runtime.
    key: String,

    /// Cryptographically secure keys
    crypto_keys: Box<tauri_utils::pattern::isolation::Keys>,
  },
}

impl Pattern {
  /// Returns the isolation schema if using the isolation pattern.
  #[cfg(feature = "isolation")]
  pub fn isolation_schema(&self) -> Option<&str> {
    match self {
      Pattern::Isolation { schema, .. } => Some(schema.as_str()),
      _ => None,
    }
  }

  /// Returns a formatted isolation frame source URL for CSP configuration.
  ///
  /// This returns URL that should be used in the Content-Security-Policy frame-src directive.
  #[cfg(feature = "isolation")]
  pub fn isolation_frame_src(&self, use_https_scheme: bool) -> Option<String> {
    self.isolation_schema()
      .map(|schema| format_real_schema(schema, use_https_scheme))
  }
}

/// The shape of the JavaScript Pattern config
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase", tag = "pattern")]
pub(crate) enum PatternObject {
  /// Brownfield pattern.
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  #[cfg(feature = "isolation")]
  Isolation {
    /// Which `IsolationSide` this `PatternObject` is getting injected into
    side: IsolationSide,
  },
}

impl From<&Pattern> for PatternObject {
  fn from(pattern: &Pattern) -> Self {
    match pattern {
      Pattern::Brownfield => Self::Brownfield,
      #[cfg(feature = "isolation")]
      Pattern::Isolation { .. } => Self::Isolation {
        side: IsolationSide::default(),
      },
    }
  }
}

/// Where the JavaScript is injected to
#[cfg(feature = "isolation")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum IsolationSide {
  /// Original frame, the Brownfield application
  Original,
  /// Secure frame, the isolation security application
  #[allow(dead_code)]
  Secure,
}

#[cfg(feature = "isolation")]
impl Default for IsolationSide {
  fn default() -> Self {
    Self::Original
  }
}

#[derive(Template)]
#[default_template("../scripts/pattern.js")]
pub(crate) struct PatternJavascript {
  pub(crate) pattern: PatternObject,
}

#[cfg(feature = "isolation")]
pub(crate) fn format_real_schema(schema: &str, https: bool) -> String {
  if cfg!(windows) || cfg!(target_os = "android") {
    let scheme = if https { "https" } else { "http" };
    format!("{scheme}://{schema}.{ISOLATION_IFRAME_SRC_DOMAIN}/")
  } else {
    format!("{schema}://{ISOLATION_IFRAME_SRC_DOMAIN}/")
  }
}
