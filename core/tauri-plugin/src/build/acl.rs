// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  env::vars_os,
  fs::File,
  io::{BufWriter, Write},
  num::NonZeroU64,
  path::PathBuf,
};

use super::Error;
use serde::Deserialize;
use tauri_utils::acl::{Commands, Permission, Scopes};

const PERMISSION_FILES_PATH_KEY: &str = "PERMISSION_FILES_PATH";
const PERMISSION_FILE_EXTENSIONS: &[&str] = &["json", "toml"];
pub(crate) const PERMISSION_SCHEMA_FILE_NAME: &str = ".schema.json";

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PermissionSet {
  /// A unique identifier for the permission.
  pub identifier: String,

  /// Human-readable description of what the permission does.
  pub description: String,

  /// All permissions this set contains.
  pub permissions: Vec<String>,
}

/// The default permission of the plugin.
///
/// Works similarly to a permission with the "default" identifier.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DefaultPermission {
  /// The version of the permission.
  pub version: Option<NonZeroU64>,

  /// Human-readable description of what the permission does.
  pub description: Option<String>,

  /// Allowed or denied commands when using this permission.
  #[serde(default)]
  pub commands: Commands,

  /// Allowed or denied scoped when using this permission.
  #[serde(default)]
  pub scope: Scopes,
}

/// Permission file that can define a default permission, a set of permissions or a list of inlined permissions.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PermissionFile {
  pub default: Option<DefaultPermission>,
  #[serde(default)]
  pub set: Vec<PermissionSet>,
  #[serde(default)]
  pub permission: Vec<Permission>,
}

pub(crate) fn define_permissions(pattern: &str) -> Result<(), Error> {
  let permission_files = glob::glob(pattern)?
    .flatten()
    .flat_map(|p| p.canonicalize())
    // filter extension
    .filter(|p| {
      p.extension()
        .and_then(|e| e.to_str())
        .map(|e| PERMISSION_FILE_EXTENSIONS.contains(&e))
        .unwrap_or_default()
    })
    // filter schema file
    .filter(|p| {
      p.file_name()
        .map(|name| name != PERMISSION_SCHEMA_FILE_NAME)
        .unwrap_or(true)
    })
    .collect::<Vec<PathBuf>>();

  for path in &permission_files {
    println!("cargo:rerun-if-changed={}", path.display());
  }

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

pub(crate) fn generate_schema() -> Result<(), Error> {
  let schema = schemars::schema_for!(PermissionFile);
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let out_path = PathBuf::from("permissions").join(PERMISSION_SCHEMA_FILE_NAME);

  let mut schema_file = BufWriter::new(File::create(out_path).map_err(Error::CreateFile)?);
  write!(schema_file, "{schema_str}").map_err(Error::WriteFile)?;
  Ok(())
}

pub fn read_permissions() -> Result<HashMap<String, Vec<PermissionFile>>, Error> {
  let mut permissions_map = HashMap::new();

  for (key, value) in vars_os() {
    let key = key.to_string_lossy();

    if let Some(plugin_crate_name_var) = key
      .strip_prefix("DEP_")
      .and_then(|v| v.strip_suffix(&format!("_{PERMISSION_FILES_PATH_KEY}")))
    {
      let permissions_path = PathBuf::from(value);
      let permissions_str = std::fs::read_to_string(&permissions_path).map_err(Error::ReadFile)?;
      let permissions: Vec<PathBuf> = serde_json::from_str(&permissions_str)?;
      let permissions = parse_permissions(permissions)?;

      let plugin_crate_name = plugin_crate_name_var.to_lowercase().replace('_', "-");
      permissions_map.insert(
        plugin_crate_name
          .strip_prefix("tauri-plugin-")
          .map(|n| n.to_string())
          .unwrap_or(plugin_crate_name),
        permissions,
      );
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
