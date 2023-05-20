// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{interface::rust::manifest::read_manifest, Result};

use anyhow::Context;
use itertools::Itertools;
use tauri_utils_v1::config::Allowlist;
use toml_edit::{Document, Entry, Item, Table, TableLike, Value};

use std::{fs::File, io::Write, path::Path};

const CRATE_TYPES: &[&str] = &["staticlib", "cdylib", "rlib"];

pub fn migrate(tauri_dir: &Path) -> Result<()> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let mut manifest = read_manifest(&manifest_path)?;
  migrate_manifest(&mut manifest)?;

  let mut manifest_file =
    File::create(&manifest_path).with_context(|| "failed to open Cargo.toml for rewrite")?;
  manifest_file.write_all(
    manifest
      .to_string()
      // apply some formatting fixes
      .replace(r#"" ,features =["#, r#"", features = ["#)
      .replace(r#"" , features"#, r#"", features"#)
      .replace("]}", "] }")
      .replace("={", "= {")
      .replace("=[", "= [")
      .as_bytes(),
  )?;
  manifest_file.flush()?;

  Ok(())
}

fn migrate_manifest(manifest: &mut Document) -> Result<()> {
  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .or_insert(Item::Table(Table::new()))
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

  let version = dependency_version();
  migrate_dependency(dependencies, "tauri", version, &features_to_remove());

  let lib = manifest
    .as_table_mut()
    .entry("lib")
    .or_insert(Item::Table(Table::new()))
    .as_table_mut()
    .expect("manifest lib isn't a table");
  match lib.entry("crate-type") {
    Entry::Occupied(mut e) => {
      if let Item::Value(Value::Array(types)) = e.get_mut() {
        let mut crate_types_to_add = CRATE_TYPES.to_vec();
        for t in types.iter() {
          // type is already in the manifest, skip adding it
          if let Some(i) = crate_types_to_add
            .iter()
            .position(|ty| Some(ty) == t.as_str().as_ref())
          {
            crate_types_to_add.remove(i);
          }
        }
        for t in crate_types_to_add {
          types.push(t);
        }
      }
    }
    Entry::Vacant(e) => {
      let mut arr = toml_edit::Array::new();
      arr.extend(CRATE_TYPES.to_vec());
      e.insert(Item::Value(arr.into()));
    }
  }

  Ok(())
}

fn features_to_remove() -> Vec<&'static str> {
  let mut features_to_remove = tauri_utils_v1::config::AllowlistConfig::all_features();
  features_to_remove.push("reqwest-client");
  features_to_remove.push("reqwest-native-tls-vendored");
  features_to_remove.push("process-command-api");
  features_to_remove.push("shell-open-api");
  features_to_remove.push("windows7-compat");
  features_to_remove.push("updater");

  // this allowlist feature was not removed
  let index = features_to_remove
    .iter()
    .position(|x| x == &"protocol-asset")
    .unwrap();
  features_to_remove.remove(index);

  features_to_remove
}

fn dependency_version() -> String {
  let pre = env!("CARGO_PKG_VERSION_PRE");
  if pre.is_empty() {
    env!("CARGO_PKG_VERSION_MAJOR").to_string()
  } else {
    format!(
      "{}.{}.{}-{}",
      env!("CARGO_PKG_VERSION_MAJOR"),
      env!("CARGO_PKG_VERSION_MINOR"),
      env!("CARGO_PKG_VERSION_PATCH"),
      pre.split('.').next().unwrap()
    )
  }
}

fn migrate_dependency(dependencies: &mut Table, name: &str, version: String, remove: &[&str]) {
  let item = dependencies.entry(name).or_insert(Item::None);

  // do not rewrite if dependency uses workspace inheritance
  if item
    .get("workspace")
    .and_then(|v| v.as_bool())
    .unwrap_or_default()
  {
    log::info!("`{name}` dependency has workspace inheritance enabled. The features array won't be automatically rewritten. Remove features: [{}]", remove.iter().join(", "));
    return;
  }

  if let Some(dep) = item.as_table_mut() {
    migrate_dependency_table(dep, version, remove);
  } else if let Some(Value::InlineTable(table)) = item.as_value_mut() {
    migrate_dependency_table(table, version, remove);
  } else if item.as_str().is_some() {
    *item = Item::Value(version.into());
  }
}

fn migrate_dependency_table<D: TableLike>(dep: &mut D, version: String, remove: &[&str]) {
  *dep.entry("version").or_insert(Item::None) = Item::Value(version.into());
  let manifest_features = dep.entry("features").or_insert(Item::None);
  if let Some(features_array) = manifest_features.as_array_mut() {
    // remove features that shouldn't be in the manifest anymore
    let mut i = features_array.len();
    let mut add_features = Vec::new();
    while i != 0 {
      let index = i - 1;
      if let Some(f) = features_array.get(index).and_then(|f| f.as_str()) {
        if remove.contains(&f) {
          let f = f.to_string();
          features_array.remove(index);
          if f == "reqwest-native-tls-vendored" {
            add_features.push("native-tls-vendored");
          }
        }
      }
      i -= 1;
    }
    for f in add_features {
      features_array.push(f);
    }
  }
}
