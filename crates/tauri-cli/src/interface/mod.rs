// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod bindings;
pub mod rust;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::ExitStatus,
  sync::Arc,
};

use crate::{
  error::Context,
  helpers::{
    app_paths::Dirs,
    config::{Config, ConfigMetadata},
  },
  ConfigValue,
};
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};
use tauri_utils::config::RunnerConfig;

pub use bindings::Bindings as BindingsInterface;
pub use rust::Rust as AppInterface;

/// Which project interface drives `dev`/`build` for the resolved Tauri directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceKind {
  /// A Rust (cargo) project — the default, detected by `Cargo.toml`.
  Rust,
  /// A tauri-ffi bindings project (Node.js, Deno, Python, …): no `Cargo.toml`,
  /// the app is run by the `build > runner` command from the config.
  Bindings,
}

/// Detects the interface kind for a Tauri directory.
pub fn detect_interface_kind(tauri_dir: &Path) -> InterfaceKind {
  if tauri_dir.join("Cargo.toml").exists() {
    InterfaceKind::Rust
  } else {
    InterfaceKind::Bindings
  }
}

/// A project interface: how the CLI builds and runs the application.
///
/// Implementations only need this trait (plus [`AppSettings`]) — `dev` and
/// `build` dispatch through it, so new runners must not require changes
/// elsewhere in the CLI.
pub trait Interface: Sized {
  type AppSettings: AppSettings;

  fn new(config: &Config, target: Option<String>, tauri_dir: &Path) -> crate::Result<Self>;
  fn app_settings(&self) -> Arc<Self::AppSettings>;
  /// Environment exposed to hook scripts (`beforeDevCommand` etc.) and the app.
  fn env(&self) -> HashMap<&str, String>;
  /// Lets the interface adjust build args/features (e.g. cargo feature flags).
  fn build_options(&self, _args: &mut Vec<String>, _features: &mut Vec<String>, _mobile: bool) {}
  /// Builds the application, returning the path to the main artifact.
  fn build(&mut self, options: Options, dirs: &Dirs) -> crate::Result<PathBuf>;
  /// Runs the application in development mode, blocking until it exits.
  fn dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    config: &mut ConfigMetadata,
    options: Options,
    on_exit: F,
    dirs: &Dirs,
  ) -> crate::Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct Options {
  pub runner: Option<RunnerConfig>,
  pub debug: bool,
  pub target: Option<String>,
  pub features: Vec<String>,
  pub args: Vec<String>,
  pub config: Vec<ConfigValue>,
  pub no_watch: bool,
  pub skip_stapling: bool,
  pub additional_watch_folders: Vec<PathBuf>,
}

impl From<crate::build::Options> for Options {
  fn from(options: crate::build::Options) -> Self {
    Self {
      runner: options.runner,
      debug: options.debug,
      target: options.target,
      features: options.features,
      args: options.args,
      config: options.config,
      no_watch: true,
      skip_stapling: options.skip_stapling,
      additional_watch_folders: Vec::new(),
    }
  }
}

impl From<crate::bundle::Options> for Options {
  fn from(options: crate::bundle::Options) -> Self {
    Self {
      debug: options.debug,
      config: options.config,
      target: options.target,
      features: options.features,
      no_watch: true,
      skip_stapling: options.skip_stapling,
      ..Default::default()
    }
  }
}

impl From<crate::dev::Options> for Options {
  fn from(options: crate::dev::Options) -> Self {
    Self {
      runner: options.runner,
      debug: !options.release_mode,
      target: options.target,
      features: options.features,
      args: options.args,
      config: options.config,
      no_watch: options.no_watch,
      skip_stapling: false,
      additional_watch_folders: options.additional_watch_folders,
    }
  }
}

#[derive(Debug, Clone)]
pub struct MobileOptions {
  pub debug: bool,
  pub features: Vec<String>,
  pub args: Vec<String>,
  pub config: Vec<ConfigValue>,
  pub no_watch: bool,
  pub additional_watch_folders: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct WatcherOptions {
  pub config: Vec<ConfigValue>,
  pub additional_watch_folders: Vec<PathBuf>,
}

pub trait DevProcess {
  fn kill(&self) -> std::io::Result<()>;
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
  #[allow(dead_code)]
  fn out_dir(&self, options: &Options, tauri_dir: &Path) -> crate::Result<PathBuf>;
  fn app_name(&self) -> Option<String>;
  fn lib_name(&self) -> Option<String>;
  /// The directory build artifacts land in.
  fn out_dir(&self, options: &Options, tauri_dir: &Path) -> crate::Result<PathBuf>;

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
