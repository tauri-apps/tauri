// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  config::get as get_config, framework::infer_from_package_json as infer_framework,
};
use crate::Result;
use clap::Parser;
use colored::Colorize;
use serde::Deserialize;

use std::{
  collections::HashMap,
  fs::{read_dir, read_to_string},
  panic,
  path::{Path, PathBuf},
  process::Command,
};

#[derive(Deserialize)]
struct YarnVersionInfo {
  data: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct CargoLockPackage {
  name: String,
  version: String,
  source: Option<String>,
}

#[derive(Deserialize)]
struct CargoLock {
  package: Vec<CargoLockPackage>,
}

#[derive(Deserialize)]
struct JsCliVersionMetadata {
  version: String,
  node: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionMetadata {
  #[serde(rename = "cli.js")]
  js_cli: JsCliVersionMetadata,
}

#[derive(Clone, Deserialize)]
struct CargoManifestDependencyPackage {
  version: Option<String>,
  git: Option<String>,
  branch: Option<String>,
  rev: Option<String>,
  path: Option<PathBuf>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum CargoManifestDependency {
  Version(String),
  Package(CargoManifestDependencyPackage),
}

#[derive(Deserialize)]
struct CargoManifestPackage {
  version: String,
}

#[derive(Deserialize)]
struct CargoManifest {
  package: CargoManifestPackage,
  dependencies: HashMap<String, CargoManifestDependency>,
}

#[derive(Debug, PartialEq, Eq)]
enum PackageManager {
  Npm,
  Pnpm,
  Yarn,
  Berry,
}

#[derive(Debug, Parser)]
#[clap(about = "Shows information about Tauri dependencies and project configuration")]
pub struct Options;

fn version_metadata() -> Result<VersionMetadata> {
  serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json")).map_err(Into::into)
}

#[cfg(not(debug_assertions))]
pub(crate) fn cli_current_version() -> Result<String> {
  version_metadata().map(|meta| meta.js_cli.version)
}

#[cfg(not(debug_assertions))]
pub(crate) fn cli_upstream_version() -> Result<String> {
  let upstream_metadata = match ureq::get(
    "https://raw.githubusercontent.com/tauri-apps/tauri/dev/tooling/cli/metadata.json",
  )
  .timeout(std::time::Duration::from_secs(3))
  .call()
  {
    Ok(r) => r,
    Err(ureq::Error::Status(code, _response)) => {
      let message = format!("Unable to find updates at the moment. Code: {}", code);
      return Err(anyhow::Error::msg(message));
    }
    Err(ureq::Error::Transport(transport)) => {
      let message = format!(
        "Unable to find updates at the moment. Error: {:?}",
        transport.kind()
      );
      return Err(anyhow::Error::msg(message));
    }
  };

  upstream_metadata
    .into_string()
    .and_then(|meta_str| Ok(serde_json::from_str::<VersionMetadata>(&meta_str)))
    .and_then(|json| Ok(json.unwrap().js_cli.version))
    .map_err(|e| anyhow::Error::new(e))
}

fn crate_latest_version(name: &str) -> Option<String> {
  let url = format!("https://docs.rs/crate/{}/", name);
  match ureq::get(&url).call() {
    Ok(response) => match (response.status(), response.header("location")) {
      (302, Some(location)) => Some(location.replace(&url, "")),
      _ => None,
    },
    Err(_) => None,
  }
}

#[allow(clippy::let_and_return)]
fn cross_command(bin: &str) -> Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = Command::new(bin);
  cmd
}

fn npm_latest_version(pm: &PackageManager, name: &str) -> crate::Result<Option<String>> {
  match pm {
    PackageManager::Yarn => {
      let mut cmd = cross_command("yarn");

      let output = cmd
        .arg("info")
        .arg(name)
        .args(&["version", "--json"])
        .output()?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: YarnVersionInfo = serde_json::from_str(&stdout)?;
        Ok(Some(info.data.last().unwrap().to_string()))
      } else {
        Ok(None)
      }
    }
    PackageManager::Berry => {
      let mut cmd = cross_command("yarn");

      let output = cmd
        .arg("npm")
        .arg("info")
        .arg(name)
        .args(["--fields", "version", "--json"])
        .output()?;
      if output.status.success() {
        let info: crate::PackageJson =
          serde_json::from_reader(std::io::Cursor::new(output.stdout)).unwrap();
        Ok(info.version)
      } else {
        Ok(None)
      }
    }
    PackageManager::Npm => {
      let mut cmd = cross_command("npm");

      let output = cmd.arg("show").arg(name).arg("version").output()?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Some(stdout.replace('\n', "")))
      } else {
        Ok(None)
      }
    }
    PackageManager::Pnpm => {
      let mut cmd = cross_command("pnpm");

      let output = cmd.arg("info").arg(name).arg("version").output()?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Some(stdout.replace('\n', "")))
      } else {
        Ok(None)
      }
    }
  }
}

