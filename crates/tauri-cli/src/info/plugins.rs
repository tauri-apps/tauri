// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::{
  helpers::{
    self,
    cargo_manifest::{crate_version, CargoLock, CargoManifest},
    npm::PackageManager,
  },
  interface::rust::get_workspace_dir,
};

use super::{packages_nodejs, packages_rust, SectionItem};

pub fn items(
  frontend_dir: Option<&PathBuf>,
  tauri_dir: Option<&Path>,
  package_manager: PackageManager,
) -> Vec<SectionItem> {
  let mut items = Vec::new();

  if tauri_dir.is_some() || frontend_dir.is_some() {
    if let Some(tauri_dir) = tauri_dir {
      let manifest: Option<CargoManifest> =
        if let Ok(manifest_contents) = fs::read_to_string(tauri_dir.join("Cargo.toml")) {
          toml::from_str(&manifest_contents).ok()
        } else {
          None
        };

      let lock: Option<CargoLock> = get_workspace_dir()
        .ok()
        .and_then(|p| fs::read_to_string(p.join("Cargo.lock")).ok())
        .and_then(|s| toml::from_str(&s).ok());

      for p in helpers::plugins::known_plugins().keys() {
        let dep = format!("tauri-plugin-{p}");
        let crate_version = crate_version(tauri_dir, manifest.as_ref(), lock.as_ref(), &dep);
        if !crate_version.has_version() {
          continue;
        }
        let item = packages_rust::rust_section_item(&dep, crate_version);
        items.push(item);

        let Some(frontend_dir) = frontend_dir else {
          continue;
        };

        let package = format!("@tauri-apps/plugin-{p}");

        let item = packages_nodejs::nodejs_section_item(
          package,
          None,
          frontend_dir.clone(),
          package_manager,
        );
        items.push(item);
      }
    }
  }

  items
}
