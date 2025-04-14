// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  interface::rust::manifest::{read_manifest, serialize_manifest},
  Result,
};

use anyhow::Context;
use tauri_utils_v1::config::Allowlist;
use toml_edit::{DocumentMut, Entry, Item, TableLike, Value};

use std::path::Path;

const CRATE_TYPES: [&str; 3] = ["lib", "staticlib", "cdylib"];

pub fn migrate(tauri_dir: &Path) -> Result<()> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let (mut manifest, _) = read_manifest(&manifest_path)?;
  migrate_manifest(&mut manifest)?;

  std::fs::write(&manifest_path, serialize_manifest(&manifest))
    .context("failed to rewrite Cargo manifest")?;

  Ok(())
}

fn migrate_manifest(manifest: &mut DocumentMut) -> Result<()> {
  let version = dependency_version();

  let remove_features = features_to_remove();
  let rename_features = features_to_rename();
  let rename_message = rename_features
    .iter()
    .map(|(from, to)| format!("{from} to {to}"))
    .collect::<Vec<_>>()
    .join(", ");

  for (dependency, table) in [
    // normal deps
    ("tauri", "dependencies"),
    ("tauri-utils", "dependencies"),
    ("tauri-runtime", "dependencies"),
    ("tauri-codegen", "dependencies"),
    ("tauri-macros", "dependencies"),
    ("tauri-runtime-wry", "dependencies"),
    // normal deps - plugins
    ("tauri-plugin-authenticator", "dependencies"),
    ("tauri-plugin-autostart", "dependencies"),
    ("tauri-plugin-fs-extra", "dependencies"),
    ("tauri-plugin-fs-watch", "dependencies"),
    ("tauri-plugin-localhost", "dependencies"),
    ("tauri-plugin-log", "dependencies"),
    ("tauri-plugin-persisted-scope", "dependencies"),
    ("tauri-plugin-positioner", "dependencies"),
    ("tauri-plugin-single-instance", "dependencies"),
    ("tauri-plugin-sql", "dependencies"),
    ("tauri-plugin-store", "dependencies"),
    ("tauri-plugin-stronghold", "dependencies"),
    ("tauri-plugin-upload", "dependencies"),
    ("tauri-plugin-websocket", "dependencies"),
    ("tauri-plugin-window-state", "dependencies"),
    // dev
    ("tauri", "dev-dependencies"),
    ("tauri-utils", "dev-dependencies"),
    ("tauri-runtime", "dev-dependencies"),
    ("tauri-codegen", "dev-dependencies"),
    ("tauri-macros", "dev-dependencies"),
    ("tauri-runtime-wry", "dev-dependencies"),
    // build
    ("tauri-build", "build-dependencies"),
  ] {
    let items = find_dependency(manifest, dependency, table);

    for item in items {
      // do not rewrite if dependency uses workspace inheritance
      if item
        .get("workspace")
        .and_then(|v| v.as_bool())
        .unwrap_or_default()
      {
        log::warn!("`{dependency}` dependency has workspace inheritance enabled. This migration must be manually migrated to v2 by changing its version to {version}, removing any of the {remove_features:?} and renaming [{}] Cargo features.", rename_message);
      } else {
        migrate_dependency(item, &version, &remove_features, &rename_features);
      }
    }
  }

  if let Some(lib) = manifest
    .as_table_mut()
    .get_mut("lib")
    .and_then(|l| l.as_table_mut())
  {
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
  }

  Ok(())
}

fn find_dependency<'a>(
  manifest: &'a mut DocumentMut,
  name: &'a str,
  table: &'a str,
) -> Vec<&'a mut Item> {
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

fn features_to_rename() -> Vec<(&'static str, &'static str)> {
  vec![
    ("window-data-url", "webview-data-url"),
    ("reqwest-native-tls-vendored", "native-tls-vendored"),
    ("system-tray", "tray-icon"),
    ("icon-ico", "image-ico"),
    ("icon-png", "image-png"),
  ]
}

fn features_to_remove() -> Vec<&'static str> {
  let mut features_to_remove = tauri_utils_v1::config::AllowlistConfig::all_features();
  features_to_remove.extend(&[
    "reqwest-client",
    "http-multipart",
    "process-command-api",
    "shell-open-api",
    "os-api",
    "global-shortcut",
    "clipboard",
    "dialog",
    "notification",
    "fs-extract-api",
    "windows7-compat",
    "updater",
    "cli",
    "linux-protocol-headers",
    "dox",
  ]);

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
      "{}.0.0-{}",
      env!("CARGO_PKG_VERSION_MAJOR"),
      pre.split('.').next().unwrap()
    )
  }
}

fn migrate_dependency(item: &mut Item, version: &str, remove: &[&str], rename: &[(&str, &str)]) {
  if let Some(dep) = item.as_table_mut() {
    migrate_dependency_table(dep, version, remove, rename);
  } else if let Some(Value::InlineTable(table)) = item.as_value_mut() {
    migrate_dependency_table(table, version, remove, rename);
  } else if item.as_str().is_some() {
    *item = Item::Value(version.into());
  }
}

