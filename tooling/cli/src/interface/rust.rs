// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  ffi::OsStr,
  fs::{File, FileType},
  io::{BufRead, Read, Write},
  path::{Path, PathBuf},
  process::Command,
  str::FromStr,
  sync::{mpsc::sync_channel, Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::Context;
use heck::ToKebabCase;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{debug, error, info};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use serde::{Deserialize, Deserializer};
use tauri_bundler::{
  AppCategory, AppImageSettings, BundleBinary, BundleSettings, DebianSettings, DmgSettings,
  MacOsSettings, PackageSettings, Position, RpmSettings, Size, UpdaterSettings, WindowsSettings,
};
use tauri_utils::config::{parse::is_configuration_file, DeepLinkProtocol};

use super::{AppSettings, DevProcess, ExitReason, Interface};
use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::{nsis_settings, reload as reload_config, wix_settings, BundleResources, Config},
};
use tauri_utils::{display_path, platform::Target};

mod cargo_config;
mod desktop;
pub mod installation;
pub mod manifest;
use cargo_config::Config as CargoConfig;
use manifest::{rewrite_manifest, Manifest};

#[derive(Debug, Default, Clone)]
pub struct Options {
  pub runner: Option<String>,
  pub debug: bool,
  pub target: Option<String>,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub config: Option<String>,
  pub no_watch: bool,
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
    }
  }
}

#[derive(Debug, Clone)]
pub struct MobileOptions {
  pub debug: bool,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub config: Option<String>,
  pub no_watch: bool,
}

#[derive(Debug)]
pub struct RustupTarget {
  name: String,
  installed: bool,
}

pub struct Rust {
  app_settings: Arc<RustAppSettings>,
  config_features: Vec<String>,
  product_name: Option<String>,
  available_targets: Option<Vec<RustupTarget>>,
}

impl Interface for Rust {
  type AppSettings = RustAppSettings;

  fn new(config: &Config, target: Option<String>) -> crate::Result<Self> {
    let manifest = {
      let (tx, rx) = sync_channel(1);
      let mut watcher = new_debouncer(Duration::from_secs(1), move |r| {
        if let Ok(events) = r {
          let _ = tx.send(events);
        }
      })
      .unwrap();
      watcher
        .watcher()
        .watch(&tauri_dir().join("Cargo.toml"), RecursiveMode::Recursive)?;
      let manifest = rewrite_manifest(config)?;
      let now = Instant::now();
      let timeout = Duration::from_secs(2);
      loop {
        if now.elapsed() >= timeout {
          break;
        }
        if rx.try_recv().is_ok() {
          break;
        }
      }
      manifest
    };

    if let Some(minimum_system_version) = &config.bundle.macos.minimum_system_version {
      std::env::set_var("MACOSX_DEPLOYMENT_TARGET", minimum_system_version);
    }

    let app_settings = RustAppSettings::new(config, manifest, target)?;

    Ok(Self {
      app_settings: Arc::new(app_settings),
      config_features: config.build.features.clone().unwrap_or_default(),
      product_name: config.product_name.clone(),
      available_targets: None,
    })
  }

  fn app_settings(&self) -> Arc<Self::AppSettings> {
    self.app_settings.clone()
  }

  fn build(&mut self, options: Options) -> crate::Result<()> {
    desktop::build(
      options,
      &self.app_settings,
      self.product_name.clone(),
      &mut self.available_targets,
      self.config_features.clone(),
    )?;
    Ok(())
  }

  fn dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    mut options: Options,
    on_exit: F,
  ) -> crate::Result<()> {
    let on_exit = Arc::new(on_exit);

    let mut run_args = Vec::new();
    dev_options(
      false,
      &mut options.args,
      &mut run_args,
      &mut options.features,
      &self.app_settings,
    );

    if options.no_watch {
      let (tx, rx) = sync_channel(1);
      self.run_dev(options, run_args, move |status, reason| {
        tx.send(()).unwrap();
        on_exit(status, reason)
      })?;

      rx.recv().unwrap();
      Ok(())
    } else {
      let config = options.config.clone();
      let run = Arc::new(|rust: &mut Rust| {
        let on_exit = on_exit.clone();
        rust.run_dev(options.clone(), run_args.clone(), move |status, reason| {
          on_exit(status, reason)
        })
      });
      self.run_dev_watcher(config, run)
    }
  }

  fn mobile_dev<R: Fn(MobileOptions) -> crate::Result<Box<dyn DevProcess + Send>>>(
    &mut self,
    mut options: MobileOptions,
    runner: R,
  ) -> crate::Result<()> {
    let mut run_args = Vec::new();
    dev_options(
      true,
      &mut options.args,
      &mut run_args,
      &mut options.features,
      &self.app_settings,
    );

    if options.no_watch {
      runner(options)?;
      Ok(())
    } else {
      let config = options.config.clone();
      let run = Arc::new(|_rust: &mut Rust| runner(options.clone()));
      self.run_dev_watcher(config, run)
    }
  }

  fn env(&self) -> HashMap<&str, String> {
    let mut env = HashMap::new();
    env.insert(
      "TAURI_ENV_TARGET_TRIPLE",
      self.app_settings.target_triple.clone(),
    );

    let target_triple = &self.app_settings.target_triple;
    let target_components: Vec<&str> = target_triple.split('-').collect();
    let (arch, host, _host_env) = match target_components.as_slice() {
      // 3 components like aarch64-apple-darwin
      [arch, _, host] => (*arch, *host, None),
      // 4 components like x86_64-pc-windows-msvc and aarch64-apple-ios-sim
      [arch, _, host, host_env] => (*arch, *host, Some(*host_env)),
      _ => {
        log::warn!("Invalid target triple: {}", target_triple);
        return env;
      }
    };

    env.insert("TAURI_ENV_ARCH", arch.into());
    env.insert("TAURI_ENV_PLATFORM", host.into());
    env.insert(
      "TAURI_ENV_FAMILY",
      match host {
        "windows" => "windows".into(),
        _ => "unix".into(),
      },
    );

    env
  }
}

