// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

use std::{
  collections::HashMap,
  fmt::Write,
  fs::read_to_string,
  path::{Path, PathBuf},
};

#[derive(Clone, Deserialize)]
pub struct CargoLockPackage {
  pub name: String,
  pub version: String,
  pub source: Option<String>,
}

#[derive(Deserialize)]
pub struct CargoLock {
  pub package: Vec<CargoLockPackage>,
}

#[derive(Clone, Deserialize)]
pub struct CargoManifestDependencyPackage {
  pub version: Option<String>,
  pub git: Option<String>,
  pub branch: Option<String>,
  pub rev: Option<String>,
  pub path: Option<PathBuf>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum CargoManifestDependency {
  Version(String),
  Package(CargoManifestDependencyPackage),
}

#[derive(Deserialize)]
pub struct CargoManifestPackage {
  pub version: String,
}

#[derive(Deserialize)]
pub struct CargoManifest {
  pub package: CargoManifestPackage,
  pub dependencies: HashMap<String, CargoManifestDependency>,
}

pub struct CrateVersion {
  pub version: String,
  pub found_crate_versions: Vec<String>,
}

pub fn crate_version(
  tauri_dir: &Path,
  manifest: Option<&CargoManifest>,
  lock: Option<&CargoLock>,
  name: &str,
) -> CrateVersion {
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
          format!("{version_string} (no manifest)"),
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
                let v = match read_to_string(manifest_path)
                  .map_err(|_| ())
                  .and_then(|m| toml::from_str::<CargoManifest>(&m).map_err(|_| ()))
                {
                  Ok(manifest) => manifest.package.version,
                  Err(_) => "unknown version".to_string(),
                };
                format!("path:{p:?} [{v}]")
              } else if let Some(g) = p.git {
                is_git = true;
                let mut v = format!("git:{g}");
                if let Some(branch) = p.branch {
                  let _ = write!(v, "&branch={branch}");
                } else if let Some(rev) = p.rev {
                  let _ = write!(v, "#{rev}");
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
          (Some(_lock), false) => crate_lock_packages
            .iter()
            .map(|p| p.version.clone())
            .collect::<Vec<String>>()
            .join(", "),
          (Some(_lock), true) => "unknown lockfile".to_string(),
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

  CrateVersion {
    found_crate_versions,
    version: crate_version_string,
  }
}
