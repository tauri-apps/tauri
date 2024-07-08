// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::ExitStatus,
};

use crate::helpers::config::Config;
use anyhow::Context;
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

pub use rust::{Options, Rust as AppInterface};

pub trait AppSettings {
  fn get_package_settings(&self) -> tauri_bundler::PackageSettings;
  fn get_bundle_settings(
    &self,
    config: &Config,
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
      .bundle_settings(self.get_bundle_settings(config, &enabled_features)?)
      .binaries(self.get_binaries(config, &target)?)
      .project_out_directory(out_dir)
      .target(target);

    if let Some(types) = package_types {
      settings_builder = settings_builder.package_types(types);
    }

    if config.tauri.bundle.use_local_tools_dir {
      settings_builder = settings_builder.local_tools_directory(
        rust::get_cargo_metadata()
          .context("failed to get cargo metadata")?
          .target_directory,
      )
    }

    settings_builder.build().map_err(Into::into)
  }
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

  fn new(config: &Config, target: Option<String>) -> crate::Result<Self>;
  fn app_settings(&self) -> &Self::AppSettings;
  fn env(&self) -> HashMap<&str, String>;
  fn build(&mut self, options: Options) -> crate::Result<()>;
  fn dev<F: Fn(ExitStatus, ExitReason) + Send + Sync + 'static>(
    &mut self,
    options: Options,
    on_exit: F,
  ) -> crate::Result<()>;
}