struct IgnoreMatcher(Vec<Gitignore>);

impl IgnoreMatcher {
  fn is_ignore(&self, path: &Path, is_dir: bool) -> bool {
    for gitignore in &self.0 {
      if gitignore.matched(path, is_dir).is_ignore() {
        return true;
      }
    }
    false
  }
}

fn build_ignore_matcher(dir: &Path) -> IgnoreMatcher {
  let mut matchers = Vec::new();

  // ignore crate doesn't expose an API to build `ignore::gitignore::GitIgnore`
  // with custom ignore file names so we have to walk the directory and collect
  // our custom ignore files and add it using `ignore::gitignore::GitIgnoreBuilder::add`
  for entry in ignore::WalkBuilder::new(dir)
    .require_git(false)
    .ignore(false)
    .overrides(
      ignore::overrides::OverrideBuilder::new(dir)
        .add(".taurignore")
        .unwrap()
        .build()
        .unwrap(),
    )
    .build()
    .flatten()
  {
    let path = entry.path();
    if path.file_name() == Some(OsStr::new(".taurignore")) {
      let mut ignore_builder = GitignoreBuilder::new(path.parent().unwrap());

      ignore_builder.add(path);

      if let Ok(ignore_file) = std::env::var("TAURI_CLI_WATCHER_IGNORE_FILENAME") {
        ignore_builder.add(dir.join(ignore_file));
      }

      for line in crate::dev::TAURI_CLI_BUILTIN_WATCHER_IGNORE_FILE
        .lines()
        .flatten()
      {
        let _ = ignore_builder.add_line(None, &line);
      }

      matchers.push(ignore_builder.build().unwrap());
    }
  }

  IgnoreMatcher(matchers)
}

fn lookup<F: FnMut(FileType, PathBuf)>(dir: &Path, mut f: F) {
  let mut default_gitignore = std::env::temp_dir();
  default_gitignore.push(".tauri");
  let _ = std::fs::create_dir_all(&default_gitignore);
  default_gitignore.push(".gitignore");
  if !default_gitignore.exists() {
    if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
      let _ = file.write_all(crate::dev::TAURI_CLI_BUILTIN_WATCHER_IGNORE_FILE);
    }
  }

  let mut builder = ignore::WalkBuilder::new(dir);
  builder.add_custom_ignore_filename(".taurignore");
  let _ = builder.add_ignore(default_gitignore);
  if let Ok(ignore_file) = std::env::var("TAURI_CLI_WATCHER_IGNORE_FILENAME") {
    builder.add_ignore(ignore_file);
  }
  builder.require_git(false).ignore(false).max_depth(Some(1));

  for entry in builder.build().flatten() {
    f(entry.file_type().unwrap(), dir.join(entry.path()));
  }
}

fn shared_options(
  mobile: bool,
  args: &mut Vec<String>,
  features: &mut Option<Vec<String>>,
  app_settings: &RustAppSettings,
) {
  if mobile {
    args.push("--lib".into());
    features
      .get_or_insert(Vec::new())
      .push("tauri/rustls-tls".into());
  } else {
    args.push("--bins".into());
    let all_features = app_settings
      .manifest
      .lock()
      .unwrap()
      .all_enabled_features(if let Some(f) = features { f } else { &[] });
    if !all_features.contains(&"tauri/rustls-tls".into()) {
      features
        .get_or_insert(Vec::new())
        .push("tauri/native-tls".into());
    }
  }
}

fn dev_options(
  mobile: bool,
  args: &mut Vec<String>,
  run_args: &mut Vec<String>,
  features: &mut Option<Vec<String>>,
  app_settings: &RustAppSettings,
) {
  let mut dev_args = Vec::new();
  let mut reached_run_args = false;
  for arg in args.clone() {
    if reached_run_args {
      run_args.push(arg);
    } else if arg == "--" {
      reached_run_args = true;
    } else {
      dev_args.push(arg);
    }
  }
  *args = dev_args;

  shared_options(mobile, args, features, app_settings);

  if !args.contains(&"--no-default-features".into()) {
    let manifest_features = app_settings.manifest.lock().unwrap().features();
    let enable_features: Vec<String> = manifest_features
      .get("default")
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|feature| {
        if let Some(manifest_feature) = manifest_features.get(feature) {
          !manifest_feature.contains(&"tauri/custom-protocol".into())
        } else {
          feature != "tauri/custom-protocol"
        }
      })
      .collect();
    args.push("--no-default-features".into());
    if !enable_features.is_empty() {
      features.get_or_insert(Vec::new()).extend(enable_features);
    }
  }
}

