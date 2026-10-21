// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  Result, VersionMetadata,
  error::ErrorExt,
  interface::rust::manifest::{read_manifest, serialize_manifest},
};

use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, TableLike, Value};

use std::path::Path;

use super::{OFFICIAL_PLUGINS, RUNTIME_CRATE, generic_dependency_version};

/// `tauri` features that no longer exist.
const REMOVED_FEATURES: &[&str] = &["wry", "macos-private-api", "objc-exception"];
/// `tauri` features that moved to the wry runtime crate.
const WRY_FEATURES: &[&str] = &["x11", "dbus", "macos-proxy"];
/// `tauri` features that must now be enabled on the runtime crate, which also enables them on `tauri`.
const RUNTIME_FEATURES: &[&str] = &["devtools", "unstable"];
/// `tauri` features that no longer forward to the runtime crate, so they must be enabled on both.
const SHARED_FEATURES: &[&str] = &["tracing"];

/// The result of the Cargo manifest migration.
#[derive(Debug, Default)]
pub struct MigratedManifest {
  /// Whether the app uses the wry runtime, in which case it must be selected with `tauri::Builder::runtime`.
  pub uses_wry: bool,
}

pub fn migrate(tauri_dir: &Path, metadata: &VersionMetadata) -> Result<MigratedManifest> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let (mut manifest, _) = read_manifest(&manifest_path)?;
  let migrated = migrate_manifest(&mut manifest, metadata);

  std::fs::write(&manifest_path, serialize_manifest(&manifest))
    .fs_context("failed to rewrite Cargo manifest", &manifest_path)?;

  Ok(migrated)
}

pub fn migrate_manifest(
  manifest: &mut DocumentMut,
  metadata: &VersionMetadata,
) -> MigratedManifest {
  let mut migrated = MigratedManifest::default();
  let generic_version = generic_dependency_version();

  for table in dependency_tables(manifest, "dependencies") {
    if let Some(runtime_dependency) = migrate_tauri_dependency(table, metadata) {
      migrated.uses_wry = true;
      add_runtime_dependency(table, runtime_dependency, &metadata.tauri_runtime_wry);
    }
    migrate_version(table, RUNTIME_CRATE, &metadata.tauri_runtime_wry);
    migrate_version(table, "tauri-plugin", &metadata.tauri_plugin);
    for dependency in [
      "tauri-utils",
      "tauri-runtime",
      "tauri-codegen",
      "tauri-macros",
    ] {
      migrate_version(table, dependency, &generic_version);
    }
    for plugin in OFFICIAL_PLUGINS {
      migrate_version(table, &format!("tauri-plugin-{plugin}"), &generic_version);
    }
  }

  for table in dependency_tables(manifest, "dev-dependencies") {
    if let Some(dependency) = table.get_mut("tauri") {
      migrate_dependency_item(dependency, "tauri", &metadata.tauri, |features| {
        remove_features(features, |f| {
          REMOVED_FEATURES.contains(&f) || WRY_FEATURES.contains(&f)
        });
      });
    }
    migrate_version(table, RUNTIME_CRATE, &metadata.tauri_runtime_wry);
  }

  for table in dependency_tables(manifest, "build-dependencies") {
    migrate_version(table, "tauri-build", &metadata.tauri_build);
  }

  if let Some(features) = manifest
    .get_mut("features")
    .and_then(|f| f.as_table_like_mut())
  {
    migrate_features_table(features, migrated.uses_wry);
  }

  migrated
}

/// What the runtime crate dependency needs, derived from the v2 `tauri` dependency.
#[derive(Debug, Default)]
struct RuntimeDependency {
  default_features: bool,
  features: Vec<String>,
}

