// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
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

#[derive(Debug)]
pub struct InstalledPlugin {
  pub crate_name: String,
  pub npm_name: String,
  pub crate_version: semver::Version,
  pub npm_version: semver::Version,
}

#[derive(Debug)]
pub struct InstalledPlugins(Vec<InstalledPlugin>);

impl InstalledPlugins {
  pub fn incompatible(&self) -> Vec<&InstalledPlugin> {
    self
      .0
      .iter()
      .filter(|p| {
        p.crate_version.major != p.npm_version.major || p.crate_version.minor != p.npm_version.minor
      })
      .collect()
  }
}

pub fn installed_plugins(
  frontend_dir: &Path,
  tauri_dir: &Path,
  package_manager: PackageManager,
) -> InstalledPlugins {
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

  let know_plugins = helpers::plugins::known_plugins();
  let crate_names: Vec<String> = know_plugins
    .keys()
    .map(|plugin_name| format!("tauri-plugin-{plugin_name}"))
    .collect();
  let npm_names: Vec<String> = know_plugins
    .keys()
    .map(|plugin_name| format!("@tauri-apps/plugin-{plugin_name}"))
    .collect();

  let mut rust_plugins: HashMap<String, semver::Version> = crate_names
    .iter()
    .filter_map(|crate_name| {
      let crate_version =
        crate_version(tauri_dir, manifest.as_ref(), lock.as_ref(), crate_name).version?;
      let crate_version = semver::Version::parse(&crate_version)
        .inspect_err(|_| {
          log::error!("Failed to parse version `{crate_version}` for crate `{crate_name}`");
        })
        .ok()?;
      Some((crate_name.clone(), crate_version))
    })
    .collect();

  let mut npm_plugins = package_manager
    .current_package_versions(&npm_names, frontend_dir)
    .unwrap_or_default();

  let installed_plugins = crate_names
    .iter()
    .zip(npm_names.iter())
    .filter_map(|(crate_name, npm_name)| {
      let (crate_name, crate_version) = rust_plugins.remove_entry(crate_name)?;
      let (npm_name, npm_version) = npm_plugins.remove_entry(npm_name)?;
      Some(InstalledPlugin {
        npm_name,
        npm_version,
        crate_name,
        crate_version,
      })
    })
    .collect();

  InstalledPlugins(installed_plugins)
}

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
