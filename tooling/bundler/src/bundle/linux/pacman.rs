// Copyright 2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{super::common, debian::{generate_data, tar_and_gzip_dir, copy_custom_files}};
use crate::Settings;
use anyhow::Context;
use log::info;
use flate2::Compression;

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Not implemented yet.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    // ARM64 is detected differently, armel isn't supported, so armhf is the only reasonable choice here.
    "arm" => "armhf",
    "aarch64" => "arm64",
    other => other,
  };
let pkgrel = 1;
  let package_base_name = format!(
    "{}-{}-{}-{}",
    settings.main_binary_name(),
    settings.version_string(),
    pkgrel,
    arch
  );

  let base_dir = settings.project_out_directory().join("bundle/pacman");

  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir)
      .with_context(|| format!("Failed to remove old {}", package_base_name))?;
  }

  let package_name = format!("{}.tar.gz", package_base_name);

  let package_path = base_dir.join(&package_name);

  info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  let (data_dir, _) = generate_data(settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  copy_custom_files(settings, &data_dir).with_context(|| "Failed to copy custom files")?;

  // Apply tar/gzip to create the final package file.
  let package_path =
    tar_and_gzip_dir(package_dir, Compression::default()).with_context(|| "Failed to tar/gzip control directory")?;

  Ok(vec![package_path])
}

/// Generates the pacman PKGBUILD file.
fn generate_pkgbuild_file(
  settings: &Settings,
  arch: &str,
  dest_path: &Path,
  data_dir: &Path,
) -> crate::Result<()> {
  // For more information about the format of this file, see
  // https://wiki.archlinux.org/title/PKGBUILD
  let mut file = common::create_file(&dest_path)?;

  let authors = settings.authors_comma_separated().unwrap_or_default();
  writeln!(file, "# Maintainer: {}", authors)?;

  file.flush()?;
  Ok(())
}
