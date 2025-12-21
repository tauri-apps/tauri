// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  iter,
  path::{Path, PathBuf},
};

use crate::{
  helpers::{
    self,
    cargo_manifest::{cargo_manifest_and_lock, crate_version},
    npm::PackageManager,
  },
  Error,
};

use super::{packages_nodejs, packages_rust, SectionItem};

#[derive(Debug)]
pub struct InstalledPackage {
  pub crate_name: String,
  pub npm_name: String,
  pub crate_version: semver::Version,
  pub npm_version: semver::Version,
}

#[derive(Debug)]
pub struct InstalledPackages(Vec<InstalledPackage>);

impl InstalledPackages {
  pub fn mismatched(&self) -> Vec<&InstalledPackage> {
    self
      .0
      .iter()
      .filter(|p| {
        p.crate_version.major != p.npm_version.major || p.crate_version.minor != p.npm_version.minor
      })
      .collect()
  }
}

pub fn installed_tauri_packages(
  frontend_dir: &Path,
  tauri_dir: &Path,
  package_manager: PackageManager,
) -> InstalledPackages {
  let know_plugins = helpers::plugins::known_plugins();
  let crate_names: Vec<String> = iter::once("tauri".to_owned())
    .chain(
      know_plugins
        .keys()
        .map(|plugin_name| format!("tauri-plugin-{plugin_name}")),
    )
    .collect();
  let npm_names: Vec<String> = iter::once("@tauri-apps/api".to_owned())
    .chain(
      know_plugins
        .keys()
        .map(|plugin_name| format!("@tauri-apps/plugin-{plugin_name}")),
    )
    .collect();

  let (manifest, lock) = cargo_manifest_and_lock(tauri_dir);

  let mut rust_plugins: HashMap<String, semver::Version> = crate_names
    .iter()
    .filter_map(|crate_name| {
      let crate_version =
        crate_version(tauri_dir, manifest.as_ref(), lock.as_ref(), crate_name).version?;
      let crate_version = semver::Version::parse(&crate_version)
        .inspect_err(|_| {
          // On first run there's no lockfile yet so we get the version requirement from Cargo.toml.
          // In our templates that's `2` which is not a valid semver version but a version requirement.
          // log::error confused users so we use log::debug to still be able to see this error if needed.
          log::debug!("Failed to parse version `{crate_version}` for crate `{crate_name}`");
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
      Some(InstalledPackage {
        npm_name,
        npm_version,
        crate_name,
        crate_version,
      })
    })
    .collect();

  InstalledPackages(installed_plugins)
}

pub fn items(
  frontend_dir: Option<&PathBuf>,
  tauri_dir: Option<&Path>,
  package_manager: PackageManager,
) -> Vec<SectionItem> {
  let mut items = Vec::new();

  if tauri_dir.is_some() || frontend_dir.is_some() {
    if let Some(tauri_dir) = tauri_dir {
      let (manifest, lock) = cargo_manifest_and_lock(tauri_dir);

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

pub fn check_mismatched_packages(frontend_dir: &Path, tauri_path: &Path) -> crate::Result<()> {
  let installed_packages = installed_tauri_packages(
    frontend_dir,
    tauri_path,
    PackageManager::from_project(frontend_dir),
  );
  let mismatched_packages = installed_packages.mismatched();
  if mismatched_packages.is_empty() {
    return Ok(());
  }
  let mismatched_text = mismatched_packages
    .iter()
    .map(
      |InstalledPackage {
         crate_name,
         crate_version,
         npm_name,
         npm_version,
       }| format!("{crate_name} (v{crate_version}) : {npm_name} (v{npm_version})"),
    )
    .collect::<Vec<_>>()
    .join("\n");
  Err(Error::GenericError(format!("Found version mismatched Tauri packages. Make sure the NPM package and Rust crate versions are on the same major/minor releases:\n{mismatched_text}")))
}
