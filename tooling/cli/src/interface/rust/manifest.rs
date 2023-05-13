// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  app_paths::tauri_dir,
  config::{Config, PatternKind},
};

use anyhow::Context;
use heck::AsKebabCase;
use itertools::Itertools;
use log::info;
use serde_json::Value as JsonValue;
use toml_edit::{Array, Document, InlineTable, Item, Value};

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
      .map(|f| format!("tauri/{f}"))
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

pub fn read_manifest(manifest_path: &Path) -> crate::Result<Document> {
  let mut manifest_str = String::new();

  let mut manifest_file = File::open(manifest_path)
    .with_context(|| format!("failed to open `{manifest_path:?}` file"))?;
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

fn find_dependency<'a>(
  manifest: &'a mut Document,
  name: &'a str,
  kind: DependencyKind,
) -> Option<&'a mut Item> {
  let table = match kind {
    DependencyKind::Build => "build-dependencies",
    DependencyKind::Normal => "dependencies",
  };

  let m = manifest.as_table_mut();
  for (k, v) in m.iter_mut() {
    if let Some(t) = v.as_table_mut() {
      if k == table {
        if let Some(item) = t.get_mut(name) {
          return Some(item);
        }
      } else if k == "target" {
        for (_, target_value) in t.iter_mut() {
          if let Some(target_table) = target_value.as_table_mut() {
            if let Some(deps) = target_table.get_mut(table) {
              if let Some(item) = deps.as_table_mut().and_then(|t| t.get_mut(name)) {
                return Some(item);
              }
            }
          }
        }
      }
    }
  }

  None
}

fn write_features<F: Fn(&str) -> bool>(
  dependency_name: &str,
  item: &mut Item,
  is_managed_feature: F,
  features: &mut HashSet<String>,
) -> crate::Result<bool> {
  if let Some(dep) = item.as_table_mut() {
    let manifest_features = dep.entry("features").or_insert(Item::None);
    if let Item::Value(Value::Array(f)) = &manifest_features {
      for feat in f.iter() {
        if let Value::String(feature) = feat {
          if !is_managed_feature(feature.value().as_str()) {
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
              if !is_managed_feature(feature.value().as_str()) {
                features.insert(feature.value().to_string());
              }
            }
          }
        }
        *manifest_features = Value::Array(toml_array(features));
      }
      Value::String(version) => {
        let mut def = InlineTable::default();
        def.get_or_insert("version", version.to_string().replace(['\"', ' '], ""));
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

#[derive(Debug, Clone, Copy)]
enum DependencyKind {
  Build,
  Normal,
}

#[derive(Debug)]
struct DependencyAllowlist {
  name: String,
  alias: Option<String>,
  kind: DependencyKind,
  all_cli_managed_features: Option<Vec<&'static str>>,
  features: HashSet<String>,
}

pub fn rewrite_manifest(config: &Config) -> crate::Result<Manifest> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let mut manifest = read_manifest(&manifest_path)?;

  let mut dependencies = Vec::new();

  // tauri-build
  let mut tauri_build_features = HashSet::new();
  if let PatternKind::Isolation { .. } = config.tauri.pattern {
    tauri_build_features.insert("isolation".to_string());
  }
  dependencies.push(DependencyAllowlist {
    name: "tauri-build".into(),
    alias: None,
    kind: DependencyKind::Build,
    all_cli_managed_features: Some(vec!["isolation"]),
    features: tauri_build_features,
  });

  // tauri
  let tauri_features =
    HashSet::from_iter(config.tauri.features().into_iter().map(|f| f.to_string()));
  dependencies.push(DependencyAllowlist {
    name: "tauri".into(),
    alias: None,
    kind: DependencyKind::Normal,
    all_cli_managed_features: Some(crate::helpers::config::TauriConfig::all_features()),
    features: tauri_features,
  });

  for (plugin, conf) in &config.plugins.0 {
    let features = if let JsonValue::Object(obj) = conf {
      if let Some(JsonValue::Object(allowlist)) = obj.get("allowlist") {
        let mut features = HashSet::new();
        for (allowed, value) in allowlist {
          if let JsonValue::Bool(true) = value {
            features.insert(format!("allow-{}", AsKebabCase(allowed)));
          }
        }
        features
      } else {
        HashSet::new()
      }
    } else {
      HashSet::new()
    };

    dependencies.push(DependencyAllowlist {
      name: plugin.into(),
      alias: Some(format!("tauri-plugin-{plugin}")),
      kind: DependencyKind::Normal,
      all_cli_managed_features: None,
      features,
    });
  }

  let mut persist = false;
  for dependency in &mut dependencies {
    let mut name = dependency.name.clone();
    let mut item = find_dependency(&mut manifest, &dependency.name, dependency.kind);
    if item.is_none() {
      if let Some(alias) = &dependency.alias {
        item = find_dependency(&mut manifest, alias, dependency.kind);
        if item.is_some() {
          name = alias.clone();
        }
      }
    }

    if let Some(item) = item {
      // do not rewrite if dependency uses workspace inheritance
      if item
        .get("workspace")
        .and_then(|v| v.as_bool())
        .unwrap_or_default()
      {
        info!("`{name}` dependency has workspace inheritance enabled. The features array won't be automatically rewritten. Expected features: [{}]", dependency.features.iter().join(", "));
      } else {
        let is_managed_feature: Box<dyn Fn(&str) -> bool> =
          if let Some(all_features) = &dependency.all_cli_managed_features {
            Box::new(move |feature| all_features.contains(&feature))
          } else {
            Box::new(|f| f.starts_with("allow-"))
          };

        let should_write =
          write_features(&name, item, is_managed_feature, &mut dependency.features)?;

        if !persist {
          persist = should_write;
        }
      }
    }
  }

  let tauri_features = dependencies
    .into_iter()
    .find(|d| d.name == "tauri")
    .unwrap()
    .features;

  if persist {
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
  } else {
    Ok(Manifest {
      inner: manifest,
      tauri_features,
    })
  }
}