fn npm_package_version<P: AsRef<Path>>(
  pm: &PackageManager,
  name: &str,
  app_dir: P,
) -> crate::Result<Option<String>> {
  let (output, regex) = match pm {
    PackageManager::Yarn => (
      cross_command("yarn")
        .args(&["list", "--pattern"])
        .arg(name)
        .args(&["--depth", "0"])
        .current_dir(app_dir)
        .output()?,
      None,
    ),
    PackageManager::Berry => (
      cross_command("yarn")
        .arg("info")
        .arg(name)
        .current_dir(app_dir)
        .output()?,
      Some(regex::Regex::new("Version: ([\\da-zA-Z\\-\\.]+)").unwrap()),
    ),
    PackageManager::Npm => (
      cross_command("npm")
        .arg("list")
        .arg(name)
        .args(&["version", "--depth", "0"])
        .current_dir(app_dir)
        .output()?,
      None,
    ),
    PackageManager::Pnpm => (
      cross_command("pnpm")
        .arg("list")
        .arg(name)
        .args(&["--parseable", "--depth", "0"])
        .current_dir(app_dir)
        .output()?,
      None,
    ),
  };
  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let regex = regex.unwrap_or_else(|| regex::Regex::new("@([\\da-zA-Z\\-\\.]+)").unwrap());
    Ok(
      regex
        .captures_iter(&stdout)
        .last()
        .and_then(|cap| cap.get(1).map(|v| v.as_str().to_string())),
    )
  } else {
    Ok(None)
  }
}

fn get_version(command: &str, args: &[&str]) -> crate::Result<Option<String>> {
  let output = cross_command(command)
    .args(args)
    .arg("--version")
    .output()?;
  let version = if output.status.success() {
    Some(
      String::from_utf8_lossy(&output.stdout)
        .replace('\n', "")
        .replace('\r', ""),
    )
  } else {
    None
  };
  Ok(version)
}

#[cfg(windows)]
fn webview2_version() -> crate::Result<Option<String>> {
  // check 64bit machine-wide installation
  let output = Command::new("powershell")
      .args(&["-NoProfile", "-Command"])
      .arg("Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
      .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }
  // check 32bit machine-wide installation
  let output = Command::new("powershell")
        .args(&["-NoProfile", "-Command"])
        .arg("Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
        .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }
  // check user-wide installation
  let output = Command::new("powershell")
      .args(&["-NoProfile", "-Command"])
      .arg("Get-ItemProperty -Path 'HKCU:\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' | ForEach-Object {$_.pv}")
      .output()?;
  if output.status.success() {
    return Ok(Some(
      String::from_utf8_lossy(&output.stdout).replace('\n', ""),
    ));
  }

  Ok(None)
}

#[cfg(windows)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VsInstanceInfo {
  display_name: String,
}

#[cfg(windows)]
const VSWHERE: &[u8] = include_bytes!("../scripts/vswhere.exe");

