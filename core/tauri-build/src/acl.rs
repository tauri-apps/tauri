// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{BTreeMap, BTreeSet},
  fs::{copy, create_dir_all, read_to_string, write},
  path::PathBuf,
};

use anyhow::{Context, Result};
use schemars::{
  schema::{
    ArrayValidation, InstanceType, Metadata, ObjectValidation, RootSchema, Schema, SchemaObject,
    SubschemaValidation,
  },
  schema_for,
};
use tauri_utils::{
  acl::{
    capability::{Capability, CapabilityFile},
    plugin::Manifest,
  },
  platform::Target,
};

const CAPABILITIES_SCHEMA_FILE_NAME: &str = "schema.json";
/// Path of the folder where schemas are saved.
const CAPABILITIES_SCHEMA_FOLDER_PATH: &str = "gen/schemas";
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

  let mut definitions = Vec::new();

  if let Some(Schema::Object(obj)) = schema.definitions.get_mut("PermissionEntry") {
    let permission_entry_any_of_schemas = obj.subschemas().any_of.as_mut().unwrap();

    if let Schema::Object(mut scope_extended_schema_obj) =
      permission_entry_any_of_schemas.remove(permission_entry_any_of_schemas.len() - 1)
    {
      let mut global_scope_one_of = Vec::new();

      for (plugin, manifest) in plugin_manifests {
        if let Some(global_scope_schema) = &manifest.global_scope_schema {
          let global_scope_schema_def: RootSchema =
            serde_json::from_value(global_scope_schema.clone())
              .unwrap_or_else(|e| panic!("invalid JSON schema for plugin {plugin}: {e}"));

          let global_scope_schema = Schema::Object(SchemaObject {
            array: Some(Box::new(ArrayValidation {
              items: Some(Schema::Object(global_scope_schema_def.schema).into()),
              ..Default::default()
            })),
            ..Default::default()
          });

          definitions.push(global_scope_schema_def.definitions);

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
            .insert("allow".to_string(), global_scope_schema.clone());
          object
            .properties
            .insert("deny".to_string(), global_scope_schema);

          global_scope_one_of.push(Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(object)),
            ..Default::default()
          }));
        }
      }

      if !global_scope_one_of.is_empty() {
        scope_extended_schema_obj.object = None;
        scope_extended_schema_obj
          .subschemas
          .replace(Box::new(SubschemaValidation {
            one_of: Some(global_scope_one_of),
            ..Default::default()
          }));

        permission_entry_any_of_schemas.push(scope_extended_schema_obj.into());
      };
    }
  }

  for definitions_map in definitions {
    schema.definitions.extend(definitions_map);
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
  if schema_str != read_to_string(&schema_path).unwrap_or_default() {
    write(&schema_path, schema_str)?;

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
  }

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
  let mut global_scope_map = tauri_utils::acl::build::read_global_scope_schemas()
    .context("failed to read global scope schemas")?;

  let mut processed = BTreeMap::new();
  for (plugin_name, permission_files) in permission_map {
    let manifest = Manifest::new(permission_files, global_scope_map.remove(&plugin_name));
    processed.insert(plugin_name, manifest);
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
