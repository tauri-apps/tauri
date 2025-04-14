// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "isolation")]
use std::sync::Arc;

use serde::Serialize;
use serialize_to_javascript::{default_template, Template};

/// The domain of the isolation iframe source.
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum IsolationSide {
  /// Original frame, the Brownfield application
  Original,
  /// Secure frame, the isolation security application
  #[allow(dead_code)]
  Secure,
}

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

#[allow(dead_code)]
pub(crate) fn format_real_schema(schema: &str, https: bool) -> String {
  if cfg!(windows) || cfg!(target_os = "android") {
    let scheme = if https { "https" } else { "http" };
    format!("{scheme}://{schema}.{ISOLATION_IFRAME_SRC_DOMAIN}")
  } else {
    format!("{schema}://{ISOLATION_IFRAME_SRC_DOMAIN}")
  }
}
