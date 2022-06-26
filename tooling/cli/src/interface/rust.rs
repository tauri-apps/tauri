// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::{rename, File},
  io::{BufReader, ErrorKind, Read, Write},
  path::{Path, PathBuf},
  process::{Command, ExitStatus, Stdio},
  str::FromStr,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use anyhow::Context;
#[cfg(target_os = "linux")]
use heck::ToKebabCase;
use log::warn;
use serde::Deserialize;
use shared_child::SharedChild;
use tauri_bundler::{
  AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings,
  UpdaterSettings, WindowsSettings,
};

use super::{AppSettings, DevProcess, ExitReason, Interface};
use crate::{
  helpers::{
    app_paths::tauri_dir,
    config::{wix_settings, Config},
    manifest::Manifest,
  },
  CommandExt,
};

#[derive(Debug, Clone)]
pub struct Options {
  pub runner: Option<String>,
  pub debug: bool,
  pub target: Option<String>,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
}

impl From<crate::build::Options> for Options {
  fn from(options: crate::build::Options) -> Self {
    Self {
      runner: options.runner,
      debug: options.debug,
      target: options.target,
      features: options.features,
      args: options.args,
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
    }
  }
}

pub struct DevChild {
  manually_killed_app: Arc<AtomicBool>,
  build_child: Arc<SharedChild>,
  app_child: Arc<Mutex<Option<Arc<SharedChild>>>>,
}

impl DevProcess for DevChild {
  fn kill(&self) -> std::io::Result<()> {
    if let Some(child) = &*self.app_child.lock().unwrap() {
      child.kill()?;
    } else {
      self.build_child.kill()?;
    }
    self.manually_killed_app.store(true, Ordering::Relaxed);
    Ok(())
  }

  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
    if let Some(child) = &*self.app_child.lock().unwrap() {
      child.try_wait()
    } else {
      self.build_child.try_wait()
    }
  }
}

#[derive(Debug)]
struct Target {
  name: String,
  installed: bool,
}

pub struct Rust {
  app_settings: RustAppSettings,
  config_features: Vec<String>,
  product_name: Option<String>,
  available_targets: Option<Vec<Target>>,
}

impl Interface for Rust {
  type AppSettings = RustAppSettings;
  type Dev = DevChild;

  fn new(config: &Config) -> crate::Result<Self> {
    Ok(Self {
      app_settings: RustAppSettings::new(config)?,
      config_features: config.build.features.clone().unwrap_or_default(),
      product_name: config.package.product_name.clone(),
      available_targets: None,
    })
  }

  fn app_settings(&self) -> &Self::AppSettings {
    &self.app_settings
  }

  fn build(&mut self, options: Options) -> crate::Result<()> {
    let bin_path = self.app_settings.app_binary_path(&options)?;
    let out_dir = bin_path.parent().unwrap();

    let bin_name = bin_path.file_stem().unwrap();

    if options.target == Some("universal-apple-darwin".into()) {
      std::fs::create_dir_all(&out_dir)
        .with_context(|| "failed to create project out directory")?;

      let mut lipo_cmd = Command::new("lipo");
      lipo_cmd
        .arg("-create")
        .arg("-output")
        .arg(out_dir.join(&bin_name));
      for triple in ["aarch64-apple-darwin", "x86_64-apple-darwin"] {
        let mut options = options.clone();
        options.target.replace(triple.into());

        let triple_out_dir = self
          .app_settings
          .out_dir(Some(triple.into()), options.debug)
          .with_context(|| format!("failed to get {} out dir", triple))?;
        self
          .build_app(options)
          .with_context(|| format!("failed to build {} binary", triple))?;

        lipo_cmd.arg(triple_out_dir.join(&bin_name));
      }

      let lipo_status = lipo_cmd.output_ok()?.status;
      if !lipo_status.success() {
        return Err(anyhow::anyhow!(format!(
          "Result of `lipo` command was unsuccessful: {}. (Is `lipo` installed?)",
          lipo_status
        )));
      }
    } else {
      self
        .build_app(options)
        .with_context(|| "failed to build app")?;
    }

    rename_app(bin_path, self.product_name.as_deref())?;

    Ok(())
  }

