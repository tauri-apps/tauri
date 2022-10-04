// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  app_paths::tauri_dir,
  config::{Config, PatternKind},
};

use anyhow::Context;
use toml_edit::{Array, Document, InlineTable, Item, Table, Value};

use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{Read, Write},
  iter::FromIterator,
  path::Path,
};

#[derive(Default)]
pub struct Manifest {
  pub inner: Document,
  pub tauri_features: HashSet<String>,
}

impl Manifest {
  pub fn features(&self) -> HashMap<String, Vec<String>> {
    let mut f = HashMap::new();

    if let Some(features) = self
      .inner
      .as_table()
      .get("features")
      .and_then(|f| f.as_table())
    {
      for (feature, enabled_features) in features.into_iter() {
        if let Item::Value(Value::Array(enabled_features)) = enabled_features {
          let mut enabled = Vec::new();
          for value in enabled_features {
            if let Value::String(s) = value {
              enabled.push(s.value().clone());
            }
          }
          f.insert(feature.to_string(), enabled);
        }
      }
    }

    f
  }

  pub fn all_enabled_features(&self, enabled_features: &[String]) -> Vec<String> {
    let mut all_enabled_features: Vec<String> = self
      .tauri_features
      .iter()
      .map(|f| format!("tauri/{}", f))
      .collect();

    let manifest_features = self.features();
    for f in enabled_features {
      all_enabled_features.extend(get_enabled_features(&manifest_features, f));
    }

    all_enabled_features
  }
}

fn get_enabled_features(list: &HashMap<String, Vec<String>>, feature: &str) -> Vec<String> {
  let mut f = Vec::new();

  if let Some(enabled_features) = list.get(feature) {
    for enabled in enabled_features {
      if list.contains_key(enabled) {
        f.extend(get_enabled_features(list, enabled));
      } else {
        f.push(enabled.clone());
      }
    }
  }

  f
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
    f.push(feature.as_str());
  }
  f
}

fn write_features(
  dependencies: &mut Table,
  dependency_name: &str,
  all_features: Vec<&str>,
  features: &mut HashSet<String>,
) -> crate::Result<bool> {
  let item = dependencies.entry(dependency_name).or_insert(Item::None);

  if let Some(dep) = item.as_table_mut() {
    let manifest_features = dep.entry("features").or_insert(Item::None);
    if let Item::Value(Value::Array(f)) = &manifest_features {
      for feat in f.iter() {
        if let Value::String(feature) = feat {
          if !all_features.contains(&feature.value().as_str()) {
            features.insert(feature.value().to_string());
          }
        }
      }
    }
    if let Some(features_array) = manifest_features.as_array_mut() {
      // add features that aren't in the manifest
      for feature in features.iter() {
        if !features_array.iter().any(|f| f.as_str() == Some(feature)) {
          features_array.insert(0, feature.as_str());
        }
      }

      // remove features that shouldn't be in the manifest anymore
      let mut i = 0;
      while i < features_array.len() {
        if let Some(f) = features_array.get(i).and_then(|f| f.as_str()) {
          if !features.contains(f) {
            features_array.remove(i);
          }
        }
        i += 1;
      }
    } else {
      *manifest_features = Item::Value(Value::Array(toml_array(features)));
    }
    Ok(true)
  } else if let Some(dep) = item.as_value_mut() {
    match dep {
      Value::InlineTable(table) => {
        let manifest_features = table.get_or_insert("features", Value::Array(Default::default()));
        if let Value::Array(f) = &manifest_features {
          for feat in f.iter() {
            if let Value::String(feature) = feat {
              if !all_features.contains(&feature.value().as_str()) {
                features.insert(feature.value().to_string());
              }
            }
          }
        }
        *manifest_features = Value::Array(toml_array(features));
      }
      Value::String(version) => {
        let mut def = InlineTable::default();
        def.get_or_insert(
          "version",
          version.to_string().replace('\"', "").replace(' ', ""),
        );
        def.get_or_insert("features", Value::Array(toml_array(features)));
        *dep = Value::InlineTable(def);
      }
      _ => {
        return Err(anyhow::anyhow!(
          "Unsupported {} dependency format on Cargo.toml",
          dependency_name
        ))
      }
    }
    Ok(true)
  } else {
    Ok(false)
  }
}

pub fn rewrite_manifest(config: &Config) -> crate::Result<Manifest> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest = read_manifest(&manifest_path)?;

  let mut tauri_build_features = HashSet::new();
  if let PatternKind::Isolation { .. } = config.tauri.pattern {
    tauri_build_features.insert("isolation".to_string());
  }
  let resp = write_features(
    manifest
      .as_table_mut()
      .entry("build-dependencies")
      .or_insert(Item::Table(Table::new()))
      .as_table_mut()
      .expect("manifest build-dependencies isn't a table"),
    "tauri-build",
    vec!["isolation"],
    &mut tauri_build_features,
  )?;

  let mut tauri_features =
    HashSet::from_iter(config.tauri.features().into_iter().map(|f| f.to_string()));
  let cli_managed_tauri_features = crate::helpers::config::TauriConfig::all_features();
  let res = match write_features(
    manifest
      .as_table_mut()
      .entry("dependencies")
      .or_insert(Item::Table(Table::new()))
      .as_table_mut()
      .expect("manifest dependencies isn't a table"),
    "tauri",
    cli_managed_tauri_features,
    &mut tauri_features,
  ) {
    Err(e) => Err(e),
    Ok(t) if !resp => Ok(t),
    _ => Ok(true),
  };

  match res {
    Ok(true) => {
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
      Ok(Manifest {
        inner: manifest,
        tauri_features,
      })
    }
    Ok(false) => Ok(Manifest {
      inner: manifest,
      tauri_features,
    }),
    Err(e) => Err(e),
  }
}
