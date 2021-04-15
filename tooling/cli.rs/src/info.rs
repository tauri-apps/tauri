// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::get as get_config,
};
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
  js_cli: JsCliVersionMetadata,
}

#[derive(Clone, Deserialize)]
struct CargoManifestDependencyPackage {
  version: Option<String>,
  path: Option<PathBuf>,
  #[serde(default)]
  features: Vec<String>,
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

#[derive(Default)]
pub struct Info;

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

fn npm_latest_version(use_yarn: bool, name: &str) -> crate::Result<Option<String>> {
  if use_yarn {
    let output = Command::new("yarn")
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
  } else {
    let output = Command::new("npm")
      .arg("show")
      .arg(name)
      .arg("version")
      .output()?;
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      Ok(Some(stdout.replace("\n", "")))
    } else {
      Ok(None)
    }
  }
}

fn npm_package_version<P: AsRef<Path>>(
  use_yarn: bool,
  name: &str,
  app_dir: P,
) -> crate::Result<Option<String>> {
  let output = if use_yarn {
    Command::new("yarn")
      .args(&["list", "--pattern"])
      .arg(name)
      .args(&["--depth", "0"])
      .current_dir(app_dir)
      .output()?
  } else {
    Command::new("npm")
      .arg("list")
      .arg(name)
      .args(&["version", "--depth", "0"])
      .current_dir(app_dir)
      .output()?
  };
  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let regex = regex::Regex::new("@([\\da-zA-Z\\.]+)").unwrap();
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
  let output = Command::new(command).args(args).arg("--version").output()?;
  let version = if output.status.success() {
    Some(String::from_utf8_lossy(&output.stdout).replace("\n", ""))
  } else {
    None
  };
  Ok(version)
}

struct InfoBlock {
  section: bool,
  key: &'static str,
  value: Option<String>,
  suffix: Option<String>,
}

impl InfoBlock {
  fn new(key: &'static str) -> Self {
    Self {
      section: false,
      key,
      value: None,
      suffix: None,
    }
  }

  fn section(mut self) -> Self {
    self.section = true;
    self
  }

  fn value<V: Into<Option<String>>>(mut self, value: V) -> Self {
    self.value = value.into();
    self
  }

  fn suffix<S: Into<Option<String>>>(mut self, suffix: S) -> Self {
    self.suffix = suffix.into();
    self
  }

  fn display(&self) {
    if self.section {
      println!();
    }
    print!("{}", self.key);
    if let Some(value) = &self.value {
      print!(" - {}", value);
    }
    if let Some(suffix) = &self.suffix {
      print!("{}", suffix);
    }
    println!();
  }
}

struct VersionBlock {
  section: bool,
  key: &'static str,
  version: Option<String>,
  target_version: Option<String>,
}

impl VersionBlock {
  fn new<V: Into<Option<String>>>(key: &'static str, version: V) -> Self {
    Self {
      section: false,
      key,
      version: version.into(),
      target_version: None,
    }
  }

  fn target_version<V: Into<Option<String>>>(mut self, version: V) -> Self {
    self.target_version = version.into();
    self
  }

  fn display(&self) {
    if self.section {
      println!();
    }
    print!("{}", self.key);
    if let Some(version) = &self.version {
      print!(" - {}", version);
    } else {
      print!(" Not installed");
    }
    if let (Some(version), Some(target_version)) = (&self.version, &self.target_version) {
      let version = semver::Version::parse(version).unwrap();
      let target_version = semver::Version::parse(target_version).unwrap();
      if version < target_version {
        print!(" (outdated, latest: {})", target_version);
      }
    }
    println!();
  }
}

