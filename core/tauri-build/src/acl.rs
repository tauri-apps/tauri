// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  fs::File,
  io::{BufWriter, Write},
  path::PathBuf,
};

use anyhow::{Context, Result};
use schemars::{
  gen::SchemaGenerator,
  schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SubschemaValidation},
};
use serde::Deserialize;
use tauri_utils::acl::{capability::Capability, plugin::Manifest, Permission, PermissionSet};

const CAPABILITY_FILE_EXTENSIONS: &[&str] = &["json", "toml"];
const CAPABILITIES_SCHEMA_FILE_NAME: &str = ".schema.json";

#[derive(Deserialize)]
#[serde(untagged)]
enum CapabilityFile {
  Capability(Capability),
  List { capabilities: Vec<Capability> },
}

fn capabilities_schema(plugin_manifests: &HashMap<String, Manifest>) -> RootSchema {
  let mut schema = SchemaGenerator::default().into_root_schema_for::<Capability>();

  let mut permission_schemas = Vec::new();
  for (plugin, manifest) in plugin_manifests {
    for (set_id, set) in &manifest.permission_sets {
      permission_schemas.push(Schema::Object(SchemaObject {
        metadata: Some(Box::new(Metadata {
          description: Some(format!("{plugin}:{set_id} -> {}", set.description)),
          ..Default::default()
        })),
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(vec![serde_json::Value::String(format!(
          "{plugin}:{set_id}"
        ))]),
        ..Default::default()
      }));
    }

    if let Some(default) = &manifest.default_permission {
      permission_schemas.push(Schema::Object(SchemaObject {
        metadata: Some(Box::new(Metadata {
          description: default
            .description
            .as_ref()
            .map(|d| format!("{plugin}:default -> {d}")),
          ..Default::default()
        })),
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(vec![serde_json::Value::String(format!("{plugin}:default"))]),
        ..Default::default()
      }));
    }

    for (permission_id, permission) in &manifest.permissions {
      permission_schemas.push(Schema::Object(SchemaObject {
        metadata: Some(Box::new(Metadata {
          description: permission
            .description
            .as_ref()
            .map(|d| format!("{plugin}:{permission_id} -> {d}")),
          ..Default::default()
        })),
        instance_type: Some(InstanceType::String.into()),
        enum_values: Some(vec![serde_json::Value::String(format!(
          "{plugin}:{permission_id}"
        ))]),
        ..Default::default()
      }));
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

  schema
}

pub(crate) fn generate_schema(plugin_manifests: &HashMap<String, Manifest>) -> Result<()> {
  let schema = capabilities_schema(plugin_manifests);
  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let out_path = PathBuf::from("capabilities").join(CAPABILITIES_SCHEMA_FILE_NAME);

  let mut schema_file = BufWriter::new(File::create(out_path)?);
  write!(schema_file, "{schema_str}")?;
  Ok(())
}

pub fn parse_capabilities(capabilities_path_pattern: &str) -> Result<HashMap<String, Capability>> {
  let mut capabilities_map = HashMap::new();

  for path in glob::glob(capabilities_path_pattern)?
    .flatten() // filter extension
    .filter(|p| {
      p.extension()
        .and_then(|e| e.to_str())
        .map(|e| CAPABILITY_FILE_EXTENSIONS.contains(&e))
        .unwrap_or_default()
    })
    // filter schema file
    .filter(|p| {
      p.file_name()
        .map(|name| name != CAPABILITIES_SCHEMA_FILE_NAME)
        .unwrap_or(true)
    })
  {
    println!("cargo:rerun-if-changed={}", path.display());

    let capability_file = std::fs::read_to_string(&path)?;
    let ext = path.extension().unwrap().to_string_lossy().to_string();
    let capability: CapabilityFile = match ext.as_str() {
      "toml" => toml::from_str(&capability_file)?,
      "json" => serde_json::from_str(&capability_file)?,
      _ => return Err(anyhow::anyhow!("unknown capability format")),
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

pub(crate) fn get_plugin_manifests() -> Result<HashMap<String, Manifest>> {
  let permission_map =
    tauri_utils::acl::build::read_permissions().context("failed to read plugin permissions")?;

  let mut processed = HashMap::new();
  for (plugin_name, permission_files) in permission_map {
    let mut manifest = Manifest {
      default_permission: None,
      permissions: HashMap::new(),
      permission_sets: HashMap::new(),
    };

    for permission_file in permission_files {
      if let Some(default) = permission_file.default {
        manifest.default_permission.replace(Permission {
          identifier: "default".into(),
          version: default.version,
          description: default.description,
          commands: default.commands,
          scope: default.scope,
        });
      }

      manifest.permissions.extend(
        permission_file
          .permission
          .into_iter()
          .map(|p| (p.identifier.clone(), p))
          .collect::<HashMap<_, _>>(),
      );

      manifest.permission_sets.extend(
        permission_file
          .set
          .into_iter()
          .map(|set| {
            (
              set.identifier.clone(),
              PermissionSet {
                identifier: set.identifier,
                description: set.description,
                permissions: set.permissions,
              },
            )
          })
          .collect::<HashMap<_, _>>(),
      );
    }

    processed.insert(plugin_name, manifest);
  }

  Ok(processed)
}

pub(crate) fn validate_capabilities(
  plugin_manifests: &HashMap<String, Manifest>,
  capabilities: &HashMap<String, Capability>,
) -> Result<()> {
  for capability in capabilities.values() {
    for permission in &capability.permissions {
      if let Some((plugin_name, permission_name)) = permission.get().split_once(':') {
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
            permission.get(),
            available_permissions.join(", ")
          );
        }
      }
    }
  }

  Ok(())
}
