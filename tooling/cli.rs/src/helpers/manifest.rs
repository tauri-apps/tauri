// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{app_paths::tauri_dir, config::ConfigHandle};

use anyhow::Context;
use toml_edit::{Array, Document, InlineTable, Item, Value};

use std::{
  fs::File,
  io::{Read, Write},
};

pub fn rewrite_manifest(config: ConfigHandle) -> crate::Result<()> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest_str = String::new();
  let mut manifest_file = File::open(&manifest_path)
    .with_context(|| format!("failed to open `{:?}` file", manifest_path))?;
  manifest_file.read_to_string(&mut manifest_str)?;
  let mut manifest: Document = manifest_str
    .parse::<Document>()
    .with_context(|| "failed to parse Cargo.toml")?;
  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

  let tauri_entry = dependencies.entry("tauri");

  let config_guard = config.lock().unwrap();
  let config = config_guard.as_ref().unwrap();

  let allowlist_features = config.tauri.features();
  let mut features = Array::default();
  for feature in allowlist_features {
    features.push(feature).unwrap();
  }
  if config.tauri.cli.is_some() {
    features.push("cli").unwrap();
  }
  if config.tauri.updater.active {
    features.push("updater").unwrap();
  }

  if let Some(tauri) = tauri_entry.as_table_mut() {
    let manifest_features = tauri.entry("features");
    *manifest_features = Item::Value(Value::Array(features));
  } else if let Some(tauri) = tauri_entry.as_value_mut() {
    match tauri {
      Value::InlineTable(table) => {
        let manifest_features = table.get_or_insert("features", Value::Array(Default::default()));
        *manifest_features = Value::Array(features);
      }
      Value::String(version) => {
        let mut def = InlineTable::default();
        def.get_or_insert(
          "version",
          version.to_string().replace("\"", "").replace(" ", ""),
        );
        def.get_or_insert("features", Value::Array(features));
        *tauri = Value::InlineTable(def);
      }
      _ => {
        return Err(anyhow::anyhow!(
          "Unsupported tauri dependency format on Cargo.toml"
        ))
      }
    }
  } else {
    return Ok(());
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

  Ok(())
}
