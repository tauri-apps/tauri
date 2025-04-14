// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{frontend_dir, tauri_dir},
    npm::PackageManager,
  },
  interface::rust::manifest::{read_manifest, serialize_manifest},
  Result,
};

use std::{fs::read_to_string, path::Path};

use anyhow::Context;
use toml_edit::{DocumentMut, Item, Table, TableLike, Value};

pub fn run() -> Result<()> {
  let frontend_dir = frontend_dir();
  let tauri_dir = tauri_dir();

  let manifest_path = tauri_dir.join("Cargo.toml");
  let (mut manifest, _) = read_manifest(&manifest_path)?;
  migrate_manifest(&mut manifest)?;

  migrate_permissions(tauri_dir)?;

  migrate_npm_dependencies(frontend_dir)?;

  std::fs::write(&manifest_path, serialize_manifest(&manifest))
    .context("failed to rewrite Cargo manifest")?;

  Ok(())
}

fn migrate_npm_dependencies(frontend_dir: &Path) -> Result<()> {
  let pm = PackageManager::from_project(frontend_dir);

  let mut install_deps = Vec::new();
  for pkg in [
    "@tauri-apps/cli",
    "@tauri-apps/api",
    "@tauri-apps/plugin-authenticator",
    "@tauri-apps/plugin-autostart",
    "@tauri-apps/plugin-barcode-scanner",
    "@tauri-apps/plugin-biometric",
    "@tauri-apps/plugin-cli",
    "@tauri-apps/plugin-clipboard-manager",
    "@tauri-apps/plugin-deep-link",
    "@tauri-apps/plugin-dialog",
    "@tauri-apps/plugin-fs",
    "@tauri-apps/plugin-global-shortcut",
    "@tauri-apps/plugin-http",
    "@tauri-apps/plugin-log",
    "@tauri-apps/plugin-nfc",
    "@tauri-apps/plugin-notification",
    "@tauri-apps/plugin-os",
    "@tauri-apps/plugin-positioner",
    "@tauri-apps/plugin-process",
    "@tauri-apps/plugin-shell",
    "@tauri-apps/plugin-sql",
    "@tauri-apps/plugin-store",
    "@tauri-apps/plugin-stronghold",
    "@tauri-apps/plugin-updater",
    "@tauri-apps/plugin-upload",
    "@tauri-apps/plugin-websocket",
    "@tauri-apps/plugin-window-state",
  ] {
    let version = pm
      .current_package_version(pkg, frontend_dir)
      .unwrap_or_default()
      .unwrap_or_default();
    if version.starts_with('1') {
      install_deps.push(format!("{pkg}@^2.0.0-rc.0"));
    }
  }

  if !install_deps.is_empty() {
    pm.install(&install_deps, frontend_dir)?;
  }

  Ok(())
}

fn migrate_permissions(tauri_dir: &Path) -> Result<()> {
  let core_plugins = [
    "app",
    "event",
    "image",
    "menu",
    "path",
    "resources",
    "tray",
    "webview",
    "window",
  ];

  for entry in walkdir::WalkDir::new(tauri_dir.join("capabilities")) {
    let entry = entry?;
    let path = entry.path();
    if path.extension().is_some_and(|ext| ext == "json") {
      let mut capability = read_to_string(path).context("failed to read capability")?;
      for plugin in core_plugins {
        capability = capability.replace(&format!("\"{plugin}:"), &format!("\"core:{plugin}:"));
      }
      std::fs::write(path, capability).context("failed to rewrite capability")?;
    }
  }
  Ok(())
}

fn migrate_manifest(manifest: &mut DocumentMut) -> Result<()> {
  let version = "2.0.0-rc.0";

  let dependencies = manifest
    .as_table_mut()
    .entry("dependencies")
    .or_insert(Item::Table(Table::new()))
    .as_table_mut()
    .context("manifest dependencies isn't a table")?;

  for dep in [
    "tauri",
    "tauri-plugin-authenticator",
    "tauri-plugin-autostart",
    "tauri-plugin-barcode-scanner",
    "tauri-plugin-biometric",
    "tauri-plugin-cli",
    "tauri-plugin-clipboard-manager",
    "tauri-plugin-deep-link",
    "tauri-plugin-dialog",
    "tauri-plugin-fs",
    "tauri-plugin-global-shortcut",
    "tauri-plugin-http",
    "tauri-plugin-localhost",
    "tauri-plugin-log",
    "tauri-plugin-nfc",
    "tauri-plugin-notification",
    "tauri-plugin-os",
    "tauri-plugin-persisted-scope",
    "tauri-plugin-positioner",
    "tauri-plugin-process",
    "tauri-plugin-shell",
    "tauri-plugin-single-instance",
    "tauri-plugin-sql",
    "tauri-plugin-store",
    "tauri-plugin-stronghold",
    "tauri-plugin-updater",
    "tauri-plugin-upload",
    "tauri-plugin-websocket",
    "tauri-plugin-window-state",
  ] {
    migrate_dependency(dependencies, dep, version);
  }

  let build_dependencies = manifest
    .as_table_mut()
    .entry("build-dependencies")
    .or_insert(Item::Table(Table::new()))
    .as_table_mut()
    .context("manifest build-dependencies isn't a table")?;

  migrate_dependency(build_dependencies, "tauri-build", version);

  Ok(())
}

fn migrate_dependency(dependencies: &mut Table, name: &str, version: &str) {
  let item = dependencies.entry(name).or_insert(Item::None);

  // do not rewrite if dependency uses workspace inheritance
  if item
    .get("workspace")
    .and_then(|v| v.as_bool())
    .unwrap_or_default()
  {
    log::info!("`{name}` dependency has workspace inheritance enabled. The features array won't be automatically rewritten.");
    return;
  }

  if let Some(dep) = item.as_table_mut() {
    migrate_dependency_table(dep, version);
  } else if let Some(Value::InlineTable(table)) = item.as_value_mut() {
    migrate_dependency_table(table, version);
  } else if item.as_str().is_some() {
    *item = Item::Value(version.into());
  }
}

fn migrate_dependency_table<D: TableLike>(dep: &mut D, version: &str) {
  *dep.entry("version").or_insert(Item::None) = Item::Value(version.into());
}
