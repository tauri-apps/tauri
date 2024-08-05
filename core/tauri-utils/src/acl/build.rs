// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! ACL items that are only useful inside of build script/codegen context.

use std::{
  collections::{BTreeMap, HashMap},
  env::{current_dir, vars_os},
  fs::{create_dir_all, read_to_string, write},
  path::{Path, PathBuf},
};

use crate::acl::Error;
use schemars::{
  schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SubschemaValidation},
  schema_for,
};

use super::{
  capability::{Capability, CapabilityFile},
  manifest::PermissionFile,
  PERMISSION_SCHEMA_FILE_NAME,
};

/// Known name of the folder containing autogenerated permissions.
pub const AUTOGENERATED_FOLDER_NAME: &str = "autogenerated";

/// Cargo cfg key for permissions file paths
pub const PERMISSION_FILES_PATH_KEY: &str = "PERMISSION_FILES_PATH";

/// Cargo cfg key for global scope schemas
pub const GLOBAL_SCOPE_SCHEMA_PATH_KEY: &str = "GLOBAL_SCOPE_SCHEMA_PATH";

/// Allowed permission file extensions
pub const PERMISSION_FILE_EXTENSIONS: &[&str] = &["json", "toml"];

/// Known foldername of the permission schema files
pub const PERMISSION_SCHEMAS_FOLDER_NAME: &str = "schemas";

/// Known filename of the permission documentation file
pub const PERMISSION_DOCS_FILE_NAME: &str = "reference.md";

/// Allowed capability file extensions
const CAPABILITY_FILE_EXTENSIONS: &[&str] = &["json", "toml"];

/// Known folder name of the capability schemas
const CAPABILITIES_SCHEMA_FOLDER_NAME: &str = "schemas";

const CORE_PLUGIN_PERMISSIONS_TOKEN: &str = "__CORE_PLUGIN__";