#[cfg(windows)]
fn build_tools_version() -> crate::Result<Option<Vec<String>>> {
  let mut vswhere = std::env::temp_dir();
  vswhere.push("vswhere.exe");

  if !vswhere.exists() {
    if let Ok(mut file) = std::fs::File::create(&vswhere) {
      use std::io::Write;
      let _ = file.write_all(VSWHERE);
    }
  }
  let output = cross_command(vswhere.to_str().unwrap())
    .args(&[
      "-prerelease",
      "-products",
      "*",
      "-requiresAny",
      "-requires",
      "Microsoft.VisualStudio.Workload.NativeDesktop",
      "-requires",
      "Microsoft.VisualStudio.Workload.VCTools",
      "-format",
      "json",
    ])
    .output()?;
  Ok(if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let instances: Vec<VsInstanceInfo> = serde_json::from_str(&stdout)?;
    Some(
      instances
        .iter()
        .map(|i| i.display_name.clone())
        .collect::<Vec<String>>(),
    )
  } else {
    None
  })
}

fn active_rust_toolchain() -> crate::Result<Option<String>> {
  let output = cross_command("rustup")
    .args(["show", "active-toolchain"])
    .output()?;
  let toolchain = if output.status.success() {
    Some(
      String::from_utf8_lossy(&output.stdout)
        .replace('\n', "")
        .replace('\r', "")
        .split('(')
        .collect::<Vec<&str>>()[0]
        .into(),
    )
  } else {
    None
  };
  Ok(toolchain)
}

fn crate_version(
  tauri_dir: &Path,
  manifest: Option<&CargoManifest>,
  lock: Option<&CargoLock>,
  name: &str,
) -> (String, Option<String>) {
  let crate_lock_packages: Vec<CargoLockPackage> = lock
    .as_ref()
    .map(|lock| {
      lock
        .package
        .iter()
        .filter(|p| p.name == name)
        .cloned()
        .collect()
    })
    .unwrap_or_default();
  let (crate_version_string, found_crate_versions) =
    match (&manifest, &lock, crate_lock_packages.len()) {
      (Some(_manifest), Some(_lock), 1) => {
        let crate_lock_package = crate_lock_packages.first().unwrap();
        let version_string = if let Some(s) = &crate_lock_package.source {
          if s.starts_with("git") {
            format!("{} ({})", s, crate_lock_package.version)
          } else {
            crate_lock_package.version.clone()
          }
        } else {
          crate_lock_package.version.clone()
        };
        (version_string, vec![crate_lock_package.version.clone()])
      }
      (None, Some(_lock), 1) => {
        let crate_lock_package = crate_lock_packages.first().unwrap();
        let version_string = if let Some(s) = &crate_lock_package.source {
          if s.starts_with("git") {
            format!("{} ({})", s, crate_lock_package.version)
          } else {
            crate_lock_package.version.clone()
          }
        } else {
          crate_lock_package.version.clone()
        };
        (
          format!("{} (no manifest)", version_string),
          vec![crate_lock_package.version.clone()],
        )
      }
      _ => {
        let mut found_crate_versions = Vec::new();
        let mut is_git = false;
        let manifest_version = match manifest.and_then(|m| m.dependencies.get(name).cloned()) {
          Some(tauri) => match tauri {
            CargoManifestDependency::Version(v) => {
              found_crate_versions.push(v.clone());
              v
            }
            CargoManifestDependency::Package(p) => {
              if let Some(v) = p.version {
                found_crate_versions.push(v.clone());
                v
              } else if let Some(p) = p.path {
                let manifest_path = tauri_dir.join(&p).join("Cargo.toml");
                let v = match read_to_string(&manifest_path)
                  .map_err(|_| ())
                  .and_then(|m| toml::from_str::<CargoManifest>(&m).map_err(|_| ()))
                {
                  Ok(manifest) => manifest.package.version,
                  Err(_) => "unknown version".to_string(),
                };
                format!("path:{:?} [{}]", p, v)
              } else if let Some(g) = p.git {
                is_git = true;
                let mut v = format!("git:{}", g);
                if let Some(branch) = p.branch {
                  v.push_str(&format!("&branch={}", branch));
                } else if let Some(rev) = p.rev {
                  v.push_str(&format!("#{}", rev));
                }
                v
              } else {
                "unknown manifest".to_string()
              }
            }
          },
          None => "no manifest".to_string(),
        };

        let lock_version = match (lock, crate_lock_packages.is_empty()) {
          (Some(_lock), true) => crate_lock_packages
            .iter()
            .map(|p| p.version.clone())
            .collect::<Vec<String>>()
            .join(", "),
          (Some(_lock), false) => "unknown lockfile".to_string(),
          _ => "no lockfile".to_string(),
        };

        (
          format!(
            "{} {}({})",
            manifest_version,
            if is_git { "(git manifest)" } else { "" },
            lock_version
          ),
          found_crate_versions,
        )
      }
    };

  let crate_version = found_crate_versions
    .into_iter()
    .map(|v| semver::Version::parse(&v).unwrap())
    .max();
  let suffix = match (crate_version, crate_latest_version(name)) {
    (Some(version), Some(target_version)) => {
      let target_version = semver::Version::parse(&target_version).unwrap();
      if version < target_version {
        Some(format!(" (outdated, latest: {})", target_version))
      } else {
        None
      }
    }
    _ => None,
  };
  (crate_version_string, suffix)
}