impl Info {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn run(self) -> crate::Result<()> {
    let os_info = os_info::get();
    InfoBlock {
      section: true,
      key: "Operating System",
      value: Some(format!(
        "{}, version {} {:?}",
        os_info.os_type(),
        os_info.version(),
        os_info.bitness()
      )),
      suffix: None,
    }
    .display();

    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_info| {
      // do nothing
    }));
    let app_dir = panic::catch_unwind(app_dir).map(Some).unwrap_or_default();
    panic::set_hook(hook);

    let use_yarn = app_dir
      .map(|dir| dir.join("yarn.lock").exists())
      .unwrap_or_default();

    if let Some(node_version) = get_version("node", &[]).unwrap_or_default() {
      InfoBlock::new("Node.js environment").section().display();
      let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json"))?;
      VersionBlock::new(
        "  Node.js",
        node_version.chars().skip(1).collect::<String>(),
      )
      .target_version(metadata.js_cli.node.replace(">= ", ""))
      .display();

      VersionBlock::new("  @tauri-apps/cli", metadata.js_cli.version)
        .target_version(npm_latest_version(use_yarn, "@tauri-apps/cli").unwrap_or_default())
        .display();
      if let Some(app_dir) = &app_dir {
        VersionBlock::new(
          "  @tauri-apps/api",
          npm_package_version(use_yarn, "@tauri-apps/api", app_dir).unwrap_or_default(),
        )
        .target_version(npm_latest_version(use_yarn, "@tauri-apps/api").unwrap_or_default())
        .display();
      }

      InfoBlock::new("Global packages").section().display();

      VersionBlock::new("  npm", get_version("npm", &[]).unwrap_or_default()).display();
      VersionBlock::new("  yarn", get_version("yarn", &[]).unwrap_or_default()).display();
    }

    InfoBlock::new("Rust environment").section().display();
    VersionBlock::new(
      "  rustc",
      get_version("rustc", &[]).unwrap_or_default().map(|v| {
        let mut s = v.split(' ');
        s.next();
        s.next().unwrap().to_string()
      }),
    )
    .display();
    VersionBlock::new(
      "  cargo",
      get_version("cargo", &[]).unwrap_or_default().map(|v| {
        let mut s = v.split(' ');
        s.next();
        s.next().unwrap().to_string()
      }),
    )
    .display();

    if let Some(app_dir) = app_dir {
      InfoBlock::new("App directory structure")
        .section()
        .display();
      for entry in read_dir(app_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
          println!("/{}", entry.path().file_name().unwrap().to_string_lossy());
        }
      }

      InfoBlock::new("App").section().display();
      let tauri_dir = tauri_dir();
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
      let tauri_lock_packages: Vec<CargoLockPackage> = lock
        .as_ref()
        .map(|lock| {
          lock
            .package
            .iter()
            .filter(|p| p.name == "tauri")
            .cloned()
            .collect()
        })
        .unwrap_or_default();
      let (tauri_version_string, found_tauri_versions) =
        match (&manifest, &lock, tauri_lock_packages.len()) {
          (Some(_manifest), Some(_lock), 1) => {
            let tauri_lock_package = tauri_lock_packages.first().unwrap();
            (
              tauri_lock_package.version.clone(),
              vec![tauri_lock_package.version.clone()],
            )
          }
          (None, Some(_lock), 1) => {
            let tauri_lock_package = tauri_lock_packages.first().unwrap();
            (
              format!("{} (no manifest)", tauri_lock_package.version),
              vec![tauri_lock_package.version.clone()],
            )
          }
          _ => {
            let mut found_tauri_versions = Vec::new();
            let manifest_version = match manifest.and_then(|m| m.dependencies.get("tauri").cloned())
            {
              Some(tauri) => match tauri {
                CargoManifestDependency::Version(v) => {
                  found_tauri_versions.push(v.clone());
                  v
                }
                CargoManifestDependency::Package(p) => {
                  if let Some(v) = p.version {
                    found_tauri_versions.push(v.clone());
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
                  } else {
                    "unknown manifest".to_string()
                  }
                }
              },
              None => "no manifest".to_string(),
            };

            let lock_version = match (lock, tauri_lock_packages.is_empty()) {
              (Some(_lock), true) => tauri_lock_packages
                .iter()
                .map(|p| p.version.clone())
                .collect::<Vec<String>>()
                .join(", "),
              (Some(_lock), false) => "unknown lockfile".to_string(),
              _ => "no lockfile".to_string(),
            };

            (
              format!("{} ({})", manifest_version, lock_version),
              found_tauri_versions,
            )
          }
        };

      let tauri_version = found_tauri_versions
        .into_iter()
        .map(|v| semver::Version::parse(&v).unwrap())
        .max();
      let suffix = match (tauri_version, crate_latest_version("tauri")) {
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
      InfoBlock::new("  tauri.rs")
        .value(tauri_version_string)
        .suffix(suffix)
        .display();

      if let Ok(config) = get_config(None) {
        let config_guard = config.lock().unwrap();
        let config = config_guard.as_ref().unwrap();
        InfoBlock::new("build-type")
          .value(if config.tauri.bundle.active {
            "bundle".to_string()
          } else {
            "build".to_string()
          })
          .display();
        InfoBlock::new("CSP")
          .value(if let Some(security) = &config.tauri.security {
            security.csp.clone().unwrap_or_else(|| "unset".to_string())
          } else {
            "unset".to_string()
          })
          .display();
        InfoBlock::new("distDir")
          .value(config.build.dist_dir.clone())
          .display();
        InfoBlock::new("devPath")
          .value(config.build.dev_path.clone())
          .display();
      }
    }

    Ok(())
  }
}
