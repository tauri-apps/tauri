// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::path::Path;

use crate::helpers::{config::Config, manifest::Manifest};
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

pub fn get_bundler_settings(
  app_settings: rust::AppSettings,
  target: Option<String>,
  manifest: &Manifest,
  config: &Config,
  out_dir: &Path,
  verbose: bool,
  package_types: Option<Vec<PackageType>>,
) -> crate::Result<Settings> {
  let mut settings_builder = SettingsBuilder::new()
    .package_settings(app_settings.get_package_settings())
    .bundle_settings(app_settings.get_bundle_settings(config, manifest)?)
    .binaries(app_settings.get_binaries(config)?)
    .project_out_directory(out_dir);

  if verbose {
    settings_builder = settings_builder.verbose();
  }

  if let Some(types) = package_types {
    settings_builder = settings_builder.package_types(types);
  }

  if let Some(target) = target {
    settings_builder = settings_builder.target(target);
  }

  settings_builder.build().map_err(Into::into)
}