  fn dev<F: FnOnce(ExitStatus, ExitReason) + Send + 'static>(
    &mut self,
    options: Options,
    manifest: &Manifest,
    on_exit: F,
  ) -> crate::Result<Self::Dev> {
    let bin_path = self.app_settings.app_binary_path(&options)?;
    let product_name = self.product_name.clone();

    let runner = options.runner.unwrap_or_else(|| "cargo".into());

    if let Some(target) = &options.target {
      self.fetch_available_targets();
      self.validate_target(target)?;
    }

    let mut build_cmd = Command::new(&runner);
    build_cmd
      .env(
        "CARGO_TERM_PROGRESS_WIDTH",
        terminal::stderr_width()
          .map(|width| {
            if cfg!(windows) {
              std::cmp::min(60, width)
            } else {
              width
            }
          })
          .unwrap_or(if cfg!(windows) { 60 } else { 80 })
          .to_string(),
      )
      .env("CARGO_TERM_PROGRESS_WHEN", "always");
    build_cmd.arg("build").arg("--color").arg("always");

    if !options.args.contains(&"--no-default-features".into()) {
      let manifest_features = manifest.features();
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
      build_cmd.arg("--no-default-features");
      if !enable_features.is_empty() {
        build_cmd.args(&["--features", &enable_features.join(",")]);
      }
    }

    if !options.debug {
      build_cmd.args(&["--release"]);
    }

    if let Some(target) = &options.target {
      build_cmd.args(&["--target", target]);
    }

    let mut features = self.config_features.clone();
    if let Some(f) = options.features {
      features.extend(f);
    }
    if !features.is_empty() {
      build_cmd.args(&["--features", &features.join(",")]);
    }

    let mut run_args = Vec::new();
    let mut reached_run_args = false;
    for arg in options.args.clone() {
      if reached_run_args {
        run_args.push(arg);
      } else if arg == "--" {
        reached_run_args = true;
      } else {
        build_cmd.arg(arg);
      }
    }

    build_cmd.stdout(os_pipe::dup_stdout()?);
    build_cmd.stderr(Stdio::piped());

    let manually_killed_app = Arc::new(AtomicBool::default());
    let manually_killed_app_ = manually_killed_app.clone();

    let build_child = match SharedChild::spawn(&mut build_cmd) {
      Ok(c) => c,
      Err(e) => {
        if e.kind() == ErrorKind::NotFound {
          return Err(anyhow::anyhow!(
            "`{}` command not found.{}",
            runner,
            if runner == "cargo" {
              " Please follow the Tauri setup guide: https://tauri.app/v1/guides/getting-started/prerequisites"
            } else {
              ""
            }
          ));
        } else {
          return Err(e.into());
        }
      }
    };
    let build_child = Arc::new(build_child);
    let build_child_stderr = build_child.take_stderr().unwrap();
    let mut stderr = BufReader::new(build_child_stderr);
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines_ = stderr_lines.clone();
    std::thread::spawn(move || {
      let mut buf = Vec::new();
      let mut lines = stderr_lines_.lock().unwrap();
      let mut io_stderr = std::io::stderr();
      loop {
        buf.clear();
        match tauri_utils::io::read_line(&mut stderr, &mut buf) {
          Ok(s) if s == 0 => break,
          _ => (),
        }
        let _ = io_stderr.write_all(&buf);
        if !buf.ends_with(&[b'\r']) {
          let _ = io_stderr.write_all(b"\n");
        }
        lines.push(String::from_utf8_lossy(&buf).into_owned());
      }
    });

    let build_child_ = build_child.clone();
    let app_child = Arc::new(Mutex::new(None));
    let app_child_ = app_child.clone();
    std::thread::spawn(move || {
      let status = build_child_.wait().expect("failed to wait on build");

      if status.success() {
        let bin_path = rename_app(bin_path, product_name.as_deref()).expect("failed to rename app");

        let mut app = Command::new(bin_path);
        app.stdout(os_pipe::dup_stdout().unwrap());
        app.stderr(os_pipe::dup_stderr().unwrap());
        app.args(run_args);
        let app_child = Arc::new(SharedChild::spawn(&mut app).unwrap());
        let app_child_t = app_child.clone();
        std::thread::spawn(move || {
          let status = app_child_t.wait().expect("failed to wait on app");
          on_exit(
            status,
            if manually_killed_app_.load(Ordering::Relaxed) {
              ExitReason::TriggeredKill
            } else {
              ExitReason::NormalExit
            },
          );
        });

        app_child_.lock().unwrap().replace(app_child);
      } else {
        let is_cargo_compile_error = stderr_lines
          .lock()
          .unwrap()
          .last()
          .map(|l| l.contains("could not compile"))
          .unwrap_or_default();
        stderr_lines.lock().unwrap().clear();

        on_exit(
          status,
          if status.code() == Some(101) && is_cargo_compile_error {
            ExitReason::CompilationFailed
          } else {
            ExitReason::NormalExit
          },
        );
      }
    });

    Ok(DevChild {
      manually_killed_app,
      build_child,
      app_child,
    })
  }
}

