// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{Context, ErrorExt},
  helpers::config::{Config, PatternKind},
};

use itertools::Itertools;
use toml_edit::{Array, DocumentMut, InlineTable, Item, TableLike, Value};

use std::{
  collections::{HashMap, HashSet},
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

  /// Whether the CEF runtime is linked into the application for the given target and enabled features.
  ///
  /// The runtime is selected by depending on the `tauri-runtime-cef` crate, so this is true when:
  /// - `tauri-runtime-cef` is a non-optional dependency in `[dependencies]`, or
  /// - a `[target.'cfg(...)'.dependencies]` table matching the target triple has it as a non-optional dependency, or
  /// - any enabled feature (transitively) enables the optional `tauri-runtime-cef` dependency.
  pub fn is_cef_runtime_used(&self, enabled_features: &[String], target_triple: &str) -> bool {
    let table = self.inner.as_table();

    if is_cef_dependency_required(table) {
      return true;
    }

    if let Some(targets) = table.get("target").and_then(|i| i.as_table()) {
      for (cfg, target_table) in targets.iter() {
        if cfg_matches_target(cfg, target_triple)
          && let Some(target_table) = target_table.as_table()
          && is_cef_dependency_required(target_table)
        {
          return true;
        }
      }
    }

    let features = self.features();
    let mut visited = HashSet::new();
    enabled_features
      .iter()
      .any(|f| feature_enables_cef_runtime(&features, f, &mut visited))
  }
}

const CEF_RUNTIME_CRATE: &str = "tauri-runtime-cef";

/// Whether the given `[dependencies]`-like table lists `tauri-runtime-cef` as a non-optional dependency.
fn is_cef_dependency_required(table: &dyn TableLike) -> bool {
  let Some(dependencies) = table.get("dependencies").and_then(|i| i.as_table_like()) else {
    return false;
  };
  let Some(dependency) = dependencies.get(CEF_RUNTIME_CRATE) else {
    return false;
  };
  // a plain version string is never optional
  let Some(dependency) = dependency.as_table_like() else {
    return true;
  };
  !dependency
    .get("optional")
    .and_then(|i| i.as_value())
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}

/// Whether the feature (transitively) enables the `tauri-runtime-cef` dependency.
fn feature_enables_cef_runtime(
  features: &HashMap<String, Vec<String>>,
  feature: &str,
  visited: &mut HashSet<String>,
) -> bool {
  if feature == CEF_RUNTIME_CRATE || feature == format!("dep:{CEF_RUNTIME_CRATE}") {
    return true;
  }
  if !visited.insert(feature.to_string()) {
    return false;
  }
  features.get(feature).is_some_and(|enabled| {
    enabled
      .iter()
      .any(|f| feature_enables_cef_runtime(features, f, visited))
  })
}

/// Best-effort check of whether a `[target.'cfg(...)']` key applies to the target triple.
///
/// Supports the target triple itself, `cfg(windows)`, `cfg(unix)`, `cfg(target_os = "...")`,
/// `cfg(target_arch = "...")` and `any`/`all`/`not` combinations of those.
fn cfg_matches_target(cfg: &str, target_triple: &str) -> bool {
  let cfg = cfg.trim();
  if cfg == target_triple {
    return true;
  }
  let Some(predicate) = cfg.strip_prefix("cfg(").and_then(|c| c.strip_suffix(')')) else {
    return false;
  };
  cfg_predicate_matches(predicate.trim(), target_triple)
}

fn cfg_predicate_matches(predicate: &str, target_triple: &str) -> bool {
  if let Some(inner) = predicate
    .strip_prefix("any(")
    .and_then(|p| p.strip_suffix(')'))
  {
    return split_cfg_list(inner)
      .iter()
      .any(|p| cfg_predicate_matches(p, target_triple));
  }
  if let Some(inner) = predicate
    .strip_prefix("all(")
    .and_then(|p| p.strip_suffix(')'))
  {
    return split_cfg_list(inner)
      .iter()
      .all(|p| cfg_predicate_matches(p, target_triple));
  }
  if let Some(inner) = predicate
    .strip_prefix("not(")
    .and_then(|p| p.strip_suffix(')'))
  {
    return !cfg_predicate_matches(inner.trim(), target_triple);
  }

  let components: Vec<&str> = target_triple.split('-').collect();
  let arch = components.first().copied().unwrap_or_default();
  let os = if target_triple.contains("darwin") {
    "macos"
  } else if target_triple.contains("windows") {
    "windows"
  } else if target_triple.contains("android") {
    "android"
  } else if target_triple.contains("ios") {
    "ios"
  } else if target_triple.contains("linux") {
    "linux"
  } else {
    components.get(2).copied().unwrap_or_default()
  };

  match predicate {
    "windows" => os == "windows",
    "unix" => os != "windows",
    _ => {
      if let Some((key, value)) = predicate.split_once('=') {
        let value = value.trim().trim_matches('"');
        match key.trim() {
          "target_os" => os == value,
          "target_arch" => arch == value,
          "target_family" => match value {
            "windows" => os == "windows",
            "unix" => os != "windows",
            _ => false,
          },
          _ => false,
        }
      } else {
        false
      }
    }
  }
}

