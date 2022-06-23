// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::path::{Path, PathBuf};

use crate::helpers::{config::Config, manifest::Manifest};
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

pub use rust::{Options, Rust as AppInterface};

pub trait AppSettings {
  fn get_package_settings(&self) -> tauri_bundler::PackageSettings;
  fn get_bundle_settings(
    &self,
    config: &Config,
    manifest: &Manifest,
    features: &[String],
  ) -> crate::Result<tauri_bundler::BundleSettings>;
  fn get_out_dir(&self, options: &Options) -> crate::Result<PathBuf>;
  fn bin_name(&self) -> String;
  fn get_binaries(
    &self,
    config: &Config,
    target: &str,
  ) -> crate::Result<Vec<tauri_bundler::BundleBinary>>;

  fn get_bundler_settings(
    &self,
    options: &Options,
    manifest: &Manifest,
    config: &Config,
    out_dir: &Path,
    package_types: Option<Vec<PackageType>>,
  ) -> crate::Result<Settings> {
    let no_default_features = options.args.contains(&"--no-default-features".into());
    let mut enabled_features = options.features.clone().unwrap_or_default();
    if !no_default_features {
      enabled_features.push("default".into());
    }

    let target: String = if let Some(target) = options.target.clone() {
      target
    } else {
      tauri_utils::platform::target_triple()?
    };

    let mut settings_builder = SettingsBuilder::new()
      .package_settings(self.get_package_settings())
      .bundle_settings(self.get_bundle_settings(config, manifest, &enabled_features)?)
      .binaries(self.get_binaries(config, &target)?)
      .project_out_directory(out_dir)
      .target(target);

    if let Some(types) = package_types {
      settings_builder = settings_builder.package_types(types);
    }

    settings_builder.build().map_err(Into::into)
  }
}

pub trait Interface: Sized {
  type AppSettings: AppSettings;

  fn new(config: &Config) -> crate::Result<Self>;
  fn app_settings(&self) -> &Self::AppSettings;
  fn build(&self, options: Options) -> crate::Result<()>;
}
