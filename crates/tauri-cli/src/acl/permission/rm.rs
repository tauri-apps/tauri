// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use clap::Parser;
use tauri_utils::acl::{manifest::PermissionFile, PERMISSION_SCHEMA_FILE_NAME};

use crate::{acl::FileFormat, helpers::app_paths::resolve_tauri_dir, Result};

fn rm_permission_files(identifier: &str, dir: &Path) -> Result<()> {
  for entry in std::fs::read_dir(dir)?.flatten() {
    let file_type = entry.file_type()?;
    let path = entry.path();
    if file_type.is_dir() {
      rm_permission_files(identifier, &path)?;
    } else {
      if path
        .file_name()
        .map(|name| name == PERMISSION_SCHEMA_FILE_NAME)
        .unwrap_or_default()
      {
        continue;
      }

      let (mut permission_file, format): (PermissionFile, FileFormat) =
        match path.extension().and_then(|o| o.to_str()) {
          Some("toml") => {
            let content = std::fs::read_to_string(&path)?;
            (toml::from_str(&content)?, FileFormat::Toml)
          }
          Some("json") => {
            let content = std::fs::read(&path)?;
            (serde_json::from_slice(&content)?, FileFormat::Json)
          }
          _ => {
            continue;
          }
        };

      let mut updated;

      if identifier == "default" {
        updated = permission_file.default.is_some();
        permission_file.default = None;
      } else {
        let set_len = permission_file.set.len();
        permission_file
          .set
          .retain(|s| !identifier_match(identifier, &s.identifier));
        updated = permission_file.set.len() != set_len;

        let permission_len = permission_file.permission.len();
        permission_file
          .permission
          .retain(|s| !identifier_match(identifier, &s.identifier));
        updated = updated || permission_file.permission.len() != permission_len;
      }

      // if the file is empty, let's remove it
      if permission_file.default.is_none()
        && permission_file.set.is_empty()
        && permission_file.permission.is_empty()
      {
        std::fs::remove_file(&path)?;
        log::info!(action = "Removed"; "file {}", dunce::simplified(&path).display());
      } else if updated {
        std::fs::write(&path, format.serialize(&permission_file)?)?;
        log::info!(action = "Removed"; "permission {identifier} from {}", dunce::simplified(&path).display());
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
          if let Ok(mut value) = content.parse::<toml_edit::DocumentMut>() {
            if let Some(permissions) = value.get_mut("permissions").and_then(|p| p.as_array_mut()) {
              let prev_len = permissions.len();
              permissions.retain(|p| match p {
                toml_edit::Value::String(s) => !identifier_match(identifier, s.value()),
                toml_edit::Value::InlineTable(o) => {
                  if let Some(toml_edit::Value::String(permission_name)) = o.get("identifier") {
                    return !identifier_match(identifier, permission_name.value());
                  }

                  true
                }
                _ => false,
              });
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
              permissions.retain(|p| match p {
                serde_json::Value::String(s) => !identifier_match(identifier, s),
                serde_json::Value::Object(o) => {
                  if let Some(serde_json::Value::String(permission_name)) = o.get("identifier") {
                    return !identifier_match(identifier, permission_name);
                  }

                  true
                }
                _ => false,
              });
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

fn identifier_match(identifier: &str, permission: &str) -> bool {
  match identifier.split_once(':') {
    Some((plugin_name, "*")) => permission.contains(plugin_name),
    _ => permission == identifier,
  }
}

#[derive(Debug, Parser)]
#[clap(about = "Remove a permission file, and its reference from any capability")]
pub struct Options {
  /// Permission to remove.
  ///
  /// To remove all permissions for a given plugin, provide `<plugin-name>:*`
  pub identifier: String,
}

pub fn command(options: Options) -> Result<()> {
  let permissions_dir = std::env::current_dir()?.join("permissions");
  if permissions_dir.exists() {
    rm_permission_files(&options.identifier, &permissions_dir)?;
  }

  if let Some(tauri_dir) = resolve_tauri_dir() {
    let capabilities_dir = tauri_dir.join("capabilities");
    if capabilities_dir.exists() {
      rm_permission_from_capabilities(&options.identifier, &capabilities_dir)?;
    }
  }

  Ok(())
}