impl Rust {
  pub fn build_options(
    &self,
    args: &mut Vec<String>,
    features: &mut Option<Vec<String>>,
    mobile: bool,
  ) {
    features
      .get_or_insert(Vec::new())
      .push("custom-protocol".into());
    shared_options(mobile, args, features, &self.app_settings);
  }

  fn run_dev<F: Fn(Option<i32>, ExitReason) + Send + Sync + 'static>(
    &mut self,
    options: Options,
    run_args: Vec<String>,
    on_exit: F,
  ) -> crate::Result<Box<dyn DevProcess + Send>> {
    desktop::run_dev(
      options,
      run_args,
      &mut self.available_targets,
      self.config_features.clone(),
      &self.app_settings,
      self.product_name.clone(),
      on_exit,
    )
    .map(|c| Box::new(c) as Box<dyn DevProcess + Send>)
  }

  fn run_dev_watcher<F: Fn(&mut Rust) -> crate::Result<Box<dyn DevProcess + Send>>>(
    &mut self,
    config: Option<String>,
    run: Arc<F>,
  ) -> crate::Result<()> {
    let child = run(self)?;

    let process = Arc::new(Mutex::new(child));
    let (tx, rx) = sync_channel(1);
    let app_path = app_dir();
    let tauri_path = tauri_dir();
    let workspace_path = get_workspace_dir()?;

    let watch_folders = if tauri_path == workspace_path {
      vec![tauri_path]
    } else {
      let cargo_settings = CargoSettings::load(&workspace_path)?;
      cargo_settings
        .workspace
        .as_ref()
        .map(|w| {
          w.members
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|p| workspace_path.join(p))
            .collect()
        })
        .unwrap_or_else(|| vec![tauri_path])
    };

    let watch_folders = watch_folders.iter().map(Path::new).collect::<Vec<_>>();
    let common_ancestor = common_path::common_path_all(watch_folders.clone()).unwrap();
    let ignore_matcher = build_ignore_matcher(&common_ancestor);

    let mut watcher = new_debouncer(Duration::from_secs(1), move |r| {
      if let Ok(events) = r {
        tx.send(events).unwrap()
      }
    })
    .unwrap();
    for path in watch_folders {
      if !ignore_matcher.is_ignore(path, true) {
        info!("Watching {} for changes...", display_path(path));
        lookup(path, |file_type, p| {
          if p != path {
            debug!("Watching {} for changes...", display_path(&p));
            let _ = watcher.watcher().watch(
              &p,
              if file_type.is_dir() {
                RecursiveMode::Recursive
              } else {
                RecursiveMode::NonRecursive
              },
            );
          }
        });
      }
    }

    loop {
      if let Ok(events) = rx.recv() {
        for event in events {
          let event_path = event.path;

          if !ignore_matcher.is_ignore(&event_path, event_path.is_dir()) {
            if is_configuration_file(self.app_settings.target, &event_path) {
              match reload_config(config.as_deref()) {
                Ok(config) => {
                  info!("Tauri configuration changed. Rewriting manifest...");
                  *self.app_settings.manifest.lock().unwrap() =
                    rewrite_manifest(config.lock().unwrap().as_ref().unwrap())?
                }
                Err(err) => {
                  let p = process.lock().unwrap();
                  if p.is_building_app() {
                    p.kill().with_context(|| "failed to kill app process")?;
                  }
                  error!("{}", err);
                }
              }
            } else {
              info!(
                "File {} changed. Rebuilding application...",
                display_path(event_path.strip_prefix(app_path).unwrap_or(&event_path))
              );
              // When tauri.conf.json is changed, rewrite_manifest will be called
              // which will trigger the watcher again
              // So the app should only be started when a file other than tauri.conf.json is changed
              let mut p = process.lock().unwrap();
              p.kill().with_context(|| "failed to kill app process")?;
              // wait for the process to exit
              loop {
                if let Ok(Some(_)) = p.try_wait() {
                  break;
                }
              }
              *p = run(self)?;
            }
          }
        }
      }
    }
  }
}

// Taken from https://github.com/rust-lang/cargo/blob/70898e522116f6c23971e2a554b2dc85fd4c84cd/src/cargo/util/toml/mod.rs#L1008-L1065
/// Enum that allows for the parsing of `field.workspace = true` in a Cargo.toml
///
/// It allows for things to be inherited from a workspace or defined as needed
#[derive(Clone, Debug)]
pub enum MaybeWorkspace<T> {
  Workspace(TomlWorkspaceField),
  Defined(T),
}

