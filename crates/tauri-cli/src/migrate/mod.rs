// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  Result,
  error::{Context, ErrorExt, bail},
  helpers::cargo_manifest::{CargoLock, CargoManifest, crate_version},
  interface::rust::get_workspace_dir,
};

use std::{fs::read_to_string, str::FromStr};

mod migrations;

pub fn command() -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();

  let manifest_contents = read_to_string(dirs.tauri.join("Cargo.toml")).fs_context(
    "failed to read Cargo manifest",
    dirs.tauri.join("Cargo.toml"),
  )?;
  let manifest = toml::from_str::<CargoManifest>(&manifest_contents).with_context(|| {
    format!(
      "failed to parse Cargo manifest {}",
      dirs.tauri.join("Cargo.toml").display()
    )
  })?;

  let workspace_dir = get_workspace_dir(dirs.tauri)?;
  let lock_path = workspace_dir.join("Cargo.lock");
  let lock = if lock_path.exists() {
    let lockfile_contents =
      read_to_string(&lock_path).fs_context("failed to read Cargo lockfile", &lock_path)?;
    let lock = toml::from_str::<CargoLock>(&lockfile_contents)
      .with_context(|| format!("failed to parse Cargo lockfile {}", lock_path.display()))?;
    Some(lock)
  } else {
    None
  };

  let tauri_version = crate_version(dirs.tauri, Some(&manifest), lock.as_ref(), "tauri")
    .version
    .context("failed to get tauri version")?;
  let tauri_version = parse_tauri_version(&tauri_version)
    .with_context(|| format!("failed to parse tauri version {tauri_version}"))?;

  if tauri_version.major == 1 {
    migrations::v1::run(&dirs).context("failed to migrate from v1")?;
  } else if tauri_version.major == 2 {
    if let Some((pre, _number)) = tauri_version.pre.as_str().split_once('.') {
      match pre {
        "beta" => {
          migrations::v2_beta::run(&dirs).context("failed to migrate from v2 beta")?;
        }
        "alpha" => {
          bail!(
            "Migrating from v2 alpha ({tauri_version}) to v2 stable is not supported yet, \
             if your project started early, try downgrading to v1 and then try again"
          )
        }
        _ => {
          bail!("Migrating from {tauri_version} to v2 stable is not supported yet")
        }
      }
    } else {
      log::info!("Nothing to do, the tauri version is already at v2 stable");
    }
  }

  Ok(())
}

/// Parses the tauri version, which is either a resolved version from the lockfile
/// or a version requirement from the manifest (e.g. `"1"` or `"^2.0.0-beta.3"`)
fn parse_tauri_version(version: &str) -> Option<semver::Version> {
  if let Ok(version) = semver::Version::from_str(version) {
    return Some(version);
  }

  let req = semver::VersionReq::from_str(version).ok()?;
  let comparator = req.comparators.first()?;
  Some(semver::Version {
    major: comparator.major,
    minor: comparator.minor.unwrap_or(0),
    patch: comparator.patch.unwrap_or(0),
    pre: comparator.pre.clone(),
    build: semver::BuildMetadata::EMPTY,
  })
}

#[cfg(test)]
mod tests {
  use super::parse_tauri_version;

  #[test]
  fn parses_lockfile_versions() {
    let version = parse_tauri_version("1.8.3").unwrap();
    assert_eq!((version.major, version.minor, version.patch), (1, 8, 3));

    let version = parse_tauri_version("2.0.0-beta.25").unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.pre.as_str(), "beta.25");
  }

  #[test]
  fn rejects_invalid_versions() {
    assert!(parse_tauri_version("*").is_none());
    assert!(parse_tauri_version("not a version").is_none());
  }

  #[test]
  fn parses_manifest_requirements() {
    assert_eq!(parse_tauri_version("1").unwrap().major, 1);
    assert_eq!(parse_tauri_version("1.8").unwrap().major, 1);
    assert_eq!(parse_tauri_version("^1.5").unwrap().major, 1);
    assert_eq!(parse_tauri_version("=1.8.3").unwrap().major, 1);
    assert_eq!(parse_tauri_version("2").unwrap().major, 2);
    assert!(parse_tauri_version("2").unwrap().pre.is_empty());

    let version = parse_tauri_version("2.0.0-beta").unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.pre.as_str(), "beta");
  }
}