/// Splits a comma separated cfg list, respecting nested parentheses.
fn split_cfg_list(list: &str) -> Vec<String> {
  let mut items = Vec::new();
  let mut depth = 0usize;
  let mut current = String::new();
  for c in list.chars() {
    match c {
      '(' => {
        depth += 1;
        current.push(c);
      }
      ')' => {
        depth = depth.saturating_sub(1);
        current.push(c);
      }
      ',' if depth == 0 => {
        items.push(current.trim().to_string());
        current.clear();
      }
      _ => current.push(c),
    }
  }
  if !current.trim().is_empty() {
    items.push(current.trim().to_string());
  }
  items
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
  let manifest_str = std::fs::read_to_string(manifest_path)
    .fs_context("failed to read Cargo.toml", manifest_path.to_path_buf())?;

  let manifest: DocumentMut = manifest_str
    .parse::<DocumentMut>()
    .context("failed to parse Cargo.toml")?;

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
          if let Some(target_table) = target_value.as_table_mut()
            && let Some(deps) = target_table.get_mut(table)
            && let Some(item) = deps.as_table_mut().and_then(|t| t.get_mut(name))
          {
            matching_deps.push(item);
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
        crate::error::bail!(
          "Unsupported {} dependency format on Cargo.toml",
          dependency_name
        );
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
      if let Value::String(feature) = feat
        && !is_managed_feature(feature.value().as_str())
      {
        features.insert(feature.value().to_string());
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
      if let Some(f) = features_array.get(index).and_then(|f| f.as_str())
        && !features.contains(f)
      {
        features_array.remove(index);
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
        log::info!(
          "`{name}` dependency has workspace inheritance enabled. The features array won't be automatically rewritten. Expected features: [{}]",
          dependency.features.iter().join(", ")
        );
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

pub fn rewrite_manifest(config: &Config, tauri_dir: &Path) -> crate::Result<(Manifest, bool)> {
  let manifest_path = tauri_dir.join("Cargo.toml");
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

  let mut tauri_features = dependencies
    .into_iter()
    .find(|d| d.name == "tauri")
    .unwrap()
    .features;

  // TODO: This is missing workspace root features.
  let items = find_dependency(&mut manifest, "tauri", DependencyKind::Normal);
  for item in items {
    if let Some(features) = item.get("features")
      && let Some(features) = features.as_array()
      && features
        .iter()
        .any(|feature| feature.as_str().unwrap_or_default() == "tray-icon")
    {
      tauri_features.insert("tray-icon".to_string());
    }
  }

  let new_manifest_str = serialize_manifest(&manifest);

  if persist && original_manifest_str != new_manifest_str {
    std::fs::write(&manifest_path, new_manifest_str)
      .fs_context("failed to rewrite Cargo manifest", &manifest_path)?;
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
        if let Some(f) = item_table.and_then(|t| t.get("features")?.as_array().cloned()) {
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

#[cfg(test)]
mod cef_runtime_detection_tests {
  use super::Manifest;

  fn manifest(toml: &str) -> Manifest {
    Manifest {
      inner: toml.parse().expect("invalid manifest"),
      tauri_features: Default::default(),
    }
  }

  const LINUX: &str = "x86_64-unknown-linux-gnu";
  const WINDOWS: &str = "x86_64-pc-windows-msvc";
  const MACOS: &str = "aarch64-apple-darwin";

  #[test]
  fn required_dependency() {
    let m = manifest(
      r#"
[dependencies]
tauri = "2"
tauri-runtime-cef = "0.1"
"#,
    );
    assert!(m.is_cef_runtime_used(&[], LINUX));
  }

  #[test]
  fn optional_dependency_behind_feature() {
    let m = manifest(
      r#"
[dependencies]
tauri = "2"
tauri-runtime-wry = { version = "2", optional = true }
tauri-runtime-cef = { version = "0.1", optional = true }

[features]
default = ["wry"]
wry = ["dep:tauri-runtime-wry"]
cef = ["dep:tauri-runtime-cef"]
chromium = ["cef"]
"#,
    );
    assert!(!m.is_cef_runtime_used(&[], LINUX));
    assert!(!m.is_cef_runtime_used(&m.all_enabled_features(&["default".into()]), LINUX));
    assert!(m.is_cef_runtime_used(&m.all_enabled_features(&["cef".into()]), LINUX));
    // nested feature
    assert!(m.is_cef_runtime_used(&m.all_enabled_features(&["chromium".into()]), LINUX));
    // feature names are also accepted directly
    assert!(m.is_cef_runtime_used(&["cef".into()], LINUX));
  }

  #[test]
  fn target_specific_dependency() {
    let m = manifest(
      r#"
[dependencies]
tauri = "2"

[target.'cfg(windows)'.dependencies]
tauri-runtime-cef = "0.1"

[target.'cfg(any(target_os = "macos", target_os = "ios"))'.dependencies]
tauri-runtime-cef = "0.1"

[target.'cfg(not(any(windows, target_os = "macos")))'.dependencies]
tauri-runtime-wry = "2"
"#,
    );
    assert!(m.is_cef_runtime_used(&[], WINDOWS));
    assert!(m.is_cef_runtime_used(&[], MACOS));
    assert!(!m.is_cef_runtime_used(&[], LINUX));
  }

  #[test]
  fn target_triple_key() {
    let m = manifest(
      r#"
[target.x86_64-unknown-linux-gnu.dependencies]
tauri-runtime-cef = { version = "0.1" }
"#,
    );
    assert!(m.is_cef_runtime_used(&[], LINUX));
    assert!(!m.is_cef_runtime_used(&[], WINDOWS));
  }
}
