use crate::Result;

use anyhow::Context;
use itertools::Itertools;
use tauri_utils_v1::config::Allowlist;
use toml_edit::{Item, Table, Value};

use std::{collections::HashSet, fs::File, io::Write, path::Path};

pub fn migrate(tauri_dir: &Path) -> Result<()> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let mut manifest = crate::interface::rust::manifest::read_manifest(&manifest_path)?;

  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .or_insert(Item::Table(Table::new()))
    .as_table_mut()
    .expect("manifest dependencies isn't a table");

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

  remove_dependencies(dependencies, "tauri", &features_to_remove);

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

fn remove_dependencies(dependencies: &mut Table, name: &str, remove: &[&str]) {
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
  } else if let Some(Value::InlineTable(table)) = item.as_value_mut() {
    let manifest_features = table.get_or_insert("features", Value::Array(Default::default()));
    let mut features = HashSet::new();
    if let Value::Array(f) = &manifest_features {
      for feat in f.iter() {
        if let Value::String(feature) = feat {
          if !remove.contains(&feature.value().as_str()) {
            features.insert(feature.value().to_string());
          }
        }
      }
    }
    *manifest_features = Value::Array(crate::interface::rust::manifest::toml_array(&features));
  }
}