impl<'de, T: Deserialize<'de>> serde::de::Deserialize<'de> for MaybeWorkspace<T> {
  fn deserialize<D>(deserializer: D) -> Result<MaybeWorkspace<T>, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    let value = serde_value::Value::deserialize(deserializer)?;
    if let Ok(workspace) = TomlWorkspaceField::deserialize(
      serde_value::ValueDeserializer::<D::Error>::new(value.clone()),
    ) {
      return Ok(MaybeWorkspace::Workspace(workspace));
    }
    T::deserialize(serde_value::ValueDeserializer::<D::Error>::new(value))
      .map(MaybeWorkspace::Defined)
  }
}

impl<T> MaybeWorkspace<T> {
  fn resolve(
    self,
    label: &str,
    get_ws_field: impl FnOnce() -> anyhow::Result<T>,
  ) -> anyhow::Result<T> {
    match self {
      MaybeWorkspace::Defined(value) => Ok(value),
      MaybeWorkspace::Workspace(TomlWorkspaceField { workspace: true }) => {
        get_ws_field().context(format!(
          "error inheriting `{label}` from workspace root manifest's `workspace.package.{label}`"
        ))
      }
      MaybeWorkspace::Workspace(TomlWorkspaceField { workspace: false }) => Err(anyhow::anyhow!(
        "`workspace=false` is unsupported for `package.{}`",
        label,
      )),
    }
  }
  fn _as_defined(&self) -> Option<&T> {
    match self {
      MaybeWorkspace::Workspace(_) => None,
      MaybeWorkspace::Defined(defined) => Some(defined),
    }
  }
}

#[derive(Deserialize, Clone, Debug)]
pub struct TomlWorkspaceField {
  workspace: bool,
}

/// The `workspace` section of the app configuration (read from Cargo.toml).
#[derive(Clone, Debug, Deserialize)]
struct WorkspaceSettings {
  /// the workspace members.
  members: Option<Vec<String>>,
  package: Option<WorkspacePackageSettings>,
}

#[derive(Clone, Debug, Deserialize)]
struct WorkspacePackageSettings {
  authors: Option<Vec<String>>,
  description: Option<String>,
  homepage: Option<String>,
  version: Option<String>,
  license: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct BinarySettings {
  name: String,
  path: Option<String>,
}

/// The package settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CargoPackageSettings {
  /// the package's name.
  pub name: Option<String>,
  /// the package's version.
  pub version: Option<MaybeWorkspace<String>>,
  /// the package's description.
  pub description: Option<MaybeWorkspace<String>>,
  /// the package's homepage.
  pub homepage: Option<MaybeWorkspace<String>>,
  /// the package's authors.
  pub authors: Option<MaybeWorkspace<Vec<String>>>,
  /// the package's license.
  pub license: Option<MaybeWorkspace<String>>,
  /// the default binary to run.
  pub default_run: Option<String>,
}

/// The Cargo settings (Cargo.toml root descriptor).
#[derive(Clone, Debug, Deserialize)]
struct CargoSettings {
  /// the package settings.
  ///
  /// it's optional because ancestor workspace Cargo.toml files may not have package info.
  package: Option<CargoPackageSettings>,
  /// the workspace settings.
  ///
  /// it's present if the read Cargo.toml belongs to a workspace root.
  workspace: Option<WorkspaceSettings>,
  /// the binary targets configuration.
  bin: Option<Vec<BinarySettings>>,
}

impl CargoSettings {
  /// Try to load a set of CargoSettings from a "Cargo.toml" file in the specified directory.
  fn load(dir: &Path) -> crate::Result<Self> {
    let toml_path = dir.join("Cargo.toml");
    let mut toml_str = String::new();
    let mut toml_file = File::open(toml_path).with_context(|| "failed to open Cargo.toml")?;
    toml_file
      .read_to_string(&mut toml_str)
      .with_context(|| "failed to read Cargo.toml")?;
    toml::from_str(&toml_str)
      .with_context(|| "failed to parse Cargo.toml")
      .map_err(Into::into)
  }
}

pub struct RustAppSettings {
  manifest: Mutex<Manifest>,
  cargo_settings: CargoSettings,
  cargo_package_settings: CargoPackageSettings,
  cargo_ws_package_settings: Option<WorkspacePackageSettings>,
  package_settings: PackageSettings,
  cargo_config: CargoConfig,
  target_triple: String,
  target: Target,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DesktopDeepLinks {
  One(DeepLinkProtocol),
  List(Vec<DeepLinkProtocol>),
}

#[derive(Deserialize)]
pub struct UpdaterConfig {
  /// Signature public key.
  pub pubkey: String,
  /// The Windows configuration for the updater.
  #[serde(default)]
  pub windows: UpdaterWindowsConfig,
}

/// Install modes for the Windows update.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WindowsUpdateInstallMode {
  /// Specifies there's a basic UI during the installation process, including a final dialog box at the end.
  BasicUi,
  /// The quiet mode means there's no user interaction required.
  /// Requires admin privileges if the installer does.
  Quiet,
  /// Specifies unattended mode, which means the installation only shows a progress bar.
  Passive,
  // to add more modes, we need to check if the updater relaunch makes sense
  // i.e. for a full UI mode, the user can also mark the installer to start the app
}

impl Default for WindowsUpdateInstallMode {
  fn default() -> Self {
    Self::Passive
  }
}

