// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
      Self::Json => serde_json::to_string_pretty(s)?,
      Self::Toml => toml_edit::ser::to_string_pretty(s)?,
    };
    Ok(contents)
  }
}
