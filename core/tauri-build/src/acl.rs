// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{BTreeMap, BTreeSet},
  fs::{copy, create_dir_all, read_to_string, File},
  io::{BufWriter, Write},
  path::PathBuf,
};

use anyhow::{Context, Result};
use schemars::{
  schema::{
    InstanceType, Metadata, ObjectValidation, RootSchema, Schema, SchemaObject, SubschemaValidation,
  },
  schema_for,
};
use tauri_utils::{
  acl::{build::CapabilityFile, capability::Capability, plugin::Manifest, Value},
  platform::Target,
};

const CAPABILITIES_SCHEMA_FILE_NAME: &str = "schema.json";
/// Path of the folder where schemas are saved.
const CAPABILITIES_SCHEMA_FOLDER_PATH: &str = "capabilities/schemas";
const CAPABILITIES_FILE_NAME: &str = "capabilities.json";
const PLUGIN_MANIFESTS_FILE_NAME: &str = "plugin-manifests.json";

fn capabilities_schema(plugin_manifests: &BTreeMap<String, Manifest>) -> RootSchema {
  let mut schema = schema_for!(CapabilityFile);

  fn schema_from(plugin: &str, id: &str, description: Option<&str>) -> Schema {
    Schema::Object(SchemaObject {
      metadata: Some(Box::new(Metadata {
        description: description
          .as_ref()
          .map(|d| format!("{plugin}:{id} -> {d}")),
        ..Default::default()
      })),
      instance_type: Some(InstanceType::String.into()),
      enum_values: Some(vec![serde_json::Value::String(format!("{plugin}:{id}"))]),
      ..Default::default()
    })
  }

  fn value_schema(value: &Value) -> Schema {
    let mut schema = SchemaObject::default();

    match value {
      Value::Map(map) => {
        schema.instance_type.replace(InstanceType::Object.into());
        for (key, value) in map {
          schema
            .object()
            .properties
            .insert(key.clone(), value_schema(value));
        }
      }
      Value::Bool(_) => {
        schema.instance_type.replace(InstanceType::Boolean.into());
      }
      Value::String(_) => {
        schema.instance_type.replace(InstanceType::String.into());
      }
      Value::Number(_) => {
        schema.instance_type.replace(InstanceType::Number.into());
      }
      Value::List(list) => {
        schema.instance_type.replace(InstanceType::Array.into());

        let mut any_of: Vec<Schema> = list.iter().map(value_schema).collect();
        any_of.dedup();
        let item_schema = Schema::Object(SchemaObject {
          subschemas: Some(Box::new(SubschemaValidation {
            any_of: Some(any_of),
            ..Default::default()
          })),
          ..Default::default()
        });
        schema.array().items.replace(item_schema.into());
      }
    }

    schema.into()
  }

  let mut permission_schemas = Vec::new();

  for (plugin, manifest) in plugin_manifests {
    for (set_id, set) in &manifest.permission_sets {
      permission_schemas.push(schema_from(plugin, set_id, Some(&set.description)));
    }

    if let Some(default) = &manifest.default_permission {
      permission_schemas.push(schema_from(
        plugin,
        "default",
        Some(default.description.as_ref()),
      ));
    }

    for (permission_id, permission) in &manifest.permissions {
      permission_schemas.push(schema_from(
        plugin,
        permission_id,
        permission.description.as_deref(),
      ));
    }
  }

  if let Some(Schema::Object(obj)) = schema.definitions.get_mut("Identifier") {
    obj.object = None;
    obj.instance_type = None;
    obj.metadata.as_mut().map(|metadata| {
      metadata
        .description
        .replace("Permission identifier".to_string());
      metadata
    });
    obj.subschemas.replace(Box::new(SubschemaValidation {
      one_of: Some(permission_schemas),
      ..Default::default()
    }));
  }

  if let Some(Schema::Object(obj)) = schema.definitions.get_mut("PermissionEntry") {
    let any_of = obj.subschemas().any_of.as_mut().unwrap();
    let scope_extended_schema = any_of.iter_mut().last().unwrap();

    if let Schema::Object(scope_extended_schema_obj) = scope_extended_schema {
      let mut one_of = Vec::new();

      for (plugin, manifest) in plugin_manifests {
        let mut scopes = Vec::new();
        for permission in manifest.permissions.values() {
          if let Some(allow) = &permission.scope.allow {
            scopes.extend(allow.clone());
          }
          if let Some(deny) = &permission.scope.deny {
            scopes.extend(deny.clone());
          }
        }

        if scopes.is_empty() {
          continue;
        }

        let scope_schema = value_schema(&Value::List(scopes));

        let mut required = BTreeSet::new();
        required.insert("identifier".to_string());

        let mut object = ObjectValidation {
          required,
          ..Default::default()
        };

        let mut permission_schemas = Vec::new();
        if let Some(default) = &manifest.default_permission {
          permission_schemas.push(schema_from(plugin, "default", Some(&default.description)));
        }
        for set in manifest.permission_sets.values() {
          permission_schemas.push(schema_from(plugin, &set.identifier, Some(&set.description)));
        }
        for permission in manifest.permissions.values() {
          permission_schemas.push(schema_from(
            plugin,
            &permission.identifier,
            permission.description.as_deref(),
          ));
        }

        let identifier_schema = Schema::Object(SchemaObject {
          subschemas: Some(Box::new(SubschemaValidation {
            one_of: Some(permission_schemas),
            ..Default::default()
          })),
          ..Default::default()
        });

        object
          .properties
          .insert("identifier".to_string(), identifier_schema);
        object
          .properties
          .insert("allow".to_string(), scope_schema.clone());
        object
          .properties
          .insert("deny".to_string(), scope_schema.clone());

        one_of.push(Schema::Object(SchemaObject {
          instance_type: Some(InstanceType::Object.into()),
          object: Some(Box::new(object)),
          ..Default::default()
        }));
      }

      scope_extended_schema_obj.object = None;
      scope_extended_schema_obj
        .subschemas
        .replace(Box::new(SubschemaValidation {
          one_of: Some(one_of),
          ..Default::default()
        }));
    }
  }

  schema
}