impl Rust {
  fn fetch_available_targets(&mut self) {
    if let Ok(output) = Command::new("rustup").args(["target", "list"]).output() {
      let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
      self.available_targets.replace(
        stdout
          .split('\n')
          .map(|t| {
            let mut s = t.split(' ');
            let name = s.next().unwrap().to_string();
            let installed = s.next().map(|v| v == "(installed)").unwrap_or_default();
            Target { name, installed }
          })
          .filter(|t| !t.name.is_empty())
          .collect(),
      );
    }
  }

  fn validate_target(&self, target: &str) -> crate::Result<()> {
    if let Some(available_targets) = &self.available_targets {
      if let Some(target) = available_targets.iter().find(|t| t.name == target) {
        if !target.installed {
          anyhow::bail!(
            "Target {target} is not installed (installed targets: {installed}). Please run `rustup target add {target}`.",
            target = target.name,
            installed = available_targets.iter().filter(|t| t.installed).map(|t| t.name.as_str()).collect::<Vec<&str>>().join(", ")
          );
        }
      }
      if !available_targets.iter().any(|t| t.name == target) {
        anyhow::bail!("Target {target} does not exist. Please run `rustup target list` to see the available targets.", target = target);
      }
    }
    Ok(())
  }

  fn build_app(&mut self, options: Options) -> crate::Result<()> {
    let runner = options.runner.unwrap_or_else(|| "cargo".into());

    if let Some(target) = &options.target {
      if self.available_targets.is_none() {
        self.fetch_available_targets();
      }
      self.validate_target(target)?;
    }

    let mut args = Vec::new();
    if !options.args.is_empty() {
      args.extend(options.args);
    }

    if let Some(features) = options.features {
      if !features.is_empty() {
        args.push("--features".into());
        args.push(features.join(","));
      }
    }

    if !options.debug {
      args.push("--release".into());
    }

    if let Some(target) = options.target {
      args.push("--target".into());
      args.push(target);
    }

    match Command::new(&runner)
      .args(&["build", "--features=custom-protocol"])
      .args(args)
      .env("STATIC_VCRUNTIME", "true")
      .piped()
    {
      Ok(status) => {
        if status.success() {
          Ok(())
        } else {
          Err(anyhow::anyhow!(
            "Result of `{} build` operation was unsuccessful",
            runner
          ))
        }
      }
      Err(e) => {
        if e.kind() == ErrorKind::NotFound {
          Err(anyhow::anyhow!(
            "`{}` command not found.{}",
            runner,
            if runner == "cargo" {
              " Please follow the Tauri setup guide: https://tauri.app/v1/guides/getting-started/prerequisites"
            } else {
              ""
            }
          ))
        } else {
          Err(e.into())
        }
      }
    }
  }
}

/// The `workspace` section of the app configuration (read from Cargo.toml).
#[derive(Clone, Debug, Deserialize)]
struct WorkspaceSettings {
  /// the workspace members.
  members: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct BinarySettings {
  name: String,
  path: Option<String>,
}

/// The package settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CargoPackageSettings {
  /// the package's name.
  pub name: Option<String>,
  /// the package's version.
  pub version: Option<String>,
  /// the package's description.
  pub description: Option<String>,
  /// the package's homepage.
  pub homepage: Option<String>,
  /// the package's authors.
  pub authors: Option<Vec<String>>,
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

#[derive(Deserialize)]
struct CargoBuildConfig {
  #[serde(rename = "target-dir")]
  target_dir: Option<String>,
}

#[derive(Deserialize)]
struct CargoConfig {
  build: Option<CargoBuildConfig>,
}

pub struct RustAppSettings {
  cargo_settings: CargoSettings,
  cargo_package_settings: CargoPackageSettings,
  package_settings: PackageSettings,
}

impl AppSettings for RustAppSettings {
  fn get_package_settings(&self) -> PackageSettings {
    self.package_settings.clone()
  }

