// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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
use glob::glob;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use serde::{Deserialize, Deserializer};
use tauri_bundler::{
  AppCategory, AppImageSettings, BundleBinary, BundleSettings, DebianSettings, DmgSettings,
  MacOsSettings, PackageSettings, Position, RpmSettings, Size, UpdaterSettings, WindowsSettings,
};
use tauri_utils::config::{parse::is_configuration_file, DeepLinkProtocol, Updater};

use super::{AppSettings, DevProcess, ExitReason, Interface};
use crate::{
  helpers::{
    app_paths::{frontend_dir, tauri_dir},
    config::{nsis_settings, reload as reload_config, wix_settings, BundleResources, Config},
  },
  ConfigValue,
};
use tauri_utils::{display_path, platform::Target};

mod cargo_config;
mod desktop;
pub mod installation;
pub mod manifest;
use crate::helpers::config::custom_sign_settings;
use cargo_config::Config as CargoConfig;
use manifest::{rewrite_manifest, Manifest};

#[derive(Debug, Default, Clone)]
pub struct Options {
  pub runner: Option<String>,
  pub debug: bool,
  pub target: Option<String>,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub config: Option<ConfigValue>,
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

impl From<crate::bundle::Options> for Options {
  fn from(options: crate::bundle::Options) -> Self {
    Self {
      debug: options.debug,
      config: options.config,
      target: options.target,
      features: options.features,
      no_watch: true,
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
    }
  }
}

#[derive(Debug, Clone)]
pub struct MobileOptions {
  pub debug: bool,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub config: Option<ConfigValue>,
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
  available_targets: Option<Vec<RustupTarget>>,
  main_binary_name: Option<String>,
}

impl Interface for Rust {
  type AppSettings = RustAppSettings;

