// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{bail, Context, ErrorExt},
  helpers::{
    app_paths::tauri_dir,
    cargo_manifest::{crate_version, CargoLock, CargoManifest},
  },
  interface::rust::get_workspace_dir,
  Result,
};

use std::{fs::read_to_string, str::FromStr};

mod migrations;

pub fn command() -> Result<()> {
  crate::helpers::app_paths::resolve();

  let tauri_dir = tauri_dir();

  let manifest_contents = read_to_string(tauri_dir.join("Cargo.toml")).fs_context(
    "failed to read Cargo manifest",
    tauri_dir.join("Cargo.toml"),
  )?;
  let manifest = toml::from_str::<CargoManifest>(&manifest_contents).with_context(|| {
    format!(
      "failed to parse Cargo manifest {}",
      tauri_dir.join("Cargo.toml").display()
    )
  })?;

  let workspace_dir = get_workspace_dir()?;
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

  let tauri_version = crate_version(tauri_dir, Some(&manifest), lock.as_ref(), "tauri")
    .version
    .context("failed to get tauri version")?;
  let tauri_version = semver::Version::from_str(&tauri_version)
    .with_context(|| format!("failed to parse tauri version {tauri_version}"))?;

  if tauri_version.major == 1 {
    migrations::v1::run().context("failed to migrate from v1")?;
  } else if tauri_version.major == 2 {
    if let Some((pre, _number)) = tauri_version.pre.as_str().split_once('.') {
      match pre {
        "beta" => {
          migrations::v2_beta::run().context("failed to migrate from v2 beta")?;
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