/// Write the permissions to a temporary directory and pass it to the immediate consuming crate.
pub fn define_permissions<F: Fn(&Path) -> bool>(
  pattern: &str,
  pkg_name: &str,
  out_dir: &Path,
  filter_fn: F,
) -> Result<Vec<PermissionFile>, Error> {
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
    .filter(|p| filter_fn(p))
    // filter schemas
    .filter(|p| p.parent().unwrap().file_name().unwrap() != PERMISSION_SCHEMAS_FOLDER_NAME)
    .collect::<Vec<PathBuf>>();

  let permission_files_path =
    out_dir.join(format!("{}-permission-files", pkg_name.replace(':', "-")));
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

/// Define the global scope schema JSON file path if it exists and pass it to the immediate consuming crate.
pub fn define_global_scope_schema(
  schema: schemars::schema::RootSchema,
  pkg_name: &str,
  out_dir: &Path,
) -> Result<(), Error> {
  let path = out_dir.join("global-scope.json");
  write(&path, serde_json::to_vec(&schema)?).map_err(Error::WriteFile)?;

  if let Some(plugin_name) = pkg_name.strip_prefix("tauri:") {
    println!(
      "cargo:{plugin_name}{CORE_PLUGIN_PERMISSIONS_TOKEN}_{GLOBAL_SCOPE_SCHEMA_PATH_KEY}={}",
      path.display()
    );
  } else {
    println!("cargo:{GLOBAL_SCOPE_SCHEMA_PATH_KEY}={}", path.display());
  }

  Ok(())
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
    // TODO: remove this before stable
    .filter(|p| p.parent().unwrap().file_name().unwrap() != CAPABILITIES_SCHEMA_FOLDER_NAME)
  {
    match CapabilityFile::load(&path)? {
      CapabilityFile::Capability(capability) => {
        capabilities_map.insert(capability.identifier.clone(), capability);
      }
      CapabilityFile::List(capabilities) | CapabilityFile::NamedList { capabilities } => {
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

  let out_dir = out_dir.as_ref().join(PERMISSION_SCHEMAS_FOLDER_NAME);
  create_dir_all(&out_dir).expect("unable to create schema output directory");

  let schema_path = out_dir.join(PERMISSION_SCHEMA_FILE_NAME);
  if schema_str != read_to_string(&schema_path).unwrap_or_default() {
    write(schema_path, schema_str).map_err(Error::WriteFile)?;
  }

  Ok(())
}

/// Generate a markdown documentation page containing the list of permissions of the plugin.
pub fn generate_docs(
  permissions: &[PermissionFile],
  out_dir: &Path,
  plugin_identifier: &str,
) -> Result<(), Error> {
  let mut permission_table = "".to_string();
  let permission_table_header =
    "### Permission Table \n\n<table>\n<tr>\n<th>Identifier</th>\n<th>Description</th>\n</tr>\n"
      .to_string();

  let mut default_permission = "## Default Permission\n\n".to_string();
  let mut contains_default = false;

  fn docs_from(id: &str, description: Option<&str>, plugin_identifier: &str) -> String {
    let mut docs = format!("\n<tr>\n<td>\n\n`{plugin_identifier}:{id}`\n\n</td>\n");
    if let Some(d) = description {
      docs.push_str(&format!("<td>\n\n{d}\n\n</td>"));
    }
    docs.push_str("\n</tr>");
    docs
  }

  for permission in permissions {
    for set in &permission.set {
      permission_table.push_str(&docs_from(
        &set.identifier,
        Some(&set.description),
        plugin_identifier,
      ));
      permission_table.push('\n');
    }

    if let Some(default) = &permission.default {
      default_permission.push_str(default.description.as_deref().unwrap_or_default());
      default_permission.push('\n');
      default_permission.push('\n');
      for permission in &default.permissions {
        default_permission.push_str(&format!("- `{permission}`"));
        default_permission.push('\n');
      }

      contains_default = true;
    }

    for permission in &permission.permission {
      permission_table.push_str(&docs_from(
        &permission.identifier,
        permission.description.as_deref(),
        plugin_identifier,
      ));
      permission_table.push('\n');
    }
  }
  permission_table.push_str("</table>");

  if !contains_default {
    default_permission = "".to_string();
  }

  let docs = format!("{default_permission}\n{permission_table_header}\n{permission_table}\n");

  let reference_path = out_dir.join(PERMISSION_DOCS_FILE_NAME);
  if docs != read_to_string(&reference_path).unwrap_or_default() {
    std::fs::write(reference_path, docs).map_err(Error::WriteFile)?;
  }

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

/// Read all global scope schemas listed from the defined cargo cfg key value.
pub fn read_global_scope_schemas() -> Result<HashMap<String, serde_json::Value>, Error> {
  let mut permissions_map = HashMap::new();

  for (key, value) in vars_os() {
    let key = key.to_string_lossy();

    if let Some(plugin_crate_name_var) = key
      .strip_prefix("DEP_")
      .and_then(|v| v.strip_suffix(&format!("_{GLOBAL_SCOPE_SCHEMA_PATH_KEY}")))
      .map(|v| {
        v.strip_suffix(CORE_PLUGIN_PERMISSIONS_TOKEN)
          .and_then(|v| v.strip_prefix("TAURI_"))
          .unwrap_or(v)
      })
    {
      let path = PathBuf::from(value);
      let json = std::fs::read_to_string(&path).map_err(Error::ReadFile)?;
      let schema: serde_json::Value = serde_json::from_str(&json)?;

      let plugin_crate_name = plugin_crate_name_var.to_lowercase().replace('_', "-");
      permissions_map.insert(
        plugin_crate_name
          .strip_prefix("tauri-plugin-")
          .map(|n| n.to_string())
          .unwrap_or(plugin_crate_name),
        schema,
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
pub fn autogenerate_command_permissions(
  path: &Path,
  commands: &[&str],
  license_header: &str,
  schema_ref: bool,
) {
  if !path.exists() {
    create_dir_all(path).expect("unable to create autogenerated commands dir");
  }

  let schema_entry = if schema_ref {
    let cwd = current_dir().unwrap();
    let components_len = path.strip_prefix(&cwd).unwrap_or(path).components().count();
    let schema_path = (1..components_len)
      .map(|_| "..")
      .collect::<PathBuf>()
      .join(PERMISSION_SCHEMAS_FOLDER_NAME)
      .join(PERMISSION_SCHEMA_FILE_NAME);
    format!(
      "\n\"$schema\" = \"{}\"\n",
      dunce::simplified(&schema_path)
        .display()
        .to_string()
        .replace('\\', "/")
    )
  } else {
    "".to_string()
  };

  for command in commands {
    let slugified_command = command.replace('_', "-");
    let toml = format!(
      r###"{license_header}# Automatically generated - DO NOT EDIT!
{schema_entry}
[[permission]]
identifier = "allow-{slugified_command}"
description = "Enables the {command} command without any pre-configured scope."
commands.allow = ["{command}"]

[[permission]]
identifier = "deny-{slugified_command}"
description = "Denies the {command} command without any pre-configured scope."
commands.deny = ["{command}"]
"###,
    );

    let out_path = path.join(format!("{command}.toml"));
    if toml != read_to_string(&out_path).unwrap_or_default() {
      std::fs::write(out_path, toml)
        .unwrap_or_else(|_| panic!("unable to autogenerate ${command}.toml"));
    }
  }
}