pub fn generate_schema(
  plugin_manifests: &BTreeMap<String, Manifest>,
  target: Target,
) -> Result<()> {
  let schema = capabilities_schema(plugin_manifests);
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let out_dir = PathBuf::from(CAPABILITIES_SCHEMA_FOLDER_PATH);
  create_dir_all(&out_dir).context("unable to create schema output directory")?;

  let schema_path = out_dir.join(format!("{target}-{CAPABILITIES_SCHEMA_FILE_NAME}"));
  let mut schema_file = BufWriter::new(File::create(&schema_path)?);
  write!(schema_file, "{schema_str}")?;

  copy(
    schema_path,
    out_dir.join(format!(
      "{}-{CAPABILITIES_SCHEMA_FILE_NAME}",
      if target.is_desktop() {
        "desktop"
      } else {
        "mobile"
      }
    )),
  )?;

  Ok(())
}

pub fn save_capabilities(capabilities: &BTreeMap<String, Capability>) -> Result<PathBuf> {
  let capabilities_path =
    PathBuf::from(CAPABILITIES_SCHEMA_FOLDER_PATH).join(CAPABILITIES_FILE_NAME);
  let capabilities_json = serde_json::to_string(&capabilities)?;
  if capabilities_json != read_to_string(&capabilities_path).unwrap_or_default() {
    std::fs::write(&capabilities_path, capabilities_json)?;
  }
  Ok(capabilities_path)
}

pub fn save_plugin_manifests(plugin_manifests: &BTreeMap<String, Manifest>) -> Result<PathBuf> {
  let plugin_manifests_path =
    PathBuf::from(CAPABILITIES_SCHEMA_FOLDER_PATH).join(PLUGIN_MANIFESTS_FILE_NAME);
  let plugin_manifests_json = serde_json::to_string(&plugin_manifests)?;
  if plugin_manifests_json != read_to_string(&plugin_manifests_path).unwrap_or_default() {
    std::fs::write(&plugin_manifests_path, plugin_manifests_json)?;
  }
  Ok(plugin_manifests_path)
}

pub fn get_plugin_manifests() -> Result<BTreeMap<String, Manifest>> {
  let permission_map =
    tauri_utils::acl::build::read_permissions().context("failed to read plugin permissions")?;

  let mut processed = BTreeMap::new();
  for (plugin_name, permission_files) in permission_map {
    processed.insert(plugin_name, Manifest::from_files(permission_files));
  }

  Ok(processed)
}

pub fn validate_capabilities(
  plugin_manifests: &BTreeMap<String, Manifest>,
  capabilities: &BTreeMap<String, Capability>,
) -> Result<()> {
  let target = tauri_utils::platform::Target::from_triple(&std::env::var("TARGET").unwrap());

  for capability in capabilities.values() {
    if !capability.platforms.contains(&target) {
      continue;
    }

    for permission_entry in &capability.permissions {
      let permission_id = permission_entry.identifier();
      if let Some((plugin_name, permission_name)) = permission_id.get().split_once(':') {
        let permission_exists = plugin_manifests
          .get(plugin_name)
          .map(|manifest| {
            if permission_name == "default" {
              manifest.default_permission.is_some()
            } else {
              manifest.permissions.contains_key(permission_name)
                || manifest.permission_sets.contains_key(permission_name)
            }
          })
          .unwrap_or(false);

        if !permission_exists {
          let mut available_permissions = Vec::new();
          for (plugin, manifest) in plugin_manifests {
            if manifest.default_permission.is_some() {
              available_permissions.push(format!("{plugin}:default"));
            }
            for p in manifest.permissions.keys() {
              available_permissions.push(format!("{plugin}:{p}"));
            }
            for p in manifest.permission_sets.keys() {
              available_permissions.push(format!("{plugin}:{p}"));
            }
          }

          anyhow::bail!(
            "Permission {} not found, expected one of {}",
            permission_id.get(),
            available_permissions.join(", ")
          );
        }
      }
    }
  }

  Ok(())
}