fn indent(spaces: usize) {
  print!("{}", " ".repeat(spaces));
}

struct Section(&'static str);
impl Section {
  fn display(&self) {
    println!();
    println!("{}", self.0.yellow().bold());
  }
}

struct VersionBlock {
  name: String,
  version: String,
  target_version: String,
  indentation: usize,
}

impl VersionBlock {
  fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      version: version.into(),
      target_version: "".into(),
      indentation: 2,
    }
  }

  fn target_version(mut self, version: impl Into<String>) -> Self {
    self.target_version = version.into();
    self
  }

  fn display(&self) {
    indent(self.indentation);
    print!("{} ", "›".cyan());
    print!("{}", self.name.bold());
    print!(": ");
    print!(
      "{}",
      if self.version.is_empty() {
        "Not installed!".red().to_string()
      } else {
        self.version.clone()
      }
    );
    if !(self.version.is_empty() || self.target_version.is_empty()) {
      let version = semver::Version::parse(self.version.as_str()).unwrap();
      let target_version = semver::Version::parse(self.target_version.as_str()).unwrap();
      if version < target_version {
        print!(
          " ({}, latest: {})",
          "outdated".red(),
          self.target_version.green()
        );
      }
    }
    println!();
  }
}

struct InfoBlock {
  key: String,
  value: String,
  indentation: usize,
}

impl InfoBlock {
  fn new(key: impl Into<String>, val: impl Into<String>) -> Self {
    Self {
      key: key.into(),
      value: val.into(),
      indentation: 2,
    }
  }

  fn display(&self) {
    indent(self.indentation);
    print!("{} ", "›".cyan());
    print!("{}", self.key.bold());
    print!(": ");
    print!("{}", self.value.clone());
    println!();
  }
}

