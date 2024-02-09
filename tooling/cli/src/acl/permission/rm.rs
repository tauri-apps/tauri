// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use clap::Parser;

use crate::{helpers::app_paths::tauri_dir_opt, Result};

fn rm_permission_files(identifier: &str, dir: &Path) -> Result<()> {
  for entry in std::fs::read_dir(dir)?.flatten() {
    let file_type = entry.file_type()?;
    let path = entry.path();
    if file_type.is_dir() {
      rm_permission_files(identifier, &path)?;
    } else {
      match path.extension().and_then(|o| o.to_str()) {
        Some("toml") => {
          let content = std::fs::read_to_string(&path)?;
          if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            if value
              .as_table()
              .and_then(|o| o.get("identifier").and_then(|v| v.as_str()))
              .map(|i| i == identifier)
              .unwrap_or(false)
            {
              std::fs::remove_file(&path)?;
              log::info!(action = "Removed"; "permission at {}", dunce::simplified(&path).display());
            }
          }
        }
        Some("json") => {
          let content = std::fs::read(&path)?;
          if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&content) {
            if value
              .as_object()
              .and_then(|o| o.get("identifier").and_then(|v| v.as_str()))
              .map(|i| i == identifier)
              .unwrap_or(false)
            {
              std::fs::remove_file(&path)?;
              log::info!(action = "Removed"; "permission at {}", dunce::simplified(&path).display());
            }
          }
        }
        _ => {}
      }
    }
  }

  Ok(())
}

fn rm_permission_from_capabilities(identifier: &str, dir: &Path) -> Result<()> {
  for entry in std::fs::read_dir(dir)?.flatten() {
    let file_type = entry.file_type()?;
    if file_type.is_file() {
      let path = entry.path();
      match path.extension().and_then(|o| o.to_str()) {
        Some("toml") => {
          let content = std::fs::read_to_string(&path)?;
          if let Ok(mut value) = content.parse::<toml_edit::Document>() {
            if let Some(permissions) = value.get_mut("permissions").and_then(|p| p.as_array_mut()) {
              let prev_len = permissions.len();
              permissions.retain(|p| p.as_str().map(|p| p != identifier).unwrap_or(false));
              if prev_len != permissions.len() {
                std::fs::write(&path, value.to_string())?;
                log::info!(action = "Removed"; "permission from capability at {}", dunce::simplified(&path).display());
              }
            }
          }
        }
        Some("json") => {
          let content = std::fs::read(&path)?;
          if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&content) {
            if let Some(permissions) = value.get_mut("permissions").and_then(|p| p.as_array_mut()) {
              let prev_len = permissions.len();
              permissions.retain(|p| p.as_str().map(|p| p != identifier).unwrap_or(false));
              if prev_len != permissions.len() {
                std::fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
                log::info!(action = "Removed"; "permission from capability at {}", dunce::simplified(&path).display());
              }
            }
          }
        }
        _ => {}
      }
    }
  }

  Ok(())
}

#[derive(Debug, Parser)]
#[clap(about = "Remove a permission file, and its reference from any capability")]
pub struct Options {
  /// Permission to remove.
  identifier: String,
}

pub fn command(options: Options) -> Result<()> {
  let dir = match tauri_dir_opt() {
    Some(t) => t,
    None => std::env::current_dir()?,
  };

  let permissions_dir = dir.join("permissions");
  if permissions_dir.exists() {
    rm_permission_files(&options.identifier, &permissions_dir)?;
  }

  let capabilities_dir = dir.join("capabilities");
  if capabilities_dir.exists() {
    rm_permission_from_capabilities(&options.identifier, &capabilities_dir)?;
  }

  Ok(())
}