  fn get_bundle_settings(
    &self,
    config: &Config,
    manifest: &Manifest,
    features: &[String],
  ) -> crate::Result<BundleSettings> {
    tauri_config_to_bundle_settings(
      manifest,
      features,
      config.tauri.bundle.clone(),
      config.tauri.system_tray.clone(),
      config.tauri.updater.clone(),
    )
  }

  fn app_binary_path(&self, options: &Options) -> crate::Result<PathBuf> {
    let bin_name = self
      .cargo_package_settings()
      .name
      .clone()
      .expect("Cargo manifest must have the `package.name` field");

    let out_dir = self
      .out_dir(options.target.clone(), options.debug)
      .with_context(|| "failed to get project out directory")?;
    let target: String = if let Some(target) = options.target.clone() {
      target
    } else {
      tauri_utils::platform::target_triple()?
    };

    let binary_extension: String = if target.contains("windows") {
      "exe"
    } else {
      ""
    }
    .into();

    Ok(out_dir.join(bin_name).with_extension(&binary_extension))
  }

  fn get_binaries(&self, config: &Config, target: &str) -> crate::Result<Vec<BundleBinary>> {
    let mut binaries: Vec<BundleBinary> = vec![];

    let binary_extension: String = if target.contains("windows") {
      ".exe"
    } else {
      ""
    }
    .into();

    if let Some(bin) = &self.cargo_settings.bin {
      let default_run = self
        .package_settings
        .default_run
        .clone()
        .unwrap_or_else(|| "".to_string());
      for binary in bin {
        binaries.push(
          if Some(&binary.name) == self.cargo_package_settings.name.as_ref()
            || binary.name.as_str() == default_run
          {
            BundleBinary::new(
              format!(
                "{}{}",
                config
                  .package
                  .binary_name()
                  .unwrap_or_else(|| binary.name.clone()),
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
          if let Some(bin_name) = config.package.binary_name() {
            bin.set_name(bin_name);
          }
        }
        None => {
          binaries.push(BundleBinary::new(
            format!(
              "{}{}",
              config
                .package
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
        #[cfg(target_os = "linux")]
        self.package_settings.product_name.to_kebab_case(),
        #[cfg(not(target_os = "linux"))]
        format!(
          "{}{}",
          self.package_settings.product_name.clone(),
          &binary_extension
        ),
        true,
      )),
      1 => binaries.get_mut(0).unwrap().set_main(true),
      _ => {}
    }

    Ok(binaries)
  }
}

impl RustAppSettings {
  pub fn new(config: &Config) -> crate::Result<Self> {
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

    let package_settings = PackageSettings {
      product_name: config.package.product_name.clone().unwrap_or_else(|| {
        cargo_package_settings
          .name
          .clone()
          .expect("Cargo manifest must have the `package.name` field")
      }),
      version: config.package.version.clone().unwrap_or_else(|| {
        cargo_package_settings
          .version
          .clone()
          .expect("Cargo manifest must have the `package.version` field")
      }),
      description: cargo_package_settings
        .description
        .clone()
        .unwrap_or_default(),
      homepage: cargo_package_settings.homepage.clone(),
      authors: cargo_package_settings.authors.clone(),
      default_run: cargo_package_settings.default_run.clone(),
    };

    Ok(Self {
      cargo_settings,
      cargo_package_settings,
      package_settings,
    })
  }

  pub fn cargo_package_settings(&self) -> &CargoPackageSettings {
    &self.cargo_package_settings
  }

  pub fn out_dir(&self, target: Option<String>, debug: bool) -> crate::Result<PathBuf> {
    let tauri_dir = tauri_dir();
    let workspace_dir = get_workspace_dir(&tauri_dir);
    get_target_dir(&workspace_dir, target, !debug)
  }
}

/// This function determines where 'target' dir is and suffixes it with 'release' or 'debug'
/// to determine where the compiled binary will be located.
fn get_target_dir(
  project_root_dir: &Path,
  target: Option<String>,
  is_release: bool,
) -> crate::Result<PathBuf> {
  let mut path: PathBuf = match std::env::var_os("CARGO_TARGET_DIR") {
    Some(target_dir) => target_dir.into(),
    None => {
      let mut root_dir = project_root_dir.to_path_buf();
      let target_path: Option<PathBuf> = loop {
        // cargo reads configs under .cargo/config.toml or .cargo/config
        let mut cargo_config_path = root_dir.join(".cargo/config");
        if !cargo_config_path.exists() {
          cargo_config_path = root_dir.join(".cargo/config.toml");
        }
        // if the path exists, parse it
        if cargo_config_path.exists() {
          let mut config_str = String::new();
          let mut config_file = File::open(&cargo_config_path)
            .with_context(|| format!("failed to open {:?}", cargo_config_path))?;
          config_file
            .read_to_string(&mut config_str)
            .with_context(|| "failed to read cargo config file")?;
          let config: CargoConfig =
            toml::from_str(&config_str).with_context(|| "failed to parse cargo config file")?;
          if let Some(build) = config.build {
            if let Some(target_dir) = build.target_dir {
              break Some(target_dir.into());
            }
          }
        }
        if !root_dir.pop() {
          break None;
        }
      };
      target_path.unwrap_or_else(|| project_root_dir.join("target"))
    }
  };

  if let Some(ref triple) = target {
    path.push(triple);
  }
  path.push(if is_release { "release" } else { "debug" });
  Ok(path)
}

/// Walks up the file system, looking for a Cargo.toml file
/// If one is found before reaching the root, then the current_dir's package belongs to that parent workspace if it's listed on [workspace.members].
///
/// If this package is part of a workspace, returns the path to the workspace directory
/// Otherwise returns the current directory.
pub fn get_workspace_dir(current_dir: &Path) -> PathBuf {
  let mut dir = current_dir.to_path_buf();
  let project_path = dir.clone();

  while dir.pop() {
    if dir.join("Cargo.toml").exists() {
      match CargoSettings::load(&dir) {
        Ok(cargo_settings) => {
          if let Some(workspace_settings) = cargo_settings.workspace {
            if let Some(members) = workspace_settings.members {
              if members.iter().any(|member| {
                glob::glob(&dir.join(member).to_string_lossy())
                  .unwrap()
                  .any(|p| p.unwrap() == project_path)
              }) {
                return dir;
              }
            }
          }
        }
        Err(e) => {
          warn!(
              "Found `{}`, which may define a parent workspace, but \
            failed to parse it. If this is indeed a parent workspace, undefined behavior may occur: \
            \n    {:#}",
              dir.display(),
              e
            );
        }
      }
    }
  }

  // Nothing found walking up the file system, return the starting directory
  current_dir.to_path_buf()
}

#[allow(unused_variables)]
fn tauri_config_to_bundle_settings(
  manifest: &Manifest,
  features: &[String],
  config: crate::helpers::config::BundleConfig,
  system_tray_config: Option<crate::helpers::config::SystemTrayConfig>,
  updater_config: crate::helpers::config::UpdaterConfig,
) -> crate::Result<BundleSettings> {
  let enabled_features = manifest.all_enabled_features(features);

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
  let mut resources = config.resources.unwrap_or_default();
  #[allow(unused_mut)]
  let mut depends = config.deb.depends.unwrap_or_default();

  #[cfg(target_os = "linux")]
  {
    if let Some(system_tray_config) = &system_tray_config {
      depends.push("pkg-config".to_string());
      let tray = std::env::var("TAURI_TRAY").unwrap_or_else(|_| "ayatana".to_string());
      if tray == "ayatana" {
        depends.push("libayatana-appindicator3-1".into());
      } else {
        depends.push("libappindicator3-1".into());
      }
    }

    // provides `libwebkit2gtk-4.0.so.37` and all `4.0` versions have the -37 package name
    depends.push("libwebkit2gtk-4.0-37".to_string());
    depends.push("libgtk-3-0".to_string());
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

  Ok(BundleSettings {
    identifier: Some(config.identifier),
    icon: Some(config.icon),
    resources: if resources.is_empty() {
      None
    } else {
      Some(resources)
    },
    copyright: config.copyright,
    category: match config.category {
      Some(category) => Some(AppCategory::from_str(&category).map_err(|e| match e {
        Some(e) => anyhow::anyhow!("invalid category, did you mean `{}`?", e),
        None => anyhow::anyhow!("invalid category"),
      })?),
      None => None,
    },
    short_description: config.short_description,
    long_description: config.long_description,
    external_bin: config.external_bin,
    deb: DebianSettings {
      depends: if depends.is_empty() {
        None
      } else {
        Some(depends)
      },
      files: config.deb.files,
    },
    macos: MacOsSettings {
      frameworks: config.macos.frameworks,
      minimum_system_version: config.macos.minimum_system_version,
      license: config.macos.license,
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
      wix: config.windows.wix.map(|w| {
        let mut wix = wix_settings(w);
        wix.license = wix.license.map(|l| tauri_dir().join(l));
        wix
      }),
      icon_path: windows_icon_path,
      webview_install_mode: config.windows.webview_install_mode,
      webview_fixed_runtime_path: config.windows.webview_fixed_runtime_path,
      allow_downgrades: config.windows.allow_downgrades,
    },
    updater: Some(UpdaterSettings {
      active: updater_config.active,
      // we set it to true by default we shouldn't have to use
      // unwrap_or as we have a default value but used to prevent any failing
      dialog: updater_config.dialog,
      pubkey: updater_config.pubkey,
      endpoints: updater_config
        .endpoints
        .map(|endpoints| endpoints.iter().map(|e| e.to_string()).collect()),
      msiexec_args: Some(updater_config.windows.install_mode.msiexec_args()),
    }),
    ..Default::default()
  })
}

fn rename_app(bin_path: PathBuf, product_name: Option<&str>) -> crate::Result<PathBuf> {
  if let Some(product_name) = product_name {
    #[cfg(target_os = "linux")]
    let product_name = product_name.to_kebab_case();

    let product_path = bin_path
      .parent()
      .unwrap()
      .join(&product_name)
      .with_extension(bin_path.extension().unwrap_or_default());

    rename(&bin_path, &product_path).with_context(|| {
      format!(
        "failed to rename `{}` to `{}`",
        bin_path.display(),
        product_path.display(),
      )
    })?;
    Ok(product_path)
  } else {
    Ok(bin_path)
  }
}

// taken from https://github.com/rust-lang/cargo/blob/78b10d4e611ab0721fc3aeaf0edd5dd8f4fdc372/src/cargo/core/shell.rs#L514
#[cfg(unix)]
mod terminal {
  use std::mem;

  pub fn stderr_width() -> Option<usize> {
    unsafe {
      let mut winsize: libc::winsize = mem::zeroed();
      // The .into() here is needed for FreeBSD which defines TIOCGWINSZ
      // as c_uint but ioctl wants c_ulong.
      #[allow(clippy::useless_conversion)]
      if libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ.into(), &mut winsize) < 0 {
        return None;
      }
      if winsize.ws_col > 0 {
        Some(winsize.ws_col as usize)
      } else {
        None
      }
    }
  }
}

// taken from https://github.com/rust-lang/cargo/blob/78b10d4e611ab0721fc3aeaf0edd5dd8f4fdc372/src/cargo/core/shell.rs#L543
#[cfg(windows)]
mod terminal {
  use std::{cmp, mem, ptr};
  use winapi::um::fileapi::*;
  use winapi::um::handleapi::*;
  use winapi::um::processenv::*;
  use winapi::um::winbase::*;
  use winapi::um::wincon::*;
  use winapi::um::winnt::*;

  pub fn stderr_width() -> Option<usize> {
    unsafe {
      let stdout = GetStdHandle(STD_ERROR_HANDLE);
      let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
      if GetConsoleScreenBufferInfo(stdout, &mut csbi) != 0 {
        return Some((csbi.srWindow.Right - csbi.srWindow.Left) as usize);
      }

      // On mintty/msys/cygwin based terminals, the above fails with
      // INVALID_HANDLE_VALUE. Use an alternate method which works
      // in that case as well.
      let h = CreateFileA(
        "CONOUT$\0".as_ptr() as *const CHAR,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        ptr::null_mut(),
        OPEN_EXISTING,
        0,
        ptr::null_mut(),
      );
      if h == INVALID_HANDLE_VALUE {
        return None;
      }

      let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();
      let rc = GetConsoleScreenBufferInfo(h, &mut csbi);
      CloseHandle(h);
      if rc != 0 {
        let width = (csbi.srWindow.Right - csbi.srWindow.Left) as usize;
        // Unfortunately cygwin/mintty does not set the size of the
        // backing console to match the actual window size. This
        // always reports a size of 80 or 120 (not sure what
        // determines that). Use a conservative max of 60 which should
        // work in most circumstances. ConEmu does some magic to
        // resize the console correctly, but there's no reasonable way
        // to detect which kind of terminal we are running in, or if
        // GetConsoleScreenBufferInfo returns accurate information.
        return Some(cmp::min(60, width));
      }

      None
    }
  }
}