pub fn command(_options: Options) -> Result<()> {
  Section("Environment").display();

  let os_info = os_info::get();
  VersionBlock::new(
    "OS",
    format!(
      "{} {} {:?}",
      os_info.os_type(),
      os_info.version(),
      os_info.bitness()
    ),
  )
  .display();

  #[cfg(windows)]
  VersionBlock::new(
    "Webview2",
    webview2_version().unwrap_or_default().unwrap_or_default(),
  )
  .display();

  #[cfg(windows)]
  {
    let build_tools = build_tools_version()
      .unwrap_or_default()
      .unwrap_or_default();

    if build_tools.is_empty() {
      InfoBlock::new("MSVC", "").display();
    } else {
      InfoBlock::new("MSVC", "").display();
      for i in build_tools {
        indent(6);
        println!("{}", format!("{} {}", "-".cyan(), i));
      }
    }
  }

  let hook = panic::take_hook();
  panic::set_hook(Box::new(|_info| {
    // do nothing
  }));
  let app_dir = panic::catch_unwind(crate::helpers::app_paths::app_dir)
    .map(Some)
    .unwrap_or_default();
  panic::set_hook(hook);

  let yarn_version = get_version("yarn", &[])
    .unwrap_or_default()
    .unwrap_or_default();

  let metadata = version_metadata()?;
  VersionBlock::new(
    "Node.js",
    get_version("node", &[])
      .unwrap_or_default()
      .unwrap_or_default()
      .chars()
      .skip(1)
      .collect::<String>(),
  )
  .target_version(metadata.js_cli.node.replace(">= ", ""))
  .display();

  VersionBlock::new(
    "npm",
    get_version("npm", &[])
      .unwrap_or_default()
      .unwrap_or_default(),
  )
  .display();
  VersionBlock::new(
    "pnpm",
    get_version("pnpm", &[])
      .unwrap_or_default()
      .unwrap_or_default(),
  )
  .display();
  VersionBlock::new("yarn", &yarn_version).display();
  VersionBlock::new(
    "rustup",
    get_version("rustup", &[])
      .unwrap_or_default()
      .map(|v| {
        let mut s = v.split(' ');
        s.next();
        s.next().unwrap().to_string()
      })
      .unwrap_or_default(),
  )
  .display();
  VersionBlock::new(
    "rustc",
    get_version("rustc", &[])
      .unwrap_or_default()
      .map(|v| {
        let mut s = v.split(' ');
        s.next();
        s.next().unwrap().to_string()
      })
      .unwrap_or_default(),
  )
  .display();
  VersionBlock::new(
    "cargo",
    get_version("cargo", &[])
      .unwrap_or_default()
      .map(|v| {
        let mut s = v.split(' ');
        s.next();
        s.next().unwrap().to_string()
      })
      .unwrap_or_default(),
  )
  .display();
  InfoBlock::new(
    "Rust toolchain",
    active_rust_toolchain()
      .unwrap_or_default()
      .unwrap_or_default(),
  )
  .display();

  Section("Packages").display();

  let mut package_manager = PackageManager::Npm;
  if let Some(app_dir) = &app_dir {
    let app_dir_entries = read_dir(app_dir)
      .unwrap()
      .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
      .collect::<Vec<String>>();
    package_manager = get_package_manager(&app_dir_entries)?;
  }

  if package_manager == PackageManager::Yarn
    && yarn_version
      .chars()
      .next()
      .map(|c| c > '1')
      .unwrap_or_default()
  {
    package_manager = PackageManager::Berry;
  }

  VersionBlock::new(
    format!("{} {}", "@tauri-apps/cli", "[NPM]".dimmed()),
    metadata.js_cli.version,
  )
  .target_version(
    npm_latest_version(&package_manager, "@tauri-apps/cli")
      .unwrap_or_default()
      .unwrap_or_default(),
  )
  .display();
  if let Some(app_dir) = &app_dir {
    VersionBlock::new(
      format!("{} {}", "@tauri-apps/api", "[NPM]".dimmed()),
      npm_package_version(&package_manager, "@tauri-apps/api", app_dir)
        .unwrap_or_default()
        .unwrap_or_default(),
    )
    .target_version(
      npm_latest_version(&package_manager, "@tauri-apps/api")
        .unwrap_or_default()
        .unwrap_or_default(),
    )
    .display();
  }

  let hook = panic::take_hook();
  panic::set_hook(Box::new(|_info| {
    // do nothing
  }));
  let tauri_dir = panic::catch_unwind(crate::helpers::app_paths::tauri_dir)
    .map(Some)
    .unwrap_or_default();
  panic::set_hook(hook);

  if tauri_dir.is_some() || app_dir.is_some() {
    if let Some(tauri_dir) = tauri_dir.clone() {
      let manifest: Option<CargoManifest> =
        if let Ok(manifest_contents) = read_to_string(tauri_dir.join("Cargo.toml")) {
          toml::from_str(&manifest_contents).ok()
        } else {
          None
        };
      let lock: Option<CargoLock> =
        if let Ok(lock_contents) = read_to_string(tauri_dir.join("Cargo.lock")) {
          toml::from_str(&lock_contents).ok()
        } else {
          None
        };

      for (dep, label) in [
        ("tauri", format!("{} {}", "tauri", "[RUST]".dimmed())),
        (
          "tauri-build",
          format!("{} {}", "tauri-build", "[RUST]".dimmed()),
        ),
        ("tao", format!("{} {}", "tao", "[RUST]".dimmed())),
        ("wry", format!("{} {}", "wry", "[RUST]".dimmed())),
      ] {
        let (version_string, version_suffix) =
          crate_version(&tauri_dir, manifest.as_ref(), lock.as_ref(), dep);
        VersionBlock::new(
          label,
          format!(
            "{},{}",
            version_string,
            version_suffix.unwrap_or_else(|| "".into())
          ),
        )
        .display();
      }
    }
  }

  if tauri_dir.is_some() || app_dir.is_some() {
    Section("App").display();
    if tauri_dir.is_some() {
      if let Ok(config) = get_config(None) {
        let config_guard = config.lock().unwrap();
        let config = config_guard.as_ref().unwrap();
        InfoBlock::new(
          "build-type",
          if config.tauri.bundle.active {
            "bundle".to_string()
          } else {
            "build".to_string()
          },
        )
        .display();
        InfoBlock::new(
          "CSP",
          config
            .tauri
            .security
            .csp
            .clone()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unset".to_string()),
        )
        .display();
        InfoBlock::new("distDir", config.build.dist_dir.to_string()).display();
        InfoBlock::new("devPath", config.build.dev_path.to_string()).display();
      }
    }

    if let Some(app_dir) = app_dir {
      if let Ok(package_json) = read_to_string(app_dir.join("package.json")) {
        let (framework, bundler) = infer_framework(&package_json);
        if let Some(framework) = framework {
          InfoBlock::new("framework", framework.to_string()).display();
        }
        if let Some(bundler) = bundler {
          InfoBlock::new("bundler", bundler.to_string()).display();
        }
      } else {
        println!("package.json not found");
      }
    }
  }

  if let Some(app_dir) = app_dir {
    Section("App directory structure").display();
    let dirs = read_dir(app_dir)?
      .filter(|p| p.is_ok() && p.as_ref().unwrap().path().is_dir())
      .collect::<Vec<Result<std::fs::DirEntry, _>>>();
    let dirs_len = dirs.len();
    for (i, entry) in dirs.into_iter().enumerate() {
      let entry = entry?;
      let prefix = if i + 1 == dirs_len {
        "└─".cyan()
      } else {
        "├─".cyan()
      };
      println!(
        "  {} {}",
        prefix,
        entry.path().file_name().unwrap().to_string_lossy()
      );
    }
  }

  Ok(())
}