/// Migrates the `tauri` dependency of the given table, returning the wry runtime dependency to add
/// if the app used it.
fn migrate_tauri_dependency(
  table: &mut Table,
  metadata: &VersionMetadata,
) -> Option<RuntimeDependency> {
  let dependency = table.get_mut("tauri")?;

  // a string dependency (`tauri = "2"`) enables the default features, including `wry`
  let default_features = dependency
    .as_table_like()
    .and_then(|t| {
      t.get("default-features")
        .or_else(|| t.get("default_features"))
    })
    .and_then(|v| v.as_bool())
    .unwrap_or(true);
  let features = dependency
    .as_table_like()
    .and_then(|t| t.get("features"))
    .and_then(|f| f.as_array())
    .map(|f| {
      f.iter()
        .filter_map(|f| f.as_str().map(ToString::to_string))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  let uses_wry = default_features || features.iter().any(|f| f == "wry");

  migrate_dependency_item(dependency, "tauri", &metadata.tauri, |features| {
    remove_features(features, |f| {
      REMOVED_FEATURES.contains(&f)
        || WRY_FEATURES.contains(&f)
        || (uses_wry && RUNTIME_FEATURES.contains(&f))
    });
  });

  uses_wry.then(|| RuntimeDependency {
    default_features,
    features: features
      .into_iter()
      .filter(|f| {
        WRY_FEATURES.contains(&f.as_str())
          || RUNTIME_FEATURES.contains(&f.as_str())
          || SHARED_FEATURES.contains(&f.as_str())
      })
      .collect(),
  })
}

fn add_runtime_dependency(table: &mut Table, runtime: RuntimeDependency, version: &str) {
  if let Some(existing) = table.get_mut(RUNTIME_CRATE) {
    // the app already depends on the runtime crate (e.g. to use its APIs), keep its settings
    if !runtime.features.is_empty()
      && let Some(dependency) = existing.as_table_like_mut()
    {
      let features = dependency
        .entry("features")
        .or_insert(Item::Value(Value::Array(Array::new())));
      if let Some(features) = features.as_array_mut() {
        for feature in runtime.features {
          if !features.iter().any(|f| f.as_str() == Some(&feature)) {
            features.push(feature);
          }
        }
      }
    } else if !runtime.features.is_empty() && existing.is_str() {
      let mut dependency = InlineTable::new();
      dependency.insert("version", existing.as_value().unwrap().clone());
      dependency.insert("features", Value::Array(runtime.features.iter().collect()));
      *existing = Item::Value(Value::InlineTable(dependency));
    }
    return;
  }

  let item = if runtime.default_features && runtime.features.is_empty() {
    Item::Value(version.into())
  } else {
    let mut dependency = InlineTable::new();
    dependency.insert("version", version.into());
    if !runtime.default_features {
      dependency.insert("default-features", false.into());
    }
    if !runtime.features.is_empty() {
      dependency.insert("features", Value::Array(runtime.features.iter().collect()));
    }
    Item::Value(Value::InlineTable(dependency))
  };
  table.insert(RUNTIME_CRATE, item);

  // place the runtime crate right after `tauri`
  let keys = table
    .iter()
    .map(|(key, _)| key.to_string())
    .collect::<Vec<_>>();
  if let Some(tauri_index) = keys.iter().position(|key| key == "tauri") {
    let rank = |key: &str| {
      if key == RUNTIME_CRATE {
        (tauri_index, 1)
      } else {
        (keys.iter().position(|k| k == key).unwrap(), 0)
      }
    };
    table.sort_values_by(|a, _, b, _| rank(a.get()).cmp(&rank(b.get())));
  }
}

/// Rewrites the `tauri/*` features enabled by the app's own features.
fn migrate_features_table(features: &mut dyn TableLike, uses_wry: bool) {
  for (_, enabled) in features.iter_mut() {
    let Some(enabled) = enabled.as_array_mut() else {
      continue;
    };

    for value in enabled.iter_mut() {
      let Some(feature) = value.as_str() else {
        continue;
      };
      let (optional, tauri_feature) = if let Some(f) = feature.strip_prefix("tauri/") {
        ("", f)
      } else if let Some(f) = feature.strip_prefix("tauri?/") {
        ("?", f)
      } else {
        continue;
      };
      if uses_wry
        && (WRY_FEATURES.contains(&tauri_feature) || RUNTIME_FEATURES.contains(&tauri_feature))
      {
        let decor = value.decor().clone();
        *value = format!("{RUNTIME_CRATE}{optional}/{tauri_feature}").into();
        *value.decor_mut() = decor;
      }
    }

    remove_features(enabled, |f| {
      let tauri_feature = f
        .strip_prefix("tauri/")
        .or_else(|| f.strip_prefix("tauri?/"));
      tauri_feature.is_some_and(|f| REMOVED_FEATURES.contains(&f) || WRY_FEATURES.contains(&f))
    });
  }
}

/// Returns the dependency tables of the given kind, including the target-specific ones.
fn dependency_tables<'a>(manifest: &'a mut DocumentMut, kind: &str) -> Vec<&'a mut Table> {
  let mut tables = Vec::new();
  for (key, value) in manifest.as_table_mut().iter_mut() {
    let Some(table) = value.as_table_mut() else {
      continue;
    };
    if key == kind {
      tables.push(table);
    } else if key == "target" {
      for (_, target) in table.iter_mut() {
        if let Some(dependencies) = target
          .as_table_mut()
          .and_then(|t| t.get_mut(kind))
          .and_then(|d| d.as_table_mut())
        {
          tables.push(dependencies);
        }
      }
    }
  }
  tables
}

fn migrate_version(table: &mut Table, name: &str, version: &str) {
  if let Some(dependency) = table.get_mut(name) {
    migrate_dependency_item(dependency, name, version, |_| {});
  }
}

/// Updates the version of a dependency and calls `migrate_features` with its features array, if any.
fn migrate_dependency_item(
  item: &mut Item,
  name: &str,
  version: &str,
  migrate_features: impl FnOnce(&mut Array),
) {
  if let Some(dependency) = item.as_table_like_mut() {
    if let Some(features) = dependency
      .get_mut("features")
      .and_then(|f| f.as_array_mut())
    {
      migrate_features(features);
    }

    if dependency
      .get("workspace")
      .and_then(|v| v.as_bool())
      .unwrap_or_default()
    {
      log::warn!(
        "`{name}` dependency has workspace inheritance enabled. Change its version to {version} in the workspace manifest."
      );
    } else if dependency.contains_key("path") {
      log::warn!("`{name}` is a path dependency, make sure it points to a {version} checkout.");
    } else {
      for key in ["git", "branch", "tag", "rev"] {
        dependency.remove(key);
      }
      let version_item = dependency.entry("version").or_insert(Item::None);
      let decor = version_item.as_value().map(|v| v.decor().clone());
      let mut value = Value::from(version);
      if let Some(decor) = decor {
        *value.decor_mut() = decor;
      }
      *version_item = Item::Value(value);
    }
  } else if let Some(value) = item.as_value_mut()
    && value.is_str()
  {
    let decor = value.decor().clone();
    *value = version.into();
    *value.decor_mut() = decor;
  }
}

/// Removes the matching features, keeping the array formatting.
fn remove_features(features: &mut Array, remove: impl Fn(&str) -> bool) {
  let first_prefix = features.get(0).and_then(|f| f.decor().prefix().cloned());
  features.retain(|f| !f.as_str().is_some_and(&remove));
  // the new first element may carry the separator whitespace of a removed one
  if let (Some(first), Some(prefix)) = (features.get_mut(0), first_prefix) {
    first.decor_mut().set_prefix(prefix);
  }
}