fn migrate_dependency_table<D: TableLike>(
  dep: &mut D,
  version: &str,
  remove: &[&str],
  rename: &[(&str, &str)],
) {
  dep.remove("rev");
  dep.remove("git");
  dep.remove("branch");
  dep.remove("tag");
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
          features_array.remove(index);
        } else if let Some((_from, rename_to)) = rename.iter().find(|(from, _to)| *from == f) {
          features_array.remove(index);
          add_features.push(rename_to);
        }
      }
      i -= 1;
    }

    for f in add_features {
      features_array.push(f.to_string());
    }
  }
}

#[cfg(test)]
mod tests {
  use itertools::Itertools;

  fn migrate_deps<F: FnOnce(&[&str]) -> String>(get_toml: F) {
    let keep_features = vec!["isolation", "protocol-asset"];
    let mut features = super::features_to_remove();
    features.extend(keep_features.clone());
    let toml = get_toml(&features);

    let mut manifest = toml
      .parse::<toml_edit::DocumentMut>()
      .expect("invalid toml");
    super::migrate_manifest(&mut manifest).expect("failed to migrate manifest");

    let dependencies = manifest
      .as_table()
      .get("dependencies")
      .expect("missing manifest dependencies")
      .as_table()
      .expect("manifest dependencies isn't a table");

    let tauri = dependencies
      .get("tauri")
      .expect("missing tauri dependency in manifest");

    let tauri_table = if let Some(table) = tauri.as_table() {
      table.clone()
    } else if let Some(toml_edit::Value::InlineTable(table)) = tauri.as_value() {
      table.clone().into_table()
    } else if let Some(version) = tauri.as_str() {
      // convert the value to a table for the assert logic below
      let mut table = toml_edit::Table::new();
      table.insert(
        "version",
        toml_edit::Item::Value(version.to_string().into()),
      );
      table.insert(
        "features",
        toml_edit::Item::Value(toml_edit::Value::Array(Default::default())),
      );
      table
    } else {
      panic!("unexpected tauri dependency format");
    };

    // assert version matches
    let version = tauri_table
      .get("version")
      .expect("missing version")
      .as_str()
      .expect("version must be a string");
    assert_eq!(version, super::dependency_version());

    // assert features matches
    let features = tauri_table
      .get("features")
      .expect("missing features")
      .as_array()
      .expect("features must be an array")
      .clone();

    if toml.contains("reqwest-native-tls-vendored") {
      assert!(
        features
          .iter()
          .any(|f| f.as_str().expect("feature must be a string") == "native-tls-vendored"),
        "reqwest-native-tls-vendored was not replaced with native-tls-vendored"
      );
    }

    if toml.contains("system-tray") {
      assert!(
        features
          .iter()
          .any(|f| f.as_str().expect("feature must be a string") == "tray-icon"),
        "system-tray was not replaced with tray-icon"
      );
    }

    for feature in features.iter() {
      let feature = feature.as_str().expect("feature must be a string");
      assert!(
        keep_features.contains(&feature)
          || feature == "native-tls-vendored"
          || feature == "tray-icon",
        "feature {feature} should have been removed"
      );
    }
  }

  #[test]
  fn migrate_table() {
    migrate_deps(|features| {
      format!(
        r#"
    [dependencies]
    tauri = {{ version = "1.0.0", features = [{}] }}
"#,
        features.iter().map(|f| format!("{:?}", f)).join(", ")
      )
    });
  }

  #[test]
  fn migrate_inline_table() {
    migrate_deps(|features| {
      format!(
        r#"
    [dependencies.tauri]
    version = "1.0.0"
    features = [{}]
"#,
        features.iter().map(|f| format!("{:?}", f)).join(", ")
      )
    });
  }

  #[test]
  fn migrate_str() {
    migrate_deps(|_features| {
      r#"
    [dependencies]
    tauri = "1.0.0"
"#
      .into()
    })
  }

  #[test]
  fn migrate_add_crate_types() {
    let toml = r#"
    [lib]
    crate-type = ["something"]"#;

    let mut manifest = toml
      .parse::<toml_edit::DocumentMut>()
      .expect("invalid toml");
    super::migrate_manifest(&mut manifest).expect("failed to migrate manifest");

    if let Some(crate_types) = manifest
      .as_table()
      .get("lib")
      .and_then(|l| l.get("crate-type"))
      .and_then(|c| c.as_array())
    {
      let mut not_added_crate_types = super::CRATE_TYPES.to_vec();
      for t in crate_types {
        let t = t.as_str().expect("crate-type must be a string");
        if let Some(i) = not_added_crate_types.iter().position(|ty| ty == &t) {
          not_added_crate_types.remove(i);
        }
      }
      assert!(
        not_added_crate_types.is_empty(),
        "missing crate-type: {not_added_crate_types:?}"
      );
    }
  }
}
