// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::tauri_dir,
    cargo_manifest::{crate_version, CargoLock, CargoManifest},
  },
  interface::rust::get_workspace_dir,
  Result,
};

use std::{fs::read_to_string, str::FromStr};

use anyhow::Context;

mod v1;

pub fn command() -> Result<()> {
  let tauri_dir = tauri_dir();

  let manifest_contents =
    read_to_string(tauri_dir.join("Cargo.toml")).context("failed to read Cargo manifest")?;
  let manifest = toml::from_str::<CargoManifest>(&manifest_contents)
    .context("failed to parse Cargo manifest")?;

  let workspace_dir = get_workspace_dir()?;
  let lock_path = workspace_dir.join("Cargo.lock");
  let lock = if lock_path.exists() {
    let lockfile_contents = read_to_string(lock_path).context("failed to read Cargo lockfile")?;
    let lock =
      toml::from_str::<CargoLock>(&lockfile_contents).context("failed to parse Cargo lockfile")?;
    Some(lock)
  } else {
    None
  };

  let tauri_version = crate_version(&tauri_dir, Some(&manifest), lock.as_ref(), "tauri").version;
  let tauri_version = semver::Version::from_str(&tauri_version)?;

  if tauri_version.major == 1 {
    v1::run().context("failed to migrate from v1")?;
  }

  Ok(())
}
