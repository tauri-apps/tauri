// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::result_large_err)]

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// All extensions that are possibly supported, but perhaps not enabled.
const EXTENSIONS_SUPPORTED: &[&str] = &["json", "json5", "toml"];

/// All configuration formats that are currently enabled.
const ENABLED_FORMATS: &[ConfigFormat] = &[
  ConfigFormat::Json,
  #[cfg(feature = "config-json5")]
  ConfigFormat::Json5,
  #[cfg(feature = "config-toml")]
  ConfigFormat::Toml,
];

/// The available configuration formats.
#[derive(Debug, Copy, Clone)]
enum ConfigFormat {
  /// The default JSON (tauri.conf.json) format.
  Json,
  /// The JSON5 (tauri.conf.json5) format.
  Json5,
  /// The TOML (Tauri.toml file) format.
  Toml,
}

impl ConfigFormat {
  /// Maps the config format to its file name.
  fn into_file_name(self) -> &'static str {
    match self {
      Self::Json => "tauri.conf.json",
      Self::Json5 => "tauri.conf.json5",
      Self::Toml => "Tauri.toml",
    }
  }

  fn into_platform_file_name(self) -> &'static str {
    match self {
      Self::Json => {
        if cfg!(target_os = "macos") {
          "tauri.macos.conf.json"
        } else if cfg!(windows) {
          "tauri.windows.conf.json"
        } else {
          "tauri.linux.conf.json"
        }
      }
      Self::Json5 => {
        if cfg!(target_os = "macos") {
          "tauri.macos.conf.json5"
        } else if cfg!(windows) {
          "tauri.windows.conf.json5"
        } else {
          "tauri.linux.conf.json5"
        }
      }
      Self::Toml => {
        if cfg!(target_os = "macos") {
          "Tauri.macos.toml"
        } else if cfg!(windows) {
          "Tauri.windows.toml"
        } else {
          "Tauri.linux.toml"
        }
      }
    }
  }
}

/// Represents all the errors that can happen while reading the config.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
  /// Failed to parse from JSON.
  #[error("unable to parse JSON Tauri config file at {path} because {error}")]
  FormatJson {
    /// The path that failed to parse into JSON.
    path: PathBuf,

    /// The parsing [`serde_json::Error`].
    error: serde_json::Error,
  },

  /// Failed to parse from JSON5.
  #[cfg(feature = "config-json5")]
  #[error("unable to parse JSON5 Tauri config file at {path} because {error}")]
  FormatJson5 {
    /// The path that failed to parse into JSON5.
    path: PathBuf,

    /// The parsing [`json5::Error`].
    error: ::json5::Error,
  },

  /// Failed to parse from TOML.
  #[cfg(feature = "config-toml")]
  #[error("unable to parse toml Tauri config file at {path} because {error}")]
  FormatToml {
    /// The path that failed to parse into TOML.
    path: PathBuf,

    /// The parsing [`toml::Error`].
    error: ::toml::de::Error,
  },

  /// Unknown config file name encountered.
  #[error("unsupported format encountered {0}")]
  UnsupportedFormat(String),

  /// Known file extension encountered, but corresponding parser is not enabled (cargo features).
  #[error("supported (but disabled) format encountered {extension} - try enabling `{feature}` ")]
  DisabledFormat {
    /// The extension encountered.
    extension: String,

    /// The cargo feature to enable it.
    feature: String,
  },

  /// A generic IO error with context of what caused it.
  #[error("unable to read Tauri config file at {path} because {error}")]
  Io {
    /// The path the IO error occurred on.
    path: PathBuf,

    /// The [`std::io::Error`].
    error: std::io::Error,
  },
}

/// See [`parse`] for specifics, returns a JSON [`Value`] instead of [`Config`].
pub fn parse_value(path: impl Into<PathBuf>) -> Result<(Value, PathBuf), ConfigError> {
  do_parse(path.into())
}

fn do_parse<D: DeserializeOwned>(path: PathBuf) -> Result<(D, PathBuf), ConfigError> {
  let file_name = path
    .file_name()
    .map(OsStr::to_string_lossy)
    .unwrap_or_default();
  let lookup_platform_config = ENABLED_FORMATS
    .iter()
    .any(|format| file_name == format.into_platform_file_name());

  let json5 = path.with_file_name(if lookup_platform_config {
    ConfigFormat::Json5.into_platform_file_name()
  } else {
    ConfigFormat::Json5.into_file_name()
  });
  let toml = path.with_file_name(if lookup_platform_config {
    ConfigFormat::Toml.into_platform_file_name()
  } else {
    ConfigFormat::Toml.into_file_name()
  });

  let path_ext = path
    .extension()
    .map(OsStr::to_string_lossy)
    .unwrap_or_default();

  if path.exists() {
    let raw = read_to_string(&path)?;

    // to allow us to easily use the compile-time #[cfg], we always bind
    #[allow(clippy::let_and_return)]
    let json = do_parse_json(&raw, &path);

    // we also want to support **valid** json5 in the .json extension if the feature is enabled.
    // if the json5 is not valid the serde_json error for regular json will be returned.
    // this could be a bit confusing, so we may want to encourage users using json5 to use the
    // .json5 extension instead of .json
    #[cfg(feature = "config-json5")]
    let json = {
      match do_parse_json5(&raw, &path) {
        json5 @ Ok(_) => json5,

        // assume any errors from json5 in a .json file is because it's not json5
        Err(_) => json,
      }
    };

    json.map(|j| (j, path))
  } else if json5.exists() {
    #[cfg(feature = "config-json5")]
    {
      let raw = read_to_string(&json5)?;
      do_parse_json5(&raw, &path).map(|config| (config, json5))
    }

    #[cfg(not(feature = "config-json5"))]
    Err(ConfigError::DisabledFormat {
      extension: ".json5".into(),
      feature: "config-json5".into(),
    })
  } else if toml.exists() {
    #[cfg(feature = "config-toml")]
    {
      let raw = read_to_string(&toml)?;
      do_parse_toml(&raw, &path).map(|config| (config, toml))
    }

    #[cfg(not(feature = "config-toml"))]
    Err(ConfigError::DisabledFormat {
      extension: ".toml".into(),
      feature: "config-toml".into(),
    })
  } else if !EXTENSIONS_SUPPORTED.contains(&path_ext.as_ref()) {
    Err(ConfigError::UnsupportedFormat(path_ext.to_string()))
  } else {
    Err(ConfigError::Io {
      path,
      error: std::io::ErrorKind::NotFound.into(),
    })
  }
}

fn do_parse_json<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
  serde_json::from_str(raw).map_err(|error| ConfigError::FormatJson {
    path: path.into(),
    error,
  })
}

#[cfg(feature = "config-json5")]
fn do_parse_json5<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
  ::json5::from_str(raw).map_err(|error| ConfigError::FormatJson5 {
    path: path.into(),
    error,
  })
}

#[cfg(feature = "config-toml")]
fn do_parse_toml<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
  ::toml::from_str(raw).map_err(|error| ConfigError::FormatToml {
    path: path.into(),
    error,
  })
}

/// Helper function to wrap IO errors from [`std::fs::read_to_string`] into a [`ConfigError`].
fn read_to_string(path: &Path) -> Result<String, ConfigError> {
  std::fs::read_to_string(path).map_err(|error| ConfigError::Io {
    path: path.into(),
    error,
  })
}
