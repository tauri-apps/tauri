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
  let tauri_version = semver::Version::from_str(&tauri_version)
    .with_context(|| format!("failed to parse tauri version {tauri_version}"))?;

  match tauri_version.major {
    1 => {
      migrations::v1::run(&dirs).context("failed to migrate from v1")?;
      migrations::v2::run(&dirs).context("failed to migrate from v2")?;
    }
    2 => match tauri_version.pre.as_str().split_once('.') {
      Some(("beta", _)) => {
        migrations::v2_beta::run(&dirs).context("failed to migrate from v2 beta")?;
        migrations::v2::run(&dirs).context("failed to migrate from v2")?;
      }
      Some(("rc", _)) | None => {
        migrations::v2::run(&dirs).context("failed to migrate from v2")?;
      }
      Some(("alpha", _)) => {
        bail!(
          "Migrating from v2 alpha ({tauri_version}) is not supported yet, \
           if your project started early, try downgrading to v1 and then try again"
        )
      }
      _ => {
        bail!("Migrating from {tauri_version} is not supported yet")
      }
    },
    _ => {
      log::info!(
        "Nothing to do, the tauri version is already at v{}",
        tauri_version.major
      );
    }
  }

  Ok(())
}
