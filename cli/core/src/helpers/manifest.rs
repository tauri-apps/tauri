// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{app_paths::tauri_dir, config::ConfigHandle};

use toml_edit::{Array, Document, Value};

use std::{
  fs::File,
  io::{Read, Write},
};

pub fn rewrite_manifest(config: ConfigHandle) -> crate::Result<()> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest_str = String::new();
  let mut manifest_file = File::open(&manifest_path)?;
  manifest_file.read_to_string(&mut manifest_str)?;
  let mut manifest: Document = manifest_str.parse::<Document>()?;
  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

  let entry = dependencies.entry("tauri");
  let tauri = entry.as_value_mut();

  let config_guard = config.lock().unwrap();
  let config = config_guard.as_ref().unwrap();

  if let Some(tauri) = tauri {
    let allowlist_features = config.tauri.features();
    let mut features = Array::default();
    for feature in allowlist_features {
      features.push(feature).unwrap();
    }
    if config.tauri.cli.is_some() {
      features.push("cli").unwrap();
    }

    match tauri {
      Value::InlineTable(tauri_def) => {
        let manifest_features =
          tauri_def.get_or_insert("features", Value::Array(Default::default()));
        *manifest_features = Value::Array(features);
      }
      _ => {
        return Err(anyhow::anyhow!(
          "Unsupported tauri dependency format on Cargo.toml"
        ))
      }
    }

    let mut manifest_file = File::create(&manifest_path)?;
    manifest_file.write_all(
      manifest
        .to_string_in_original_order()
        .replace(r#"" ,features =["#, r#"", features = ["#)
        .as_bytes(),
    )?;
    manifest_file.flush()?;
  }

  Ok(())
}
