// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  app_paths::tauri_dir,
  config::{Config, PatternKind},
};

use anyhow::Context;
use itertools::Itertools;
use toml_edit::{Array, DocumentMut, InlineTable, Item, TableLike, Value};

use std::{
  collections::{HashMap, HashSet},
  fs::File,
  io::{Read, Write},
  path::Path,
};

#[derive(Default)]
pub struct Manifest {
  pub inner: DocumentMut,
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

pub fn read_manifest(manifest_path: &Path) -> crate::Result<(DocumentMut, String)> {
  let mut manifest_str = String::new();

  let mut manifest_file = File::open(manifest_path)
    .with_context(|| format!("failed to open `{manifest_path:?}` file"))?;
  manifest_file.read_to_string(&mut manifest_str)?;

  let manifest: DocumentMut = manifest_str
    .parse::<DocumentMut>()
    .with_context(|| "failed to parse Cargo.toml")?;

  Ok((manifest, manifest_str))
}

pub fn serialize_manifest(manifest: &DocumentMut) -> String {
  manifest
    .to_string()
    // apply some formatting fixes
    .replace(r#"" ,features =["#, r#"", features = ["#)
    .replace(r#"" , features"#, r#"", features"#)
    .replace("]}", "] }")
    .replace("={", "= {")
    .replace("=[", "= [")
    .replace(r#"",""#, r#"", ""#)
}

pub fn toml_array(features: &HashSet<String>) -> Array {
  let mut f = Array::default();
  let mut features: Vec<String> = features.iter().map(|f| f.to_string()).collect();
  features.sort();
  for feature in features {
    f.push(feature.as_str());
  }
  f
}

fn find_dependency<'a>(
  manifest: &'a mut DocumentMut,
  name: &'a str,
  kind: DependencyKind,
) -> Vec<&'a mut Item> {
  let table = match kind {
    DependencyKind::Build => "build-dependencies",
    DependencyKind::Normal => "dependencies",
  };

  let m = manifest.as_table_mut();
  for (k, v) in m.iter_mut() {
    if let Some(t) = v.as_table_mut() {
      if k == table {
        if let Some(item) = t.get_mut(name) {
          return vec![item];
        }
      } else if k == "target" {
        let mut matching_deps = Vec::new();
        for (_, target_value) in t.iter_mut() {
          if let Some(target_table) = target_value.as_table_mut() {
            if let Some(deps) = target_table.get_mut(table) {
              if let Some(item) = deps.as_table_mut().and_then(|t| t.get_mut(name)) {
                matching_deps.push(item);
              }
            }
          }
        }
        return matching_deps;
      }
    }
  }

  Vec::new()
}

fn write_features<F: Fn(&str) -> bool>(
  dependency_name: &str,
  item: &mut Item,
  is_managed_feature: F,
  features: &mut HashSet<String>,
) -> crate::Result<bool> {
  if let Some(dep) = item.as_table_mut() {
    inject_features_table(dep, is_managed_feature, features);
    Ok(true)
  } else if let Some(dep) = item.as_value_mut() {
    match dep {
      Value::InlineTable(table) => {
        inject_features_table(table, is_managed_feature, features);
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
  kind: DependencyKind,
  all_cli_managed_features: Vec<&'static str>,
  features: HashSet<String>,
}

fn inject_features_table<D: TableLike, F: Fn(&str) -> bool>(
  dep: &mut D,
  is_managed_feature: F,
  features: &mut HashSet<String>,
) {
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
    let mut i = features_array.len();
    while i != 0 {
      let index = i - 1;
      if let Some(f) = features_array.get(index).and_then(|f| f.as_str()) {
        if !features.contains(f) {
          features_array.remove(index);
        }
      }
      i -= 1;
    }
  } else {
    *manifest_features = Item::Value(Value::Array(toml_array(features)));
  }
}

fn inject_features(
  manifest: &mut DocumentMut,
  dependencies: &mut Vec<DependencyAllowlist>,
) -> crate::Result<bool> {
  let mut persist = false;
  for dependency in dependencies {
    let name = dependency.name.clone();
    let items = find_dependency(manifest, &dependency.name, dependency.kind);

    for item in items {
      // do not rewrite if dependency uses workspace inheritance
      if item
        .get("workspace")
        .and_then(|v| v.as_bool())
        .unwrap_or_default()
      {
        log::info!("`{name}` dependency has workspace inheritance enabled. The features array won't be automatically rewritten. Expected features: [{}]", dependency.features.iter().join(", "));
      } else {
        let all_cli_managed_features = dependency.all_cli_managed_features.clone();
        let is_managed_feature: Box<dyn Fn(&str) -> bool> =
          Box::new(move |feature| all_cli_managed_features.contains(&feature));

        let should_write =
          write_features(&name, item, is_managed_feature, &mut dependency.features)?;

        if !persist {
          persist = should_write;
        }
      }
    }
  }

  Ok(persist)
}

pub fn rewrite_manifest(config: &Config) -> crate::Result<(Manifest, bool)> {
  let manifest_path = tauri_dir().join("Cargo.toml");
  let (mut manifest, original_manifest_str) = read_manifest(&manifest_path)?;

  let mut dependencies = Vec::new();

  // tauri-build
  let mut tauri_build_features = HashSet::new();
  if let PatternKind::Isolation { .. } = config.app.security.pattern {
    tauri_build_features.insert("isolation".to_string());
  }
  dependencies.push(DependencyAllowlist {
    name: "tauri-build".into(),
    kind: DependencyKind::Build,
    all_cli_managed_features: vec!["isolation"],
    features: tauri_build_features,
  });

  // tauri
  let tauri_features = HashSet::from_iter(config.app.features().into_iter().map(|f| f.to_string()));
  dependencies.push(DependencyAllowlist {
    name: "tauri".into(),
    kind: DependencyKind::Normal,
    all_cli_managed_features: crate::helpers::config::AppConfig::all_features()
      .into_iter()
      .filter(|f| f != &"tray-icon")
      .collect(),
    features: tauri_features,
  });

  let persist = inject_features(&mut manifest, &mut dependencies)?;

  let tauri_features = dependencies
    .into_iter()
    .find(|d| d.name == "tauri")
    .unwrap()
    .features;

  let new_manifest_str = serialize_manifest(&manifest);

  if persist && original_manifest_str != new_manifest_str {
    let mut manifest_file =
      File::create(&manifest_path).with_context(|| "failed to open Cargo.toml for rewrite")?;
    manifest_file.write_all(new_manifest_str.as_bytes())?;
    manifest_file.flush()?;
    Ok((
      Manifest {
        inner: manifest,
        tauri_features,
      },
      true,
    ))
  } else {
    Ok((
      Manifest {
        inner: manifest,
        tauri_features,
      },
      false,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::{DependencyAllowlist, DependencyKind};
  use std::collections::{HashMap, HashSet};

  fn inject_features(toml: &str, mut dependencies: Vec<DependencyAllowlist>) {
    let mut manifest = toml
      .parse::<toml_edit::DocumentMut>()
      .expect("invalid toml");

    let mut expected = HashMap::new();
    for dep in &dependencies {
      let mut features = dep.features.clone();
      for item in super::find_dependency(&mut manifest, &dep.name, dep.kind) {
        let item_table = if let Some(table) = item.as_table() {
          Some(table.clone())
        } else if let Some(toml_edit::Value::InlineTable(table)) = item.as_value() {
          Some(table.clone().into_table())
        } else {
          None
        };
        if let Some(f) = item_table
          .and_then(|t| t.get("features").cloned())
          .and_then(|f| f.as_array().cloned())
        {
          for feature in f.iter() {
            let feature = feature.as_str().expect("feature is not a string");
            if !dep.all_cli_managed_features.contains(&feature) {
              features.insert(feature.into());
            }
          }
        }
      }
      expected.insert(dep.name.clone(), features);
    }

    super::inject_features(&mut manifest, &mut dependencies).expect("failed to migrate manifest");

    for dep in dependencies {
      let expected_features = expected.get(&dep.name).unwrap();
      for item in super::find_dependency(&mut manifest, &dep.name, dep.kind) {
        let item_table = if let Some(table) = item.as_table() {
          table.clone()
        } else if let Some(toml_edit::Value::InlineTable(table)) = item.as_value() {
          table.clone().into_table()
        } else {
          panic!("unexpected TOML item kind for {}", dep.name);
        };

        let features_array = item_table
          .get("features")
          .expect("missing features")
          .as_array()
          .expect("features must be an array")
          .clone();

        let mut features = Vec::new();
        for feature in features_array.iter() {
          let feature = feature.as_str().expect("feature must be a string");
          features.push(feature);
        }
        for expected in expected_features {
          assert!(
            features.contains(&expected.as_str()),
            "feature {expected} should have been injected"
          );
        }
      }
    }
  }

  fn tauri_dependency(features: HashSet<String>) -> DependencyAllowlist {
    DependencyAllowlist {
      name: "tauri".into(),
      kind: DependencyKind::Normal,
      all_cli_managed_features: vec!["isolation"],
      features,
    }
  }

  fn tauri_build_dependency(features: HashSet<String>) -> DependencyAllowlist {
    DependencyAllowlist {
      name: "tauri-build".into(),
      kind: DependencyKind::Build,
      all_cli_managed_features: crate::helpers::config::AppConfig::all_features(),
      features,
    }
  }

  #[test]
  fn inject_features_table() {
    inject_features(
      r#"
    [dependencies]
    tauri = { version = "1", features = ["dummy"] }

    [build-dependencies]
    tauri-build = { version = "1" }
"#,
      vec![
        tauri_dependency(HashSet::from_iter(
          crate::helpers::config::AppConfig::all_features()
            .iter()
            .map(|f| f.to_string()),
        )),
        tauri_build_dependency(HashSet::from_iter(vec!["isolation".into()])),
      ],
    );
  }

  #[test]
  fn inject_features_target() {
    inject_features(
      r#"
    [target."cfg(windows)".dependencies]
    tauri = { version = "1", features = ["dummy"] }

    [target."cfg(target_os = \"macos\")".build-dependencies]
    tauri-build = { version = "1" }

    [target."cfg(target_os = \"linux\")".dependencies]
    tauri = { version = "1", features = ["isolation"] }

    [target."cfg(windows)".build-dependencies]
    tauri-build = { version = "1" }
"#,
      vec![
        tauri_dependency(Default::default()),
        tauri_build_dependency(HashSet::from_iter(vec!["isolation".into()])),
      ],
    );
  }

  #[test]
  fn inject_features_inline_table() {
    inject_features(
      r#"
    [dependencies.tauri]
    version = "1"
    features = ["test"]

    [build-dependencies.tauri-build]
    version = "1"
    features = ["config-toml", "codegen", "isolation"]
"#,
      vec![
        tauri_dependency(HashSet::from_iter(vec![
          "isolation".into(),
          "native-tls-vendored".into(),
        ])),
        tauri_build_dependency(HashSet::from_iter(vec!["isolation".into()])),
      ],
    );
  }

  #[test]
  fn inject_features_string() {
    inject_features(
      r#"
    [dependencies]
    tauri = "1"

    [build-dependencies]
    tauri-build = "1"
"#,
      vec![
        tauri_dependency(HashSet::from_iter(vec![
          "isolation".into(),
          "native-tls-vendored".into(),
        ])),
        tauri_build_dependency(HashSet::from_iter(vec!["isolation".into()])),
      ],
    );
  }
}
