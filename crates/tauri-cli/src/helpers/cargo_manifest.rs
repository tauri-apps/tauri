// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

use std::{
  collections::HashMap,
  fs,
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

#[derive(Default)]
pub struct CrateVersion {
  pub version: Option<String>,
  pub git: Option<String>,
  pub git_branch: Option<String>,
  pub git_rev: Option<String>,
  pub path: Option<PathBuf>,
  pub lock_version: Option<String>,
}

impl CrateVersion {
  pub fn has_version(&self) -> bool {
    self.version.is_some() || self.git.is_some() || self.path.is_some()
  }
}

impl std::fmt::Display for CrateVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(g) = &self.git {
      if let Some(version) = &self.version {
        write!(f, "{g} ({version})")?;
      } else {
        write!(f, "git:{g}")?;
        if let Some(branch) = &self.git_branch {
          write!(f, "&branch={branch}")?;
        } else if let Some(rev) = &self.git_rev {
          write!(f, "#rev={rev}")?;
        }
      }
    } else if let Some(p) = &self.path {
      write!(f, "path:{}", p.display())?;
      if let Some(version) = &self.version {
        write!(f, " ({version})")?;
      }
    } else if let Some(version) = &self.version {
      write!(f, "{version}")?;
    } else {
      return write!(f, "No version detected");
    }

    if let Some(lock_version) = &self.lock_version {
      write!(f, " ({lock_version})")?;
    }

    Ok(())
  }
}

pub fn crate_latest_version(name: &str) -> Option<String> {
  let url = format!("https://docs.rs/crate/{name}/");
  match ureq::get(&url).call() {
    Ok(response) => match (response.status(), response.header("location")) {
      (302, Some(location)) => Some(location.replace(&url, "")),
      _ => None,
    },
    Err(_) => None,
  }
}

pub fn crate_version(
  tauri_dir: &Path,
  manifest: Option<&CargoManifest>,
  lock: Option<&CargoLock>,
  name: &str,
) -> CrateVersion {
  let mut version = CrateVersion::default();

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

  if crate_lock_packages.len() == 1 {
    let crate_lock_package = crate_lock_packages.first().unwrap();
    if let Some(s) = crate_lock_package
      .source
      .as_ref()
      .filter(|s| s.starts_with("git"))
    {
      version.git = Some(s.clone());
    }

    version.version = Some(crate_lock_package.version.clone());
  } else {
    if let Some(dep) = manifest.and_then(|m| m.dependencies.get(name).cloned()) {
      match dep {
        CargoManifestDependency::Version(v) => version.version = Some(v),
        CargoManifestDependency::Package(p) => {
          if let Some(v) = p.version {
            version.version = Some(v);
          } else if let Some(p) = p.path {
            let manifest_path = tauri_dir.join(&p).join("Cargo.toml");
            let v = fs::read_to_string(manifest_path)
              .ok()
              .and_then(|m| toml::from_str::<CargoManifest>(&m).ok())
              .map(|m| m.package.version);
            version.version = v;
            version.path = Some(p);
          } else if let Some(g) = p.git {
            version.git = Some(g);
            version.git_branch = p.branch;
            version.git_rev = p.rev;
          }
        }
      }
    }

    if lock.is_some() && crate_lock_packages.is_empty() {
      let lock_version = crate_lock_packages
        .iter()
        .map(|p| p.version.clone())
        .collect::<Vec<String>>()
        .join(", ");

      if !lock_version.is_empty() {
        version.lock_version = Some(lock_version);
      }
    }
  }

  version
}
