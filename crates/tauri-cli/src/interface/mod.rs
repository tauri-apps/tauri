// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::ExitStatus,
  sync::Arc,
};

use crate::{
  error::Context, helpers::app_paths::Dirs, helpers::config::Config,
  helpers::config::ConfigMetadata,
};
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

pub use rust::{MobileOptions, Options, Rust as AppInterface, WatcherOptions};

pub trait DevProcess {
  fn kill(&self) -> std::io::Result<()>;
  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>>;
  #[allow(unused)]
  fn wait(&self) -> std::io::Result<ExitStatus>;
  #[allow(unused)]
  fn manually_killed_process(&self) -> bool;
}

pub trait AppSettings {
  fn get_package_settings(&self) -> tauri_bundler::PackageSettings;
  fn get_bundle_settings(
    &self,
    options: &Options,
    config: &Config,
    features: &[String],
    tauri_dir: &Path,
  ) -> crate::Result<tauri_bundler::BundleSettings>;
  fn app_binary_path(&self, options: &Options, tauri_dir: &Path) -> crate::Result<PathBuf>;
  fn get_binaries(
    &self,
    options: &Options,
    tauri_dir: &Path,
  ) -> crate::Result<Vec<tauri_bundler::BundleBinary>>;
  fn app_name(&self) -> Option<String>;
  fn lib_name(&self) -> Option<String>;

  fn get_bundler_settings(
    &self,
    options: Options,
    config: &Config,
    out_dir: &Path,
    package_types: Vec<PackageType>,
    tauri_dir: &Path,
  ) -> crate::Result<Settings> {
    let no_default_features = options.args.contains(&"--no-default-features".into());
    let mut enabled_features = options.features.clone();
    if !no_default_features {
      enabled_features.push("default".into());
    }

    let target: String = if let Some(target) = options.target.clone() {
      target
    } else {
      tauri_utils::platform::target_triple().context("failed to get target triple")?
    };

    let mut bins = self.get_binaries(&options, tauri_dir)?;
    if let Some(main_binary_name) = &config.main_binary_name {
      let main = bins.iter_mut().find(|b| b.main()).context("no main bin?")?;
      main.set_name(main_binary_name.to_owned());
    }

    let mut settings_builder = SettingsBuilder::new()
      .package_settings(self.get_package_settings())
      .bundle_settings(self.get_bundle_settings(&options, config, &enabled_features, tauri_dir)?)
      .binaries(bins)
      .project_out_directory(out_dir)
      .target(target)
      .package_types(package_types);

    if config.bundle.use_local_tools_dir {
      settings_builder = settings_builder.local_tools_directory(
        rust::get_cargo_metadata(tauri_dir)
          .context("failed to get cargo metadata")?
          .target_directory,
      )
    }

    settings_builder
      .build()
      .map_err(Box::new)
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

  fn new(config: &Config, target: Option<String>, tauri_dir: &Path) -> crate::Result<Self>;
  fn app_settings(&self) -> Arc<Self::AppSettings>;
  fn env(&self) -> HashMap<&str, String>;
  fn build(&mut self, options: Options, dirs: &Dirs) -> crate::Result<PathBuf>;
  fn dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    config: &mut ConfigMetadata,
    options: Options,
    on_exit: F,
    dirs: &Dirs,
  ) -> crate::Result<()>;
  fn mobile_dev<
    R: Fn(MobileOptions, &ConfigMetadata) -> crate::Result<Box<dyn DevProcess + Send>>,
  >(
    &mut self,
    config: &mut ConfigMetadata,
    options: MobileOptions,
    runner: R,
    dirs: &Dirs,
  ) -> crate::Result<()>;
  fn watch<R: Fn(&ConfigMetadata) -> crate::Result<Box<dyn DevProcess + Send>>>(
    &mut self,
    config: &mut ConfigMetadata,
    options: WatcherOptions,
    runner: R,
    dirs: &Dirs,
  ) -> crate::Result<()>;
}