  fn new(config: &Config, target: Option<String>) -> crate::Result<Self> {
    let manifest = {
      let (tx, rx) = sync_channel(1);
      let mut watcher = new_debouncer(Duration::from_secs(1), None, move |r| {
        if let Ok(events) = r {
          let _ = tx.send(events);
        }
      })
      .unwrap();
      watcher.watch(tauri_dir().join("Cargo.toml"), RecursiveMode::Recursive)?;
      let (manifest, _modified) = rewrite_manifest(config)?;
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

    let target_ios = target
      .as_ref()
      .is_some_and(|target| target.ends_with("ios") || target.ends_with("ios-sim"));
    if target_ios {
      std::env::set_var(
        "IPHONEOS_DEPLOYMENT_TARGET",
        &config.bundle.ios.minimum_system_version,
      );
    } else if let Some(minimum_system_version) = &config.bundle.macos.minimum_system_version {
      std::env::set_var("MACOSX_DEPLOYMENT_TARGET", minimum_system_version);
    }

    let app_settings = RustAppSettings::new(config, manifest, target)?;

    Ok(Self {
      app_settings: Arc::new(app_settings),
      config_features: config.build.features.clone().unwrap_or_default(),
      main_binary_name: config.main_binary_name.clone(),
      available_targets: None,
    })
  }

  fn app_settings(&self) -> Arc<Self::AppSettings> {
    self.app_settings.clone()
  }

  fn build(&mut self, options: Options) -> crate::Result<PathBuf> {
    desktop::build(
      options,
      &self.app_settings,
      &mut self.available_targets,
      self.config_features.clone(),
      self.main_binary_name.as_deref(),
    )
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
      let config = options.config.clone().map(|c| c.0);
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
      let config = options.config.clone().map(|c| c.0);
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
      if path.starts_with(gitignore.path())
        && gitignore
          .matched_path_or_any_parents(path, is_dir)
          .is_ignore()
      {
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
        .map_while(Result::ok)
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

  if mobile {
    args.push("--lib".into());
  }

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

// Copied from https://github.com/rust-lang/cargo/blob/69255bb10de7f74511b5cef900a9d102247b6029/src/cargo/core/workspace.rs#L665
fn expand_member_path(path: &Path) -> crate::Result<Vec<PathBuf>> {
  let Some(path) = path.to_str() else {
    return Err(anyhow::anyhow!("path is not UTF-8 compatible"));
  };
  let res = glob(path).with_context(|| format!("could not parse pattern `{}`", &path))?;
  let res = res
    .map(|p| p.with_context(|| format!("unable to match path to pattern `{}`", &path)))
    .collect::<Result<Vec<_>, _>>()?;
  Ok(res)
}

fn get_watch_folders() -> crate::Result<Vec<PathBuf>> {
  let tauri_path = tauri_dir();
  let workspace_path = get_workspace_dir()?;

  // We always want to watch the main tauri folder.
  let mut watch_folders = vec![tauri_path.to_path_buf()];

  // We also try to watch workspace members, no matter if the tauri cargo project is the workspace root or a workspace member
  let cargo_settings = CargoSettings::load(&workspace_path)?;
  if let Some(members) = cargo_settings.workspace.and_then(|w| w.members) {
    for p in members {
      let p = workspace_path.join(p);
      match expand_member_path(&p) {
        // Sometimes expand_member_path returns an empty vec, for example if the path contains `[]` as in `C:/[abc]/project/`.
        // Cargo won't complain unless theres a workspace.members config with glob patterns so we should support it too.
        Ok(expanded_paths) => {
          if expanded_paths.is_empty() {
            watch_folders.push(p);
          } else {
            watch_folders.extend(expanded_paths);
          }
        }
        Err(err) => {
          // If this fails cargo itself should fail too. But we still try to keep going with the unexpanded path.
          log::error!("Error watching {}: {}", p.display(), err.to_string());
          watch_folders.push(p);
        }
      };
    }
  }

  Ok(watch_folders)
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
      .push("tauri/custom-protocol".into());
    if mobile {
      args.push("--lib".into());
    } else {
      args.push("--bins".into());
    }
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
      on_exit,
    )
    .map(|c| Box::new(c) as Box<dyn DevProcess + Send>)
  }

  fn run_dev_watcher<F: Fn(&mut Rust) -> crate::Result<Box<dyn DevProcess + Send>>>(
    &mut self,
    config: Option<serde_json::Value>,
    run: Arc<F>,
  ) -> crate::Result<()> {
    let child = run(self)?;

    let process = Arc::new(Mutex::new(child));
    let (tx, rx) = sync_channel(1);
    let frontend_path = frontend_dir();

    let watch_folders = get_watch_folders()?;

    let common_ancestor = common_path::common_path_all(watch_folders.iter().map(Path::new))
      .expect("watch_folders should not be empty");
    let ignore_matcher = build_ignore_matcher(&common_ancestor);

    let mut watcher = new_debouncer(Duration::from_secs(1), None, move |r| {
      if let Ok(events) = r {
        tx.send(events).unwrap()
      }
    })
    .unwrap();
    for path in watch_folders {
      if !ignore_matcher.is_ignore(&path, true) {
        log::info!("Watching {} for changes...", display_path(&path));
        lookup(&path, |file_type, p| {
          if p != path {
            log::debug!("Watching {} for changes...", display_path(&p));
            let _ = watcher.watch(
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
          if event.kind.is_access() {
            continue;
          }

          if let Some(event_path) = event.paths.first() {
            if !ignore_matcher.is_ignore(event_path, event_path.is_dir()) {
              if is_configuration_file(self.app_settings.target, event_path) {
                if let Ok(config) = reload_config(config.as_ref()) {
                  let (manifest, modified) =
                    rewrite_manifest(config.lock().unwrap().as_ref().unwrap())?;
                  if modified {
                    *self.app_settings.manifest.lock().unwrap() = manifest;
                    // no need to run the watcher logic, the manifest was modified
                    // and it will trigger the watcher again
                    continue;
                  }
                }
              }

              log::info!(
                "File {} changed. Rebuilding application...",
                display_path(event_path.strip_prefix(frontend_path).unwrap_or(event_path))
              );

              let mut p = process.lock().unwrap();
              p.kill().with_context(|| "failed to kill app process")?;

              // wait for the process to exit
              // note that on mobile, kill() already waits for the process to exit (duct implementation)
              loop {
                if !matches!(p.try_wait(), Ok(None)) {
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
  pub name: String,
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
#[serde(rename_all = "camelCase")]
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

    let updater_enabled = config.bundle.create_updater_artifacts != Updater::Bool(false);
    let v1_compatible = matches!(config.bundle.create_updater_artifacts, Updater::String(_));
    let updater_settings = if updater_enabled {
      let updater: UpdaterConfig = serde_json::from_value(
        config
          .plugins
          .0
          .get("updater")
          .ok_or_else(|| {
            anyhow::anyhow!("failed to get updater configuration: plugins > updater doesn't exist")
          })?
          .clone(),
      )?;
      Some(UpdaterSettings {
        v1_compatible,
        pubkey: updater.pubkey,
        msiexec_args: updater.windows.install_mode.msiexec_args(),
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

    if let Some(open) = config.plugins.0.get("shell").and_then(|v| v.get("open")) {
      if open.as_bool().is_some_and(|x| x) || open.is_string() {
        settings.appimage.bundle_xdg_open = true;
      }
    }

    if let Some(deps) = self
      .manifest
      .lock()
      .unwrap()
      .inner
      .as_table()
      .get("dependencies")
      .and_then(|f| f.as_table())
    {
      if deps.contains_key("tauri-plugin-opener") {
        settings.appimage.bundle_xdg_open = true;
      };
    }

    Ok(settings)
  }

  fn app_binary_path(&self, options: &Options) -> crate::Result<PathBuf> {
    let binaries = self.get_binaries()?;
    let bin_name = binaries
      .iter()
      .find(|x| x.main())
      .context("failed to find main binary, make sure you have a `package > default-run` in the Cargo.toml file")?
      .name();

    let out_dir = self
      .out_dir(options)
      .context("failed to get project out directory")?;

    let ext = if self.target_triple.contains("windows") {
      "exe"
    } else {
      ""
    };

    Ok(out_dir.join(bin_name).with_extension(ext))
  }

  fn get_binaries(&self) -> crate::Result<Vec<BundleBinary>> {
    let mut binaries: Vec<BundleBinary> = vec![];

    if let Some(bins) = &self.cargo_settings.bin {
      let default_run = self
        .package_settings
        .default_run
        .clone()
        .unwrap_or_default();
      for bin in bins {
        let is_main = bin.name == self.cargo_package_settings.name || bin.name == default_run;
        binaries.push(BundleBinary::with_path(
          bin.name.clone(),
          is_main,
          bin.path.clone(),
        ))
      }
    }

    let tauri_dir = tauri_dir();

    let mut binaries_paths = std::fs::read_dir(tauri_dir.join("src/bin"))
      .map(|dir| {
        dir
          .into_iter()
          .flatten()
          .map(|entry| {
            (
              entry
                .path()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
              entry.path(),
            )
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    if !binaries_paths
      .iter()
      .any(|(_name, path)| path == Path::new("src/main.rs"))
      && tauri_dir.join("src/main.rs").exists()
    {
      binaries_paths.push((
        self.cargo_package_settings.name.clone(),
        tauri_dir.join("src/main.rs"),
      ));
    }

    for (name, path) in binaries_paths {
      // see https://github.com/tauri-apps/tauri/pull/10977#discussion_r1759742414
      let bin_exists = binaries
        .iter()
        .any(|bin| bin.name() == name || path.ends_with(bin.src_path().unwrap_or(&"".to_string())));
      if !bin_exists {
        binaries.push(BundleBinary::new(name, false))
      }
    }

    if let Some(default_run) = self.package_settings.default_run.as_ref() {
      if let Some(binary) = binaries.iter_mut().find(|bin| bin.name() == default_run) {
        binary.set_main(true);
      } else {
        binaries.push(BundleBinary::new(default_run.clone(), true));
      }
    }

    match binaries.len() {
      0 => binaries.push(BundleBinary::new(
        self.cargo_package_settings.name.clone(),
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
    let tauri_dir = tauri_dir();
    let cargo_settings =
      CargoSettings::load(tauri_dir).with_context(|| "failed to load cargo settings")?;
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
      product_name: config
        .product_name
        .clone()
        .unwrap_or_else(|| cargo_package_settings.name.clone()),
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

    let cargo_config = CargoConfig::load(tauri_dir)?;

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

  fn target<'a>(&'a self, options: &'a Options) -> Option<&'a str> {
    options
      .target
      .as_deref()
      .or_else(|| self.cargo_config.build().target())
  }

  pub fn out_dir(&self, options: &Options) -> crate::Result<PathBuf> {
    get_target_dir(self.target(options), options)
  }
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadata {
  pub(crate) target_directory: PathBuf,
  pub(crate) workspace_root: PathBuf,
}

pub(crate) fn get_cargo_metadata() -> crate::Result<CargoMetadata> {
  let output = Command::new("cargo")
    .args(["metadata", "--no-deps", "--format-version", "1"])
    .current_dir(tauri_dir())
    .output()?;

  if !output.status.success() {
    return Err(anyhow::anyhow!(
      "cargo metadata command exited with a non zero exit code: {}",
      String::from_utf8_lossy(&output.stderr)
    ));
  }

  Ok(serde_json::from_slice(&output.stdout)?)
}

/// This function determines the 'target' directory and suffixes it with the profile
/// to determine where the compiled binary will be located.
fn get_target_dir(triple: Option<&str>, options: &Options) -> crate::Result<PathBuf> {
  let mut path = if let Some(target) = get_cargo_option(&options.args, "--target-dir") {
    std::env::current_dir()?.join(target)
  } else {
    let mut path = get_cargo_metadata()
      .with_context(|| "failed to get cargo metadata")?
      .target_directory;

    if let Some(triple) = triple {
      path.push(triple);
    }

    path
  };

  path.push(get_profile_dir(options));

  Ok(path)
}

#[inline]
fn get_cargo_option<'a>(args: &'a [String], option: &'a str) -> Option<&'a str> {
  args
    .iter()
    .position(|a| a.starts_with(option))
    .and_then(|i| {
      args[i]
        .split_once('=')
        .map(|(_, p)| Some(p))
        .unwrap_or_else(|| args.get(i + 1).map(|s| s.as_str()))
    })
}

/// Executes `cargo metadata` to get the workspace directory.
pub fn get_workspace_dir() -> crate::Result<PathBuf> {
  Ok(
    get_cargo_metadata()
      .with_context(|| "failed to get cargo metadata")?
      .workspace_root,
  )
}

pub fn get_profile(options: &Options) -> &str {
  get_cargo_option(&options.args, "--profile").unwrap_or(if options.debug {
    "dev"
  } else {
    "release"
  })
}

pub fn get_profile_dir(options: &Options) -> &str {
  match get_profile(options) {
    "dev" => "debug",
    profile => profile,
  }
}

#[allow(unused_variables, deprecated)]
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

  #[allow(unused_mut)]
  let mut resources = config
    .resources
    .unwrap_or(BundleResources::List(Vec::new()));
  #[allow(unused_mut)]
  let mut depends_deb = config.linux.deb.depends.unwrap_or_default();

  #[allow(unused_mut)]
  let mut depends_rpm = config.linux.rpm.depends.unwrap_or_default();

  #[allow(unused_mut)]
  let mut appimage_files = config.linux.appimage.files;

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
          libs.push("libayatana-appindicator3.so.1".into());
        }
        pkgconfig_utils::TrayKind::Libappindicator => {
          depends_deb.push("libappindicator3-1".into());
          libs.push("libappindicator3.so.1".into());
        }
      }

      // conditionally setting it in case the user provided its own version for some reason
      let path = PathBuf::from(path);
      if !appimage_files.contains_key(&path) {
        // manually construct target path, just in case the source path is something unexpected
        appimage_files.insert(Path::new("/usr/lib/").join(path.file_name().unwrap()), path);
      }
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
    if let crate::helpers::config::WebviewInstallMode::FixedRuntime { path } =
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
    homepage: config.homepage,
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
      recommends: config.linux.deb.recommends,
      provides: config.linux.deb.provides,
      conflicts: config.linux.deb.conflicts,
      replaces: config.linux.deb.replaces,
      files: config.linux.deb.files,
      desktop_template: config.linux.deb.desktop_template,
      section: config.linux.deb.section,
      priority: config.linux.deb.priority,
      changelog: config.linux.deb.changelog,
      pre_install_script: config.linux.deb.pre_install_script,
      post_install_script: config.linux.deb.post_install_script,
      pre_remove_script: config.linux.deb.pre_remove_script,
      post_remove_script: config.linux.deb.post_remove_script,
    },
    appimage: AppImageSettings {
      files: appimage_files,
      bundle_media_framework: config.linux.appimage.bundle_media_framework,
      bundle_xdg_open: false,
    },
    rpm: RpmSettings {
      depends: if depends_rpm.is_empty() {
        None
      } else {
        Some(depends_rpm)
      },
      recommends: config.linux.rpm.recommends,
      provides: config.linux.rpm.provides,
      conflicts: config.linux.rpm.conflicts,
      obsoletes: config.linux.rpm.obsoletes,
      release: config.linux.rpm.release,
      epoch: config.linux.rpm.epoch,
      files: config.linux.rpm.files,
      desktop_template: config.linux.rpm.desktop_template,
      pre_install_script: config.linux.rpm.pre_install_script,
      post_install_script: config.linux.rpm.post_install_script,
      pre_remove_script: config.linux.rpm.pre_remove_script,
      post_remove_script: config.linux.rpm.post_remove_script,
      compression: config.linux.rpm.compression,
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
      hardened_runtime: config.macos.hardened_runtime,
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
      icon_path: PathBuf::new(),
      webview_install_mode: config.windows.webview_install_mode,
      allow_downgrades: config.windows.allow_downgrades,
      sign_command: config.windows.sign_command.map(custom_sign_settings),
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
        Some(String::from_utf8_lossy(&word).trim().to_string())
      } else {
        None
      }
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_cargo_option() {
    let args = vec![
      "build".into(),
      "--".into(),
      "--profile".into(),
      "holla".into(),
      "--features".into(),
      "a".into(),
      "b".into(),
      "--target-dir".into(),
      "path/to/dir".into(),
    ];

    assert_eq!(get_cargo_option(&args, "--profile"), Some("holla"));
    assert_eq!(get_cargo_option(&args, "--target-dir"), Some("path/to/dir"));
    assert_eq!(get_cargo_option(&args, "--non-existent"), None);
  }

  #[test]
  fn parse_profile_from_opts() {
    let options = Options {
      args: vec![
        "build".into(),
        "--".into(),
        "--profile".into(),
        "testing".into(),
        "--features".into(),
        "feat1".into(),
      ],
      ..Default::default()
    };
    assert_eq!(get_profile(&options), "testing");

    let options = Options {
      args: vec![
        "build".into(),
        "--".into(),
        "--profile=customprofile".into(),
        "testing".into(),
        "--features".into(),
        "feat1".into(),
      ],
      ..Default::default()
    };
    assert_eq!(get_profile(&options), "customprofile");

    let options = Options {
      debug: true,
      args: vec![
        "build".into(),
        "--".into(),
        "testing".into(),
        "--features".into(),
        "feat1".into(),
      ],
      ..Default::default()
    };
    assert_eq!(get_profile(&options), "dev");

    let options = Options {
      debug: false,
      args: vec![
        "build".into(),
        "--".into(),
        "testing".into(),
        "--features".into(),
        "feat1".into(),
      ],
      ..Default::default()
    };
    assert_eq!(get_profile(&options), "release");

    let options = Options {
      args: vec!["build".into(), "--".into(), "--profile".into()],
      ..Default::default()
    };
    assert_eq!(get_profile(&options), "release");
  }

  #[test]
  fn parse_target_dir_from_opts() {
    crate::helpers::app_paths::resolve();
    let current_dir = std::env::current_dir().unwrap();

    let options = Options {
      args: vec![
        "build".into(),
        "--".into(),
        "--target-dir".into(),
        "path/to/some/dir".into(),
        "--features".into(),
        "feat1".into(),
      ],
      debug: false,
      ..Default::default()
    };

    assert_eq!(
      get_target_dir(None, &options).unwrap(),
      current_dir.join("path/to/some/dir/release")
    );
    assert_eq!(
      get_target_dir(Some("x86_64-pc-windows-msvc"), &options).unwrap(),
      current_dir.join("path/to/some/dir/release")
    );

    let options = Options {
      args: vec![
        "build".into(),
        "--".into(),
        "--features".into(),
        "feat1".into(),
      ],
      debug: false,
      ..Default::default()
    };

    #[cfg(windows)]
    assert!(get_target_dir(Some("x86_64-pc-windows-msvc"), &options)
      .unwrap()
      .ends_with("x86_64-pc-windows-msvc\\release"));
    #[cfg(not(windows))]
    assert!(get_target_dir(Some("x86_64-pc-windows-msvc"), &options)
      .unwrap()
      .ends_with("x86_64-pc-windows-msvc/release"));

    #[cfg(windows)]
    {
      std::env::set_var("CARGO_TARGET_DIR", "D:\\path\\to\\env\\dir");
      assert_eq!(
        get_target_dir(None, &options).unwrap(),
        PathBuf::from("D:\\path\\to\\env\\dir\\release")
      );
      assert_eq!(
        get_target_dir(Some("x86_64-pc-windows-msvc"), &options).unwrap(),
        PathBuf::from("D:\\path\\to\\env\\dir\\x86_64-pc-windows-msvc\\release")
      );
    }

    #[cfg(not(windows))]
    {
      std::env::set_var("CARGO_TARGET_DIR", "/path/to/env/dir");
      assert_eq!(
        get_target_dir(None, &options).unwrap(),
        PathBuf::from("/path/to/env/dir/release")
      );
      assert_eq!(
        get_target_dir(Some("x86_64-pc-windows-msvc"), &options).unwrap(),
        PathBuf::from("/path/to/env/dir/x86_64-pc-windows-msvc/release")
      );
    }
  }
}
