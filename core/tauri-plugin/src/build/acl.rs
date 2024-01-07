use std::{collections::HashMap, env::vars_os, path::PathBuf};

use super::Error;
use serde::Deserialize;
use tauri_utils::acl::{Identifier, Permission};

const PERMISSION_FILES_PATH_KEY: &str = "PERMISSION_FILES_PATH";

#[derive(Debug, Deserialize)]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  pub identifier: Identifier,

  /// Human-readable description of what the permission does.
  pub description: String,

  /// All permissions this set contains.
  pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionFile {
  pub default: Option<Permission>,
  pub set: Option<Vec<PermissionSet>>,
  pub permission: Option<Vec<Permission>>,
}

pub fn define_permissions(pattern: &str) -> Result<(), Error> {
  let permission_files = glob::glob(pattern)?
    .flatten()
    .map(|p| p.canonicalize())
    .flatten()
    .collect::<Vec<PathBuf>>();

  let permission_files_path = std::env::temp_dir().join(format!(
    "{}-permission-files",
    std::env::var("CARGO_PKG_NAME").unwrap()
  ));
  std::fs::write(
    &permission_files_path,
    serde_json::to_string(&permission_files)?,
  )
  .map_err(Error::WriteFile)?;
  println!(
    "cargo:{PERMISSION_FILES_PATH_KEY}={}",
    permission_files_path.display()
  );

  Ok(())
}

pub fn read_permissions() -> Result<HashMap<String, Vec<PermissionFile>>, Error> {
  let mut permissions_map = HashMap::new();

  for (key, value) in vars_os() {
    let key = key.to_string_lossy();

    if let Some(plugin_crate_name) = key
      .strip_prefix("DEP_")
      .and_then(|v| v.strip_suffix(&format!("_{PERMISSION_FILES_PATH_KEY}")))
    {
      let permissions_path = PathBuf::from(value);
      let permissions_str = std::fs::read_to_string(&permissions_path).map_err(Error::ReadFile)?;
      let permissions: Vec<PathBuf> = serde_json::from_str(&permissions_str)?;
      let permissions = parse_permissions(permissions)?;

      permissions_map.insert(plugin_crate_name.to_lowercase(), permissions);
    }
  }

  Ok(permissions_map)
}

fn parse_permissions(paths: Vec<PathBuf>) -> Result<Vec<PermissionFile>, Error> {
  let mut permissions = Vec::new();
  for path in paths {
    let permission_file = std::fs::read_to_string(&path).map_err(Error::ReadFile)?;
    let ext = path.extension().unwrap().to_string_lossy().to_string();
    let permission: PermissionFile = match ext.as_str() {
      "toml" => toml::from_str(&permission_file)?,
      "json" => serde_json::from_str(&permission_file)?,
      _ => return Err(Error::UnknownPermissionFormat(ext)),
    };
    permissions.push(permission);
  }
  Ok(permissions)
}
