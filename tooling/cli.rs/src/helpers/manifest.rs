// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  app_paths::tauri_dir,
  config::{all_allowlist_features, ConfigHandle},
};

use anyhow::Context;
use toml_edit::{Array, Document, InlineTable, Item, Value};

use std::{
  collections::HashSet,
  fs::File,
  io::{Read, Write},
  path::Path,
};

#[derive(Default)]
pub struct Manifest {
  pub features: HashSet<String>,
}

fn read_manifest(manifest_path: &Path) -> crate::Result<Document> {
  let mut manifest_str = String::new();

  let mut manifest_file = File::open(manifest_path)
    .with_context(|| format!("failed to open `{:?}` file", manifest_path))?;
  manifest_file.read_to_string(&mut manifest_str)?;

  let manifest: Document = manifest_str
    .parse::<Document>()
    .with_context(|| "failed to parse Cargo.toml")?;

  Ok(manifest)
}

fn toml_array(features: &HashSet<String>) -> Array {
  let mut f = Array::default();
  let mut features: Vec<String> = features.iter().map(|f| f.to_string()).collect();
  features.sort();
  for feature in features {
    f.push(feature.as_str()).unwrap();
  }
  f
}

pub fn rewrite_manifest(config: ConfigHandle) -> crate::Result<Manifest> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest = read_manifest(&manifest_path)?;
  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

  let tauri_entry = dependencies.entry("tauri");

  let config_guard = config.lock().unwrap();
  let config = config_guard.as_ref().unwrap();

  let allowlist_features = config.tauri.features();
  let mut features = HashSet::new();
  for feature in allowlist_features {
    features.insert(feature.to_string());
  }
  if config.tauri.cli.is_some() {
    features.insert("cli".to_string());
  }
  if config.tauri.updater.active {
    features.insert("updater".to_string());
  }
  if config.tauri.system_tray.is_some() {
    features.insert("system-tray".to_string());
  }

  let mut cli_managed_features = all_allowlist_features();
  cli_managed_features.extend(vec!["cli", "updater", "system-tray"]);

  if let Some(tauri) = tauri_entry.as_table_mut() {
    let manifest_features = tauri.entry("features");
    if let Item::Value(Value::Array(f)) = &manifest_features {
      for feat in f.iter() {
        if let Value::String(feature) = feat {
          if !cli_managed_features.contains(&feature.value().as_str()) {
            features.insert(feature.value().to_string());
          }
        }
      }
    }
    *manifest_features = Item::Value(Value::Array(toml_array(&features)));
  } else if let Some(tauri) = tauri_entry.as_value_mut() {
    match tauri {
      Value::InlineTable(table) => {
        let manifest_features = table.get_or_insert("features", Value::Array(Default::default()));
        if let Value::Array(f) = &manifest_features {
          for feat in f.iter() {
            if let Value::String(feature) = feat {
              if !cli_managed_features.contains(&feature.value().as_str()) {
                features.insert(feature.value().to_string());
              }
            }
          }
        }
        *manifest_features = Value::Array(toml_array(&features));
      }
      Value::String(version) => {
        let mut def = InlineTable::default();
        def.get_or_insert(
          "version",
          version.to_string().replace("\"", "").replace(" ", ""),
        );
        def.get_or_insert("features", Value::Array(toml_array(&features)));
        *tauri = Value::InlineTable(def);
      }
      _ => {
        return Err(anyhow::anyhow!(
          "Unsupported tauri dependency format on Cargo.toml"
        ))
      }
    }
  } else {
    return Ok(Manifest { features });
  }

  let mut manifest_file =
    File::create(&manifest_path).with_context(|| "failed to open Cargo.toml for rewrite")?;
  manifest_file.write_all(
    manifest
      .to_string_in_original_order()
      // apply some formatting fixes
      .replace(r#"" ,features =["#, r#"", features = ["#)
      .replace("]}", "] }")
      .replace("={", "= {")
      .replace("=[", "= [")
      .as_bytes(),
  )?;
  manifest_file.flush()?;

  Ok(Manifest { features })
}

pub fn get_workspace_members() -> crate::Result<Vec<String>> {
  let mut manifest = read_manifest(&tauri_dir().join("Cargo.toml"))?;
  let workspace = manifest.as_table_mut().entry("workspace").as_table_mut();

  match workspace {
    Some(workspace) => {
      let members = workspace
        .entry("members")
        .as_array()
        .expect("workspace members aren't an array");
      Ok(
        members
          .iter()
          .map(|v| v.as_str().unwrap().to_string())
          .collect(),
      )
    }
    None => Ok(vec![]),
  }
}
