// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! ACL items that are only useful inside of build script/codegen context.

use std::{
  collections::HashMap,
  env::vars_os,
  fs::File,
  io::{BufWriter, Write},
  num::NonZeroU64,
  path::PathBuf,
};

use crate::acl::Error;
use crate::acl::{Commands, Permission, Scopes};
use schemars::{
  schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SubschemaValidation},
  schema_for,
};
use serde::Deserialize;

/// Cargo cfg key for permissions file paths
pub const PERMISSION_FILES_PATH_KEY: &str = "PERMISSION_FILES_PATH";

/// Allowed permission file extensions
pub const PERMISSION_FILE_EXTENSIONS: &[&str] = &["json", "toml"];

/// Known filename of a permission schema
pub const PERMISSION_SCHEMA_FILE_NAME: &str = ".schema.json";

/// A set of permissions or other permission sets.
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
  /// The default permission set for the plugin
  pub default: Option<DefaultPermission>,

  /// A list of permissions sets defined
  #[serde(default)]
  pub set: Vec<PermissionSet>,

  /// Test something!!
  pub test: Option<PermissionSet>,

  /// A list of inlined permissions
  #[serde(default)]
  pub permission: Vec<Permission>,
}

/// Write the permissions to a temporary directory and pass it to the immediate consuming crate.
pub fn define_permissions(pattern: &str) -> Result<Vec<PermissionFile>, Error> {
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

  parse_permissions(permission_files)
}

fn permissions_schema(permissions: &[PermissionFile]) -> RootSchema {
  let mut schema = schema_for!(PermissionFile);

  fn schema_from(id: &str, description: Option<&str>) -> Schema {
    Schema::Object(SchemaObject {
      metadata: Some(Box::new(Metadata {
        description: description.map(|d| format!("{id} -> {d}")),
        ..Default::default()
      })),
      instance_type: Some(InstanceType::String.into()),
      enum_values: Some(vec![serde_json::Value::String(id.into())]),
      ..Default::default()
    })
  }

  let mut permission_schemas = Vec::new();
  for file in permissions {
    if let Some(permission) = &file.default {
      permission_schemas.push(schema_from("default", permission.description.as_deref()));
    }

    permission_schemas.extend(
      file
        .set
        .iter()
        .map(|set| schema_from(&set.identifier, Some(set.description.as_str())))
        .collect::<Vec<_>>(),
    );

    permission_schemas.extend(
      file
        .permission
        .iter()
        .map(|permission| schema_from(&permission.identifier, permission.description.as_deref()))
        .collect::<Vec<_>>(),
    );
  }

  if let Some(Schema::Object(obj)) = schema.definitions.get_mut("PermissionSet") {
    if let Some(Schema::Object(permissions_prop_schema)) =
      obj.object().properties.get_mut("permissions")
    {
      permissions_prop_schema.array().items.replace(
        Schema::Object(SchemaObject {
          reference: Some("#/definitions/PermissionKind".into()),
          ..Default::default()
        })
        .into(),
      );

      schema.definitions.insert(
        "PermissionKind".into(),
        Schema::Object(SchemaObject {
          instance_type: Some(InstanceType::String.into()),
          subschemas: Some(Box::new(SubschemaValidation {
            one_of: Some(permission_schemas),
            ..Default::default()
          })),
          ..Default::default()
        }),
      );
    }
  }

  schema
}

/// Generate and write a schema based on the format of a [`PermissionFile`].
pub fn generate_schema(permissions: &[PermissionFile]) -> Result<(), Error> {
  let schema = permissions_schema(permissions);
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let out_path = PathBuf::from("permissions").join(PERMISSION_SCHEMA_FILE_NAME);

  let mut schema_file = BufWriter::new(File::create(out_path).map_err(Error::CreateFile)?);
  write!(schema_file, "{schema_str}").map_err(Error::WriteFile)?;
  Ok(())
}

/// Read all permissions listed from the defined cargo cfg key value.
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
