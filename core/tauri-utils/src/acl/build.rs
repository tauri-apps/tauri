// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! ACL items that are only useful inside of build script/codegen context.

use std::{
  collections::{BTreeMap, HashMap},
  env::{current_dir, vars_os},
  fs::{create_dir_all, File},
  io::{BufWriter, Write},
  path::{Path, PathBuf},
};

use crate::acl::Error;
use schemars::{
  schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SubschemaValidation},
  schema_for,
};
use serde::Deserialize;

use super::{capability::Capability, plugin::PermissionFile};

/// Cargo cfg key for permissions file paths
pub const PERMISSION_FILES_PATH_KEY: &str = "PERMISSION_FILES_PATH";

/// Allowed permission file extensions
pub const PERMISSION_FILE_EXTENSIONS: &[&str] = &["json", "toml"];

/// Known filename of a permission schema
pub const PERMISSION_SCHEMA_FILE_NAME: &str = ".schema.json";

/// Allowed capability file extensions
const CAPABILITY_FILE_EXTENSIONS: &[&str] = &["json", "toml"];

/// Known folder name of the capability schemas
const CAPABILITIES_SCHEMA_FOLDER_NAME: &str = "schemas";

const CORE_PLUGIN_PERMISSIONS_TOKEN: &str = "__CORE_PLUGIN__";

/// Capability formats accepted in a capability file.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum CapabilityFile {
  /// A single capability.
  Capability(Capability),
  /// A list of capabilities.
  List {
    /// The list of capabilities.
    capabilities: Vec<Capability>,
  },
}

/// Write the permissions to a temporary directory and pass it to the immediate consuming crate.
pub fn define_permissions(pattern: &str, pkg_name: &str) -> Result<Vec<PermissionFile>, Error> {
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

  let permission_files_path = std::env::temp_dir().join(format!("{}-permission-files", pkg_name));
  std::fs::write(
    &permission_files_path,
    serde_json::to_string(&permission_files)?,
  )
  .map_err(Error::WriteFile)?;

  if let Some(plugin_name) = pkg_name.strip_prefix("tauri:") {
    println!(
      "cargo:{plugin_name}{CORE_PLUGIN_PERMISSIONS_TOKEN}_{PERMISSION_FILES_PATH_KEY}={}",
      permission_files_path.display()
    );
  } else {
    println!(
      "cargo:{PERMISSION_FILES_PATH_KEY}={}",
      permission_files_path.display()
    );
  }

  parse_permissions(permission_files)
}

/// Parses all capability files with the given glob pattern.
pub fn parse_capabilities(
  capabilities_path_pattern: &str,
) -> Result<BTreeMap<String, Capability>, Error> {
  let mut capabilities_map = BTreeMap::new();

  for path in glob::glob(capabilities_path_pattern)?
    .flatten() // filter extension
    .filter(|p| {
      p.extension()
        .and_then(|e| e.to_str())
        .map(|e| CAPABILITY_FILE_EXTENSIONS.contains(&e))
        .unwrap_or_default()
    })
    // filter schema files
    .filter(|p| p.parent().unwrap().file_name().unwrap() != CAPABILITIES_SCHEMA_FOLDER_NAME)
  {
    println!("cargo:rerun-if-changed={}", path.display());

    let capability_file = std::fs::read_to_string(&path).map_err(Error::ReadFile)?;
    let ext = path.extension().unwrap().to_string_lossy().to_string();
    let capability: CapabilityFile = match ext.as_str() {
      "toml" => toml::from_str(&capability_file)?,
      "json" => serde_json::from_str(&capability_file)?,
      _ => return Err(Error::UnknownCapabilityFormat(ext)),
    };

    match capability {
      CapabilityFile::Capability(capability) => {
        capabilities_map.insert(capability.identifier.clone(), capability);
      }
      CapabilityFile::List { capabilities } => {
        for capability in capabilities {
          capabilities_map.insert(capability.identifier.clone(), capability);
        }
      }
    }
  }

  Ok(capabilities_map)
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
pub fn generate_schema<P: AsRef<Path>>(
  permissions: &[PermissionFile],
  out_dir: P,
) -> Result<(), Error> {
  let schema = permissions_schema(permissions);
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();

  let out_dir = out_dir.as_ref();
  create_dir_all(out_dir).expect("unable to create schema output directory");

  let mut schema_file = BufWriter::new(
    File::create(out_dir.join(PERMISSION_SCHEMA_FILE_NAME)).map_err(Error::CreateFile)?,
  );
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
      .map(|v| {
        v.strip_suffix(CORE_PLUGIN_PERMISSIONS_TOKEN)
          .and_then(|v| v.strip_prefix("TAURI_"))
          .unwrap_or(v)
      })
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

/// Autogenerate permission files for a list of commands.
pub fn autogenerate_command_permissions(path: &Path, commands: &[&str], license_header: &str) {
  if !path.exists() {
    create_dir_all(path).expect("unable to create autogenerated commands dir");
  }

  let cwd = current_dir().unwrap();
  let components_len = path.strip_prefix(&cwd).unwrap_or(path).components().count();
  let schema_path = (1..components_len)
    .map(|_| "..")
    .collect::<PathBuf>()
    .join(PERMISSION_SCHEMA_FILE_NAME);

  for command in commands {
    let slugified_command = command.replace('_', "-");
    let toml = format!(
      r###"{license_header}# Automatically generated - DO NOT EDIT!

"$schema" = "{schema_path}"

[[permission]]
identifier = "allow-{slugified_command}"
description = "Enables the {command} command without any pre-configured scope."
commands.allow = ["{command}"]

[[permission]]
identifier = "deny-{slugified_command}"
description = "Denies the {command} command without any pre-configured scope."
commands.deny = ["{command}"]
"###,
      command = command,
      slugified_command = slugified_command,
      schema_path = schema_path.display().to_string().replace('\\', "\\\\")
    );

    std::fs::write(path.join(format!("{command}.toml")), toml)
      .unwrap_or_else(|_| panic!("unable to autogenerate ${command}.toml"));
  }
}