fn get_package_manager<T: AsRef<str>>(app_dir_entries: &[T]) -> crate::Result<PackageManager> {
  let mut use_npm = false;
  let mut use_pnpm = false;
  let mut use_yarn = false;

  for name in app_dir_entries {
    if name.as_ref() == "package-lock.json" {
      use_npm = true;
    } else if name.as_ref() == "pnpm-lock.yaml" {
      use_pnpm = true;
    } else if name.as_ref() == "yarn.lock" {
      use_yarn = true;
    }
  }

  if !use_npm && !use_pnpm && !use_yarn {
    println!("WARNING: no lock files found, defaulting to npm");
    return Ok(PackageManager::Npm);
  }

  let mut found = Vec::new();

  if use_npm {
    found.push("npm");
  }
  if use_pnpm {
    found.push("pnpm");
  }
  if use_yarn {
    found.push("yarn");
  }

  if found.len() > 1 {
    return Err(anyhow::anyhow!(
        "only one package mangager should be used, but found {}\nplease remove unused package manager lock files",
        found.join(" and ")
      ));
  }

  if use_npm {
    Ok(PackageManager::Npm)
  } else if use_pnpm {
    Ok(PackageManager::Pnpm)
  } else {
    Ok(PackageManager::Yarn)
  }
}