impl<'de> Deserialize<'de> for WindowsUpdateInstallMode {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
      "basicui" => Ok(Self::BasicUi),
      "quiet" => Ok(Self::Quiet),
      "passive" => Ok(Self::Passive),
      _ => Err(serde::de::Error::custom(format!(
        "unknown update install mode '{s}'"
      ))),
    }
  }
}

impl WindowsUpdateInstallMode {
  /// Returns the associated `msiexec.exe` arguments.
  pub fn msiexec_args(&self) -> &'static [&'static str] {
    match self {
      Self::BasicUi => &["/qb+"],
      Self::Quiet => &["/quiet"],
      Self::Passive => &["/passive"],
    }
  }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterWindowsConfig {
  #[serde(default, alias = "install-mode")]
  pub install_mode: WindowsUpdateInstallMode,
}

impl AppSettings for RustAppSettings {
  fn get_package_settings(&self) -> PackageSettings {
    self.package_settings.clone()
  }

  fn get_bundle_settings(
    &self,
    config: &Config,
    features: &[String],
  ) -> crate::Result<BundleSettings> {
    let arch64bits =
      self.target_triple.starts_with("x86_64") || self.target_triple.starts_with("aarch64");

    let updater_settings = if let Some(updater_plugin_config) = config.plugins.0.get("updater") {
      let updater: UpdaterConfig = serde_json::from_value(updater_plugin_config.clone())?;
      Some(UpdaterSettings {
        pubkey: updater.pubkey,
        msiexec_args: Some(updater.windows.install_mode.msiexec_args()),
      })
    } else {
      None
    };

    let mut settings = tauri_config_to_bundle_settings(
      self,
      features,
      config.identifier.clone(),
      config.bundle.clone(),
      updater_settings,
      arch64bits,
    )?;

    if let Some(plugin_config) = config
      .plugins
      .0
      .get("deep-link")
      .and_then(|c| c.get("desktop").cloned())
    {
      let protocols: DesktopDeepLinks = serde_json::from_value(plugin_config.clone())?;
      settings.deep_link_protocols = Some(match protocols {
        DesktopDeepLinks::One(p) => vec![p],
        DesktopDeepLinks::List(p) => p,
      });
    }

    Ok(settings)
  }

  fn app_binary_path(&self, options: &Options) -> crate::Result<PathBuf> {
    let bin_name = self
      .cargo_package_settings()
      .name
      .clone()
      .expect("Cargo manifest must have the `package.name` field");

    let out_dir = self
      .out_dir(options.target.clone(), get_profile(options))
      .with_context(|| "failed to get project out directory")?;

    let binary_extension: String = if self.target_triple.contains("windows") {
      "exe"
    } else {
      ""
    }
    .into();

    Ok(out_dir.join(bin_name).with_extension(binary_extension))
  }

  fn get_binaries(&self, config: &Config, target: &str) -> crate::Result<Vec<BundleBinary>> {
    let mut binaries: Vec<BundleBinary> = vec![];

    let binary_extension: String = if target.contains("windows") {
      ".exe"
    } else {
      ""
    }
    .into();

    let target_os = target.split('-').nth(2).unwrap_or(std::env::consts::OS);

    if let Some(bin) = &self.cargo_settings.bin {
      let default_run = self
        .package_settings
        .default_run
        .clone()
        .unwrap_or_default();
      for binary in bin {
        binaries.push(
          if Some(&binary.name) == self.cargo_package_settings.name.as_ref()
            || binary.name.as_str() == default_run
          {
            BundleBinary::new(
              format!(
                "{}{}",
                config.binary_name().unwrap_or_else(|| binary.name.clone()),
                &binary_extension
              ),
              true,
            )
          } else {
            BundleBinary::new(
              format!("{}{}", binary.name.clone(), &binary_extension),
              false,
            )
          }
          .set_src_path(binary.path.clone()),
        )
      }
    }

    let mut bins_path = tauri_dir();
    bins_path.push("src/bin");
    if let Ok(fs_bins) = std::fs::read_dir(bins_path) {
      for entry in fs_bins {
        let path = entry?.path();
        if let Some(name) = path.file_stem() {
          let bin_exists = binaries.iter().any(|bin| {
            bin.name() == name || path.ends_with(bin.src_path().unwrap_or(&"".to_string()))
          });
          if !bin_exists {
            binaries.push(BundleBinary::new(
              format!("{}{}", name.to_string_lossy(), &binary_extension),
              false,
            ))
          }
        }
      }
    }

    if let Some(default_run) = self.package_settings.default_run.as_ref() {
      match binaries.iter_mut().find(|bin| bin.name() == default_run) {
        Some(bin) => {
          if let Some(bin_name) = config.binary_name() {
            bin.set_name(bin_name);
          }
        }
        None => {
          binaries.push(BundleBinary::new(
            format!(
              "{}{}",
              config
                .binary_name()
                .unwrap_or_else(|| default_run.to_string()),
              &binary_extension
            ),
            true,
          ));
        }
      }
    }

    match binaries.len() {
      0 => binaries.push(BundleBinary::new(
        if target_os == "linux" {
          self.package_settings.product_name.to_kebab_case()
        } else {
          format!(
            "{}{}",
            self.package_settings.product_name.clone(),
            &binary_extension
          )
        },
        true,
      )),
      1 => binaries.get_mut(0).unwrap().set_main(true),
      _ => {}
    }

    Ok(binaries)
  }

