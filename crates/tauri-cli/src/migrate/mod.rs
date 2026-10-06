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

// The version comes from the lockfile when available, otherwise from the manifest,
// where it may be a requirement such as `1` or `^1.5`.
fn parse_tauri_version(version: &str) -> Result<semver::Version> {
  if let Ok(version) = semver::Version::from_str(version) {
    return Ok(version);
  }

  let comparator = semver::VersionReq::parse(version)
    .ok()
    .and_then(|req| req.comparators.into_iter().next())
    .with_context(|| format!("failed to parse tauri version {version}"))?;

  Ok(semver::Version {
    major: comparator.major,
    minor: comparator.minor.unwrap_or(0),
    patch: comparator.patch.unwrap_or(0),
    pre: comparator.pre,
    build: semver::BuildMetadata::EMPTY,
  })
}

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
  let tauri_version = parse_tauri_version(&tauri_version)?;

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

#[cfg(test)]
mod tests {
  use super::parse_tauri_version;

  #[test]
  fn parses_complete_versions() {
    assert_eq!(parse_tauri_version("1.8.3").unwrap().to_string(), "1.8.3");
    assert_eq!(
      parse_tauri_version("2.0.0-beta.12").unwrap().to_string(),
      "2.0.0-beta.12"
    );
  }

  #[test]
  fn parses_version_requirements() {
    for (input, expected) in [
      ("1", "1.0.0"),
      ("1.8", "1.8.0"),
      ("^1.5", "1.5.0"),
      ("~1.5.2", "1.5.2"),
      ("=1.0.0", "1.0.0"),
      (">=1.2, <2", "1.2.0"),
      ("1.*", "1.0.0"),
      ("2", "2.0.0"),
      ("^2.0.0-beta.12", "2.0.0-beta.12"),
    ] {
      assert_eq!(
        parse_tauri_version(input).unwrap().to_string(),
        expected,
        "input: {input}"
      );
    }
  }

  #[test]
  fn rejects_invalid_versions() {
    for input in ["", "*", "garbage"] {
      assert!(parse_tauri_version(input).is_err(), "input: {input}");
    }
  }
}
