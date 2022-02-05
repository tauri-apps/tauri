// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::config::Config;
use json_patch::merge;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// All extensions that are possibly supported, but perhaps not enabled.
pub const EXTENSIONS_SUPPORTED: &[&str] = &["json", "json5"];

/// All extensions that are currently enabled.
pub const EXTENSIONS_ENABLED: &[&str] = &[
  "json",
  #[cfg(feature = "config-json5")]
  "json5",
];

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

  /// Unknown file extension encountered.
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
    /// The path the IO error occured on.
    path: PathBuf,

    /// The [`std::io::Error`].
    error: std::io::Error,
  },
}

/// Reads the configuration from the given root directory.
///
/// It first looks for a `tauri.conf.json[5]` file on the given directory. The file must exist.
/// Then it looks for a platform-specific configuration file:
/// - `tauri.macos.conf.json[5]` on macOS
/// - `tauri.linux.conf.json[5]` on Linux
/// - `tauri.windows.conf.json[5]` on Windows
/// Merging the configurations using [JSON Merge Patch (RFC 7396)].
///
/// [JSON Merge Patch (RFC 7396)]: https://datatracker.ietf.org/doc/html/rfc7396.
pub fn read_from(root_dir: PathBuf) -> Result<Value, ConfigError> {
  let mut config: Value = parse_value(root_dir.join("tauri.conf.json"))?;

  let platform_config_filename = if cfg!(target_os = "macos") {
    "tauri.macos.conf.json"
  } else if cfg!(windows) {
    "tauri.windows.conf.json"
  } else {
    "tauri.linux.conf.json"
  };
  let platform_config_path = root_dir.join(platform_config_filename);
  if does_supported_extension_exist(&platform_config_path) {
    let platform_config: Value = parse_value(platform_config_path)?;
    merge(&mut config, &platform_config);
  }
  Ok(config)
}

/// Check if a supported config file exists at path.
///
/// The passed path is expected to be the path to the "default" configuration format, in this case
/// JSON with `.json`.
pub fn does_supported_extension_exist(path: impl Into<PathBuf>) -> bool {
  let path = path.into();
  EXTENSIONS_ENABLED
    .iter()
    .any(|ext| path.with_extension(ext).exists())
}

/// Parse the config from path, including alternative formats.
///
/// Hierarchy:
/// 1. Check if `tauri.conf.json` exists
///   a. Parse it with `serde_json`
///   b. Parse it with `json5` if `serde_json` fails
///   c. Return original `serde_json` error if all above steps failed
/// 2. Check if `tauri.conf.json5` exists
///   a. Parse it with `json5`
///   b. Return error if all above steps failed
/// 3. Return error if all above steps failed
pub fn parse(path: impl Into<PathBuf>) -> Result<Config, ConfigError> {
  do_parse(path.into())
}

/// See [`parse`] for specifics, returns a JSON [`Value`] instead of [`Config`].
pub fn parse_value(path: impl Into<PathBuf>) -> Result<Value, ConfigError> {
  do_parse(path.into())
}

fn do_parse<D: DeserializeOwned>(path: PathBuf) -> Result<D, ConfigError> {
  let json5 = path.with_extension("json5");
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

    json
  } else if json5.exists() {
    #[cfg(feature = "config-json5")]
    {
      let raw = read_to_string(&json5)?;
      do_parse_json5(&raw, &path)
    }

    #[cfg(not(feature = "config-json5"))]
    Err(ConfigError::DisabledFormat {
      extension: ".json5".into(),
      feature: "config-json5".into(),
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

/// "Low-level" helper to parse JSON into a [`Config`].
///
/// `raw` should be the contents of the file that is represented by `path`.
pub fn parse_json(raw: &str, path: &Path) -> Result<Config, ConfigError> {
  do_parse_json(raw, path)
}

/// "Low-level" helper to parse JSON into a JSON [`Value`].
///
/// `raw` should be the contents of the file that is represented by `path`.
pub fn parse_json_value(raw: &str, path: &Path) -> Result<Value, ConfigError> {
  do_parse_json(raw, path)
}

fn do_parse_json<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
  serde_json::from_str(raw).map_err(|error| ConfigError::FormatJson {
    path: path.into(),
    error,
  })
}

/// "Low-level" helper to parse JSON5 into a [`Config`].
///
/// `raw` should be the contents of the file that is represented by `path`. This function requires
/// the `config-json5` feature to be enabled.
#[cfg(feature = "config-json5")]
pub fn parse_json5(raw: &str, path: &Path) -> Result<Config, ConfigError> {
  do_parse_json5(raw, path)
}

/// "Low-level" helper to parse JSON5 into a JSON [`Value`].
///
/// `raw` should be the contents of the file that is represented by `path`. This function requires
/// the `config-json5` feature to be enabled.
#[cfg(feature = "config-json5")]
pub fn parse_json5_value(raw: &str, path: &Path) -> Result<Value, ConfigError> {
  do_parse_json5(raw, path)
}

#[cfg(feature = "config-json5")]
fn do_parse_json5<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
  ::json5::from_str(raw).map_err(|error| ConfigError::FormatJson5 {
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