  fn app_name(&self) -> Option<String> {
    self
      .manifest
      .lock()
      .unwrap()
      .inner
      .as_table()
      .get("package")
      .and_then(|p| p.as_table())
      .and_then(|p| p.get("name"))
      .and_then(|n| n.as_str())
      .map(|n| n.to_string())
  }

  fn lib_name(&self) -> Option<String> {
    self
      .manifest
      .lock()
      .unwrap()
      .inner
      .as_table()
      .get("lib")
      .and_then(|p| p.as_table())
      .and_then(|p| p.get("name"))
      .and_then(|n| n.as_str())
      .map(|n| n.to_string())
  }
}

impl RustAppSettings {
  pub fn new(config: &Config, manifest: Manifest, target: Option<String>) -> crate::Result<Self> {
    let cargo_settings =
      CargoSettings::load(&tauri_dir()).with_context(|| "failed to load cargo settings")?;
    let cargo_package_settings = match &cargo_settings.package {
      Some(package_info) => package_info.clone(),
      None => {
        return Err(anyhow::anyhow!(
          "No package info in the config file".to_owned(),
        ))
      }
    };

    let ws_package_settings = CargoSettings::load(&get_workspace_dir()?)
      .with_context(|| "failed to load cargo settings from workspace root")?
      .workspace
      .and_then(|v| v.package);

    let package_settings = PackageSettings {
      product_name: config.product_name.clone().unwrap_or_else(|| {
        cargo_package_settings
          .name
          .clone()
          .expect("Cargo manifest must have the `package.name` field")
      }),
      version: config.version.clone().unwrap_or_else(|| {
        cargo_package_settings
          .version
          .clone()
          .expect("Cargo manifest must have the `package.version` field")
          .resolve("version", || {
            ws_package_settings
              .as_ref()
              .and_then(|p| p.version.clone())
              .ok_or_else(|| anyhow::anyhow!("Couldn't inherit value for `version` from workspace"))
          })
          .expect("Cargo project does not have a version")
      }),
      description: cargo_package_settings
        .description
        .clone()
        .map(|description| {
          description
            .resolve("description", || {
              ws_package_settings
                .as_ref()
                .and_then(|v| v.description.clone())
                .ok_or_else(|| {
                  anyhow::anyhow!("Couldn't inherit value for `description` from workspace")
                })
            })
            .unwrap()
        })
        .unwrap_or_default(),
      homepage: cargo_package_settings.homepage.clone().map(|homepage| {
        homepage
          .resolve("homepage", || {
            ws_package_settings
              .as_ref()
              .and_then(|v| v.homepage.clone())
              .ok_or_else(|| {
                anyhow::anyhow!("Couldn't inherit value for `homepage` from workspace")
              })
          })
          .unwrap()
      }),
      authors: cargo_package_settings.authors.clone().map(|authors| {
        authors
          .resolve("authors", || {
            ws_package_settings
              .as_ref()
              .and_then(|v| v.authors.clone())
              .ok_or_else(|| anyhow::anyhow!("Couldn't inherit value for `authors` from workspace"))
          })
          .unwrap()
      }),
      default_run: cargo_package_settings.default_run.clone(),
    };

    let cargo_config = CargoConfig::load(&tauri_dir())?;

    let target_triple = target.unwrap_or_else(|| {
      cargo_config
        .build()
        .target()
        .map(|t| t.to_string())
        .unwrap_or_else(|| {
          let output = Command::new("rustc")
            .args(["-vV"])
            .output()
            .expect("\"rustc\" could not be found, did you install Rust?");
          let stdout = String::from_utf8_lossy(&output.stdout);
          stdout
            .split('\n')
            .find(|l| l.starts_with("host:"))
            .unwrap()
            .replace("host:", "")
            .trim()
            .to_string()
        })
    });
    let target = Target::from_triple(&target_triple);

    Ok(Self {
      manifest: Mutex::new(manifest),
      cargo_settings,
      cargo_package_settings,
      cargo_ws_package_settings: ws_package_settings,
      package_settings,
      cargo_config,
      target_triple,
      target,
    })
  }

  pub fn cargo_package_settings(&self) -> &CargoPackageSettings {
    &self.cargo_package_settings
  }

  pub fn out_dir(&self, target: Option<String>, profile: String) -> crate::Result<PathBuf> {
    get_target_dir(
      target
        .as_deref()
        .or_else(|| self.cargo_config.build().target()),
      profile,
    )
  }
}

#[derive(Deserialize)]
struct CargoMetadata {
  target_directory: PathBuf,
  workspace_root: PathBuf,
}

fn get_cargo_metadata() -> crate::Result<CargoMetadata> {
  let output = Command::new("cargo")
    .args(["metadata", "--no-deps", "--format-version", "1"])
    .current_dir(tauri_dir())
    .output()?;

  if !output.status.success() {
    return Err(anyhow::anyhow!(
      "cargo metadata command exited with a non zero exit code: {}",
      String::from_utf8(output.stderr)?
    ));
  }

  Ok(serde_json::from_slice(&output.stdout)?)
}

