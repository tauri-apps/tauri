// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::ExitStatus,
  sync::Arc,
};

use crate::helpers::config::Config;
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

pub use rust::{MobileOptions, Options, Rust as AppInterface};

pub trait DevProcess {
  fn kill(&self) -> std::io::Result<()>;
  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>>;
  fn wait(&self) -> std::io::Result<ExitStatus>;
  fn manually_killed_process(&self) -> bool;
  fn is_building_app(&self) -> bool;
}

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
  fn app_name(&self) -> Option<String>;
  fn lib_name(&self) -> Option<String>;

  fn get_bundler_settings(
    &self,
    options: Options,
    config: &Config,
    out_dir: &Path,
    package_types: Vec<PackageType>,
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

    SettingsBuilder::new()
      .package_settings(self.get_package_settings())
      .bundle_settings(self.get_bundle_settings(config, &enabled_features)?)
      .binaries(self.get_binaries(config, &target)?)
      .project_out_directory(out_dir)
      .target(target)
      .package_types(package_types)
      .build()
      .map_err(Into::into)
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
  fn app_settings(&self) -> Arc<Self::AppSettings>;
  fn env(&self) -> HashMap<&str, String>;
  fn build(&mut self, options: Options) -> crate::Result<()>;
  fn dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    options: Options,
    on_exit: F,
  ) -> crate::Result<()>;
  fn mobile_dev<R: Fn(MobileOptions) -> crate::Result<Box<dyn DevProcess + Send>>>(
    &mut self,
    options: MobileOptions,
    runner: R,
  ) -> crate::Result<()>;
}
