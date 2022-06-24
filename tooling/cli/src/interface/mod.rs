// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::{
  path::{Path, PathBuf},
  process::ExitStatus,
};

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
  fn app_binary_path(&self, options: &Options) -> crate::Result<PathBuf>;
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

pub trait DevProcess {
  fn kill(&self) -> std::io::Result<()>;
  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>>;
}

#[derive(Debug)]
pub enum ExitReason {
  /// Killed manually.
  TriggeredKill,
  /// App compilation failed.
  CompilationFailed,
  /// Regular exit.
  NormalExit,
}

pub trait Interface: Sized {
  type AppSettings: AppSettings;
  type Dev: DevProcess;

  fn new(config: &Config) -> crate::Result<Self>;
  fn app_settings(&self) -> &Self::AppSettings;
  fn build(&mut self, options: Options) -> crate::Result<()>;
  fn dev<F: FnOnce(ExitStatus, ExitReason) + Send + 'static>(
    &mut self,
    options: Options,
    manifest: &Manifest,
    on_exit: F,
  ) -> crate::Result<Self::Dev>;
}