/// This function determines the 'target' directory and suffixes it with the profile
/// to determine where the compiled binary will be located.
fn get_target_dir(target: Option<&str>, profile: String) -> crate::Result<PathBuf> {
  let mut path = get_cargo_metadata()
    .with_context(|| "failed to get cargo metadata")?
    .target_directory;

  if let Some(triple) = target {
    path.push(triple);
  }

  path.push(profile);

  Ok(path)
}

/// Executes `cargo metadata` to get the workspace directory.
pub fn get_workspace_dir() -> crate::Result<PathBuf> {
  Ok(
    get_cargo_metadata()
      .with_context(|| "failed to get cargo metadata")?
      .workspace_root,
  )
}

pub fn get_profile(options: &Options) -> String {
  options
    .args
    .iter()
    .position(|a| a == "--profile")
    .map(|i| options.args[i + 1].clone())
    .unwrap_or_else(|| if options.debug { "debug" } else { "release" }.into())
}

#[allow(unused_variables)]
fn tauri_config_to_bundle_settings(
  settings: &RustAppSettings,
  features: &[String],
  identifier: String,
  config: crate::helpers::config::BundleConfig,
  updater_config: Option<UpdaterSettings>,
  arch64bits: bool,
) -> crate::Result<BundleSettings> {
  let enabled_features = settings
    .manifest
    .lock()
    .unwrap()
    .all_enabled_features(features);

  #[cfg(windows)]
  let windows_icon_path = PathBuf::from(
    config
      .icon
      .iter()
      .find(|i| i.ends_with(".ico"))
      .cloned()
      .expect("the bundle config must have a `.ico` icon"),
  );
  #[cfg(not(windows))]
  let windows_icon_path = PathBuf::from("");

  #[allow(unused_mut)]
  let mut resources = config
    .resources
    .unwrap_or(BundleResources::List(Vec::new()));
  #[allow(unused_mut)]
  let mut depends_deb = config.linux.deb.depends.unwrap_or_default();
  #[allow(unused_mut)]
  let mut depends_rpm = config.linux.rpm.depends.unwrap_or_default();

  // set env vars used by the bundler and inject dependencies
  #[cfg(target_os = "linux")]
  {
    let mut libs: Vec<String> = Vec::new();

    if enabled_features.contains(&"tray-icon".into())
      || enabled_features.contains(&"tauri/tray-icon".into())
    {
      let (tray_kind, path) = std::env::var("TAURI_LINUX_AYATANA_APPINDICATOR")
        .map(|ayatana| {
          if ayatana == "true" || ayatana == "1" {
            (
              pkgconfig_utils::TrayKind::Ayatana,
              format!(
                "{}/libayatana-appindicator3.so.1",
                pkgconfig_utils::get_library_path("ayatana-appindicator3-0.1")
                  .expect("failed to get ayatana-appindicator library path using pkg-config.")
              ),
            )
          } else {
            (
              pkgconfig_utils::TrayKind::Libappindicator,
              format!(
                "{}/libappindicator3.so.1",
                pkgconfig_utils::get_library_path("appindicator3-0.1")
                  .expect("failed to get libappindicator-gtk library path using pkg-config.")
              ),
            )
          }
        })
        .unwrap_or_else(|_| pkgconfig_utils::get_appindicator_library_path());
      match tray_kind {
        pkgconfig_utils::TrayKind::Ayatana => {
          depends_deb.push("libayatana-appindicator3-1".into());
        }
        pkgconfig_utils::TrayKind::Libappindicator => {
          depends_deb.push("libappindicator3-1".into());
          libs.push("libappindicator3.so.1".into());
        }
      }

      std::env::set_var("TAURI_TRAY_LIBRARY_PATH", path);
    }

    depends_deb.push("libwebkit2gtk-4.1-0".to_string());
    depends_deb.push("libgtk-3-0".to_string());

    libs.push("libwebkit2gtk-4.1.so.0".into());
    libs.push("libgtk-3.so.0".into());

    for lib in libs {
      let mut requires = lib;
      if arch64bits {
        requires.push_str("()(64bit)");
      }
      depends_rpm.push(requires);
    }
  }

  #[cfg(windows)]
  {
    if let Some(webview_fixed_runtime_path) = &config.windows.webview_fixed_runtime_path {
      resources.push(webview_fixed_runtime_path.display().to_string());
    } else if let crate::helpers::config::WebviewInstallMode::FixedRuntime { path } =
      &config.windows.webview_install_mode
    {
      resources.push(path.display().to_string());
    }
  }

  let signing_identity = match std::env::var_os("APPLE_SIGNING_IDENTITY") {
    Some(signing_identity) => Some(
      signing_identity
        .to_str()
        .expect("failed to convert APPLE_SIGNING_IDENTITY to string")
        .to_string(),
    ),
    None => config.macos.signing_identity,
  };

  let provider_short_name = match std::env::var_os("APPLE_PROVIDER_SHORT_NAME") {
    Some(provider_short_name) => Some(
      provider_short_name
        .to_str()
        .expect("failed to convert APPLE_PROVIDER_SHORT_NAME to string")
        .to_string(),
    ),
    None => config.macos.provider_short_name,
  };

  let (resources, resources_map) = match resources {
    BundleResources::List(paths) => (Some(paths), None),
    BundleResources::Map(map) => (None, Some(map)),
  };

  Ok(BundleSettings {
    identifier: Some(identifier),
    publisher: config.publisher,
    icon: Some(config.icon),
    resources,
    resources_map,
    copyright: config.copyright,
    category: match config.category {
      Some(category) => Some(AppCategory::from_str(&category).map_err(|e| match e {
        Some(e) => anyhow::anyhow!("invalid category, did you mean `{}`?", e),
        None => anyhow::anyhow!("invalid category"),
      })?),
      None => None,
    },
    file_associations: config.file_associations,
    short_description: config.short_description,
    long_description: config.long_description,
    external_bin: config.external_bin,
    deb: DebianSettings {
      depends: if depends_deb.is_empty() {
        None
      } else {
        Some(depends_deb)
      },
      files: config.linux.deb.files,
      desktop_template: config.linux.deb.desktop_template,
    },
    appimage: AppImageSettings {
      files: config.linux.appimage.files,
    },
    rpm: RpmSettings {
      depends: if depends_rpm.is_empty() {
        None
      } else {
        Some(depends_rpm)
      },
      release: config.linux.rpm.release,
      epoch: config.linux.rpm.epoch,
      files: config.linux.rpm.files,
      desktop_template: config.linux.rpm.desktop_template,
    },
    dmg: DmgSettings {
      background: config.macos.dmg.background,
      window_position: config
        .macos
        .dmg
        .window_position
        .map(|window_position| Position {
          x: window_position.x,
          y: window_position.y,
        }),
      window_size: Size {
        width: config.macos.dmg.window_size.width,
        height: config.macos.dmg.window_size.height,
      },
      app_position: Position {
        x: config.macos.dmg.app_position.x,
        y: config.macos.dmg.app_position.y,
      },
      application_folder_position: Position {
        x: config.macos.dmg.application_folder_position.x,
        y: config.macos.dmg.application_folder_position.y,
      },
    },
    macos: MacOsSettings {
      frameworks: config.macos.frameworks,
      files: config.macos.files,
      minimum_system_version: config.macos.minimum_system_version,
      exception_domain: config.macos.exception_domain,
      signing_identity,
      provider_short_name,
      entitlements: config.macos.entitlements,
      info_plist_path: {
        let path = tauri_dir().join("Info.plist");
        if path.exists() {
          Some(path)
        } else {
          None
        }
      },
    },
    windows: WindowsSettings {
      timestamp_url: config.windows.timestamp_url,
      tsp: config.windows.tsp,
      digest_algorithm: config.windows.digest_algorithm,
      certificate_thumbprint: config.windows.certificate_thumbprint,
      wix: config.windows.wix.map(wix_settings),
      nsis: config.windows.nsis.map(nsis_settings),
      icon_path: windows_icon_path,
      webview_install_mode: config.windows.webview_install_mode,
      webview_fixed_runtime_path: config.windows.webview_fixed_runtime_path,
      allow_downgrades: config.windows.allow_downgrades,
    },
    license: config.license.or_else(|| {
      settings
        .cargo_package_settings
        .license
        .clone()
        .map(|license| {
          license
            .resolve("license", || {
              settings
                .cargo_ws_package_settings
                .as_ref()
                .and_then(|v| v.license.clone())
                .ok_or_else(|| {
                  anyhow::anyhow!("Couldn't inherit value for `license` from workspace")
                })
            })
            .unwrap()
        })
    }),
    license_file: config.license_file.map(|l| tauri_dir().join(l)),
    updater: updater_config,
    ..Default::default()
  })
}

