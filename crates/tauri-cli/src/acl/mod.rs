// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Context;
use serde::Serialize;
use std::fmt::Display;

pub mod capability;
pub mod permission;

#[derive(Debug, clap::ValueEnum, Clone)]
enum FileFormat {
  Json,
  Toml,
}

impl Display for FileFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Json => write!(f, "json"),
      Self::Toml => write!(f, "toml"),
    }
  }
}

impl FileFormat {
  pub fn extension(&self) -> &'static str {
    match self {
      Self::Json => "json",
      Self::Toml => "toml",
    }
  }

  pub fn serialize<S: Serialize>(&self, s: &S) -> crate::Result<String> {
    let contents = match self {
      Self::Json => serde_json::to_string_pretty(s).context("failed to serialize JSON")?,
      Self::Toml => toml_edit::ser::to_string_pretty(s).context("failed to serialize TOML")?,
    };
    Ok(contents)
  }
}

/// Splits a comma separated prompt answer, trimming each entry and skipping empty ones.
fn split_comma_separated(input: &str) -> impl Iterator<Item = String> + '_ {
  input
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(ToString::to_string)
}

/// Validates a capability identifier, which is also used as its file name.
///
/// Only ASCII letters, digits, hyphens and underscores are allowed,
/// so the identifier cannot be used to write outside of the capabilities directory.
fn validate_capability_identifier(identifier: &str) -> crate::Result<()> {
  if identifier.is_empty() {
    crate::error::bail!("capability identifier cannot be empty");
  }
  if !identifier
    .bytes()
    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
  {
    crate::error::bail!(
      "invalid capability identifier `{}`: only ASCII letters, digits, hyphens and underscores are allowed",
      identifier
    );
  }
  Ok(())
}

/// Validates an app permission identifier, which is also used as its file name.
fn validate_permission_identifier(identifier: &str) -> crate::Result<()> {
  let parsed = match tauri_utils::acl::identifier::Identifier::try_from(identifier.to_string()) {
    Ok(parsed) => parsed,
    Err(e) => crate::error::bail!("invalid permission identifier `{}`: {}", identifier, e),
  };
  // app permissions are referenced without a prefix, the prefix is reserved for plugins
  if parsed.get_prefix().is_some() {
    crate::error::bail!(
      "invalid permission identifier `{}`: app permissions cannot have a `:` prefix",
      identifier
    );
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{
    split_comma_separated, validate_capability_identifier, validate_permission_identifier,
  };

  #[test]
  fn splits_comma_separated_input() {
    assert_eq!(
      split_comma_separated("fs:default, core:default,, ").collect::<Vec<_>>(),
      vec!["fs:default", "core:default"]
    );
    assert_eq!(split_comma_separated(" ").count(), 0);
  }

  #[test]
  fn capability_identifier_validation() {
    assert!(validate_capability_identifier("main-capability").is_ok());
    assert!(validate_capability_identifier("main_window2").is_ok());
    for invalid in ["", "../../x", "a/b", "a\\b", "..", "a.b", "a:b", "a b"] {
      assert!(
        validate_capability_identifier(invalid).is_err(),
        "{invalid} should be rejected"
      );
    }
  }

  #[test]
  fn permission_identifier_validation() {
    assert!(validate_permission_identifier("allow-greet").is_ok());
    for invalid in [
      "",
      "../../x",
      "a/b",
      "a\\b",
      "..",
      "a.b",
      "fs:allow-read",
      "-a",
    ] {
      assert!(
        validate_permission_identifier(invalid).is_err(),
        "{invalid} should be rejected"
      );
    }
  }
}
