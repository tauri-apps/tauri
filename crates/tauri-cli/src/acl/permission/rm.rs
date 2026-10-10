// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use clap::Parser;
use tauri_utils::acl::{PERMISSION_SCHEMA_FILE_NAME, manifest::PermissionFile};

use crate::{
  Result,
  acl::FileFormat,
  error::{Context, ErrorExt},
  helpers::app_paths::resolve_tauri_dir,
};

/// Parses a permission file, returning `None` if it is not a TOML or JSON file.
fn parse_permission_file(path: &Path) -> Result<Option<(PermissionFile, FileFormat)>> {
  let parsed = match path.extension().and_then(|o| o.to_str()) {
    Some("toml") => {
      let content = std::fs::read_to_string(path)
        .fs_context("failed to read permission file", path.to_path_buf())?;
      (
        toml::from_str(&content).context("failed to deserialize permission file")?,
        FileFormat::Toml,
      )
    }
    Some("json") => {
      let content =
        std::fs::read(path).fs_context("failed to read permission file", path.to_path_buf())?;
      (
        serde_json::from_slice(&content).context("failed to parse permission file as JSON")?,
        FileFormat::Json,
      )
    }
    _ => return Ok(None),
  };
  Ok(Some(parsed))
}

fn rm_permission_files(identifier: &str, dir: &Path) -> Result<()> {
  for entry in std::fs::read_dir(dir)
    .fs_context("failed to read permissions directory", dir.to_path_buf())?
    .flatten()
  {
    let file_type = entry
      .file_type()
      .fs_context("failed to get permission file type", entry.path())?;
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

      let (mut permission_file, format) = match parse_permission_file(&path) {
        Ok(Some(parsed)) => parsed,
        Ok(None) => continue,
        Err(e) => {
          log::warn!(
            "Skipping permission file {}: {e}",
            dunce::simplified(&path).display()
          );
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
        std::fs::remove_file(&path).fs_context("failed to remove permission file", path.clone())?;
        log::info!(action = "Removed"; "file {}", dunce::simplified(&path).display());
      } else if updated {
        std::fs::write(
          &path,
          format
            .serialize(&permission_file)
            .context("failed to serialize permission")?,
        )
        .fs_context("failed to write permission file", path.clone())?;
        log::info!(action = "Removed"; "permission {identifier} from {}", dunce::simplified(&path).display());
      }
    }
  }

  Ok(())
}

fn rm_permission_from_capabilities(identifier: &str, dir: &Path) -> Result<()> {
  for entry in std::fs::read_dir(dir)
    .fs_context("failed to read capabilities directory", dir.to_path_buf())?
    .flatten()
  {
    let file_type = entry
      .file_type()
      .fs_context("failed to get capability file type", entry.path())?;
    if !file_type.is_file() {
      continue;
    }
    let path = entry.path();
    match path.extension().and_then(|o| o.to_str()) {
      Some("toml") => {
        let content = std::fs::read_to_string(&path)
          .fs_context("failed to read capability file", path.clone())?;
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
              std::fs::write(&path, value.to_string())
                .fs_context("failed to write capability file", path.clone())?;
              log::info!(action = "Removed"; "permission from capability at {}", dunce::simplified(&path).display());
            }
          }
        }
      }
      Some("json") => {
        let content =
          std::fs::read(&path).fs_context("failed to read capability file", path.clone())?;
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
              std::fs::write(
                &path,
                serde_json::to_vec_pretty(&value).context("failed to serialize capability JSON")?,
              )
              .fs_context("failed to write capability file", path.clone())?;
              log::info!(action = "Removed"; "permission from capability at {}", dunce::simplified(&path).display());
            }
          }
        }
      }
      _ => {}
    }
  }

  Ok(())
}

fn identifier_match(identifier: &str, permission: &str) -> bool {
  match identifier.split_once(':') {
    Some((plugin_name, "*")) => {
      permission == plugin_name || permission.starts_with(&format!("{plugin_name}:"))
    }
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
  let tauri_dir = resolve_tauri_dir();

  // app-local permission identifiers are not plugin-prefixed,
  // so the `<plugin-name>:*` form never matches them
  if !is_plugin_wildcard(&options.identifier) {
    // same directory `permission new` writes to
    let dir = match &tauri_dir {
      Some(t) => t.clone(),
      None => std::env::current_dir().context("failed to resolve current directory")?,
    };
    let permissions_dir = dir.join("permissions");
    if permissions_dir.exists() {
      rm_permission_files(&options.identifier, &permissions_dir)?;
    }
  }

  if let Some(tauri_dir) = tauri_dir {
    let capabilities_dir = tauri_dir.join("capabilities");
    if capabilities_dir.exists() {
      rm_permission_from_capabilities(&options.identifier, &capabilities_dir)?;
    }
  }

  Ok(())
}

fn is_plugin_wildcard(identifier: &str) -> bool {
  matches!(identifier.split_once(':'), Some((_, "*")))
}

#[cfg(test)]
mod tests {
  use super::identifier_match;

  #[test]
  fn wildcard_matches_only_the_plugin_prefix() {
    assert!(identifier_match("os:*", "os:default"));
    assert!(identifier_match("os:*", "os:allow-platform"));
    assert!(identifier_match("os:*", "os"));

    assert!(!identifier_match("os:*", "positioner:default"));
    assert!(!identifier_match("os:*", "core:window:allow-close"));
    assert!(!identifier_match("os:*", "cos:default"));
    assert!(!identifier_match("os:*", "osx:default"));
  }

  #[test]
  fn wildcard_does_not_match_nested_plugin_names() {
    assert!(identifier_match("core:*", "core:window:allow-close"));
    assert!(!identifier_match("window:*", "core:window:allow-close"));
  }

  #[test]
  fn exact_identifier_match() {
    assert!(identifier_match("fs:default", "fs:default"));
    assert!(!identifier_match("fs:default", "fs:default-extra"));
    assert!(!identifier_match("fs:default", "fs:allow-read"));
    assert!(identifier_match("my-permission", "my-permission"));
  }
}
