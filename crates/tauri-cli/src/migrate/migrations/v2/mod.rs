// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Migrates a Tauri v2 app to v3.

use crate::{
  Result, VersionMetadata,
  error::Context,
  helpers::{app_paths::Dirs, npm::PackageManager},
};

use std::path::Path;

pub(crate) mod config;
mod manifest;
mod source;

const RUNTIME_CRATE: &str = "tauri-runtime-wry";

/// Official plugins, published as `tauri-plugin-<name>` and `@tauri-apps/plugin-<name>`.
const OFFICIAL_PLUGINS: &[&str] = &[
  "autostart",
  "barcode-scanner",
  "biometric",
  "cli",
  "clipboard-manager",
  "deep-link",
  "dialog",
  "fs",
  "geolocation",
  "global-shortcut",
  "haptics",
  "http",
  "localhost",
  "log",
  "nfc",
  "notification",
  "opener",
  "os",
  "persisted-scope",
  "positioner",
  "process",
  "shell",
  "single-instance",
  "sql",
  "store",
  "stronghold",
  "updater",
  "upload",
  "websocket",
  "window-state",
];

pub fn run(dirs: &Dirs) -> Result<()> {
  migrate_tauri_dir(dirs.tauri)?;
  migrate_npm_dependencies(dirs.frontend)?;
  Ok(())
}

/// Migrates the Cargo manifest, the configuration and the Rust sources of the app.
fn migrate_tauri_dir(tauri_dir: &Path) -> Result<()> {
  let metadata = version_metadata()?;

  let manifest =
    manifest::migrate(tauri_dir, &metadata).context("Could not migrate Cargo manifest")?;
  config::migrate(tauri_dir).context("Could not migrate config")?;
  let source =
    source::migrate(tauri_dir, manifest.uses_wry).context("Could not migrate Rust sources")?;

  if manifest.uses_wry && !source.runtime_selected {
    log::warn!(
      "Could not find `tauri::Builder::default()`, select the runtime with `.runtime(tauri_runtime_wry::Wry::default())` on your `tauri::Builder`"
    );
  }

  Ok(())
}

fn migrate_npm_dependencies(frontend_dir: &Path) -> Result<()> {
  let pm = PackageManager::from_project(frontend_dir);
  let npm_version = format!("^{}.0", generic_dependency_version());

  let packages = ["@tauri-apps/cli".to_string(), "@tauri-apps/api".to_string()]
    .into_iter()
    .chain(
      OFFICIAL_PLUGINS
        .iter()
        .map(|plugin| format!("@tauri-apps/plugin-{plugin}")),
    );

  let mut install_deps = Vec::new();
  for pkg in packages {
    let version = pm
      .current_package_version(&pkg, frontend_dir)
      .unwrap_or_default()
      .unwrap_or_default();
    if version.starts_with('2') {
      install_deps.push(format!("{pkg}@{npm_version}"));
    }
  }

  if !install_deps.is_empty() {
    pm.install(&install_deps, frontend_dir)?;
  }

  Ok(())
}

fn version_metadata() -> Result<VersionMetadata> {
  serde_json::from_str(include_str!("../../../../metadata-v2.json"))
    .context("failed to parse version metadata")
}

/// The version requirement matching any release of this major (and pre-release channel), e.g. `3.0.0-alpha`.
fn generic_dependency_version() -> String {
  let pre = env!("CARGO_PKG_VERSION_PRE");
  if pre.is_empty() {
    format!("{}.0.0", env!("CARGO_PKG_VERSION_MAJOR"))
  } else {
    format!(
      "{}.0.0-{}",
      env!("CARGO_PKG_VERSION_MAJOR"),
      pre.split('.').next().unwrap()
    )
  }
}

#[cfg(test)]
mod tests {
  use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
  };

  fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/migrate/migrations/v2/fixtures")
  }

  fn copy_dir(from: &Path, to: &Path) {
    for entry in walkdir::WalkDir::new(from) {
      let entry = entry.unwrap();
      let target = to.join(entry.path().strip_prefix(from).unwrap());
      if entry.file_type().is_dir() {
        std::fs::create_dir_all(&target).unwrap();
      } else {
        std::fs::copy(entry.path(), &target).unwrap();
      }
    }
  }

  fn read_files(dir: &Path) -> BTreeMap<PathBuf, String> {
    walkdir::WalkDir::new(dir)
      .into_iter()
      .map(|e| e.unwrap())
      .filter(|e| e.file_type().is_file())
      .map(|e| {
        (
          e.path().strip_prefix(dir).unwrap().to_path_buf(),
          std::fs::read_to_string(e.path()).unwrap(),
        )
      })
      .collect()
  }

  /// Migrates a copy of `fixtures/<name>` and snapshots each of its files.
  fn migrate_fixture(name: &str) {
    let temp_dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture_dir().join(name), temp_dir.path());

    super::migrate_tauri_dir(temp_dir.path()).expect("failed to migrate");
    let migrated = read_files(temp_dir.path());
    for (file, contents) in &migrated {
      let snapshot_name = format!("{name}__{}", file.display()).replace(['/', '\\'], "__");
      insta::assert_snapshot!(snapshot_name, contents);
    }

    // migrating again is a no-op
    super::migrate_tauri_dir(temp_dir.path()).expect("failed to migrate again");
    pretty_assertions::assert_eq!(read_files(temp_dir.path()), migrated);
  }

  #[test]
  fn migrate_app() {
    migrate_fixture("app");
  }

  #[test]
  fn migrate_app_without_default_features() {
    migrate_fixture("no-default-features");
  }

  #[test]
  fn migrate_app_without_wry() {
    migrate_fixture("no-wry");
  }
}