#[cfg(target_os = "linux")]
mod pkgconfig_utils {
  use std::process::Command;

  pub enum TrayKind {
    Ayatana,
    Libappindicator,
  }

  pub fn get_appindicator_library_path() -> (TrayKind, String) {
    match get_library_path("ayatana-appindicator3-0.1") {
      Some(p) => (
        TrayKind::Ayatana,
        format!("{p}/libayatana-appindicator3.so.1"),
      ),
      None => match get_library_path("appindicator3-0.1") {
        Some(p) => (
          TrayKind::Libappindicator,
          format!("{p}/libappindicator3.so.1"),
        ),
        None => panic!("Can't detect any appindicator library"),
      },
    }
  }

  /// Gets the folder in which a library is located using `pkg-config`.
  pub fn get_library_path(name: &str) -> Option<String> {
    let mut cmd = Command::new("pkg-config");
    cmd.env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1");
    cmd.arg("--libs-only-L");
    cmd.arg(name);
    if let Ok(output) = cmd.output() {
      if !output.stdout.is_empty() {
        // output would be "-L/path/to/library\n"
        let word = output.stdout[2..].to_vec();
        return Some(String::from_utf8_lossy(&word).trim().to_string());
      } else {
        None
      }
    } else {
      None
    }
  }
}
