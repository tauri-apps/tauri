// Copyright 2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  super::common,
  debian::{copy_custom_files, generate_data, tar_and_gzip_dir},
};
use crate::Settings;
use anyhow::Context;
use flate2::Compression;
use heck::AsKebabCase;
use log::info;
use sha2::{Sha512, Digest};

use std::{
  fs::{self, File},
  io::{self, Write},
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the archive.tar.gz was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    // Arch Linux recognises `x86_64` and `aarch64` instead of `amd64` and `arm64` respectively.
    "x86" => "i386",
    // ARM64 is detected differently, armel isn't supported, so armhf is the only reasonable choice here.
    "arm" => "armhf",
    other => other,
  };
  let package_base_name = format!(
    "{}-{}-1-{}",
    settings.main_binary_name(),
    settings.version_string(),
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
  let package_path = tar_and_gzip_dir(package_dir.clone(), Compression::default())
    .with_context(|| "Failed to create the tar.gz package")?;

  // Generate PKGBUILD file.
  generate_pkgbuild_file(settings, arch, &package_dir, &package_path)
    .with_context(|| "Failed to create PKGBUILD file")?;

  Ok(vec![package_path])
}

/// Generates the pacman PKGBUILD file.
/// For more information about the format of this file, see
/// https://wiki.archlinux.org/title/PKGBUILD
fn generate_pkgbuild_file(settings: &Settings, arch: &str, dest_dir: &Path, package_path: &Path) -> crate::Result<()> {
  let pkgbuild_path = dest_dir.with_file_name("PKGBUILD");
  let mut file = common::create_file(&pkgbuild_path)?;

  let authors = settings.authors_comma_separated().unwrap_or_default();
  writeln!(file, "# Maintainer: {}", authors)?;
  writeln!(file, "pkgname={}", AsKebabCase(settings.product_name()))?;
  writeln!(file, "pkgver={}", settings.version_string())?;
  writeln!(file, "pkgrel=1")?;
  writeln!(file, "epoch=")?;
  writeln!(file, "pkgdesc=\"{}\"", settings.short_description().trim())?;
  writeln!(file, "arch=('{}')", arch)?;
  writeln!(file, "url=\"{}\"", settings.homepage_url())?;

  let dependencies = settings
    .pacman()
    .depends
    .as_ref()
    .cloned()
    .unwrap_or_default();
  writeln!(file, "depends=({})", dependencies.join(" \n"))?;

  let provides = settings
    .pacman()
    .provides
    .as_ref()
    .cloned()
    .unwrap_or_default();
  writeln!(file, "provides=({})", provides.join(" \n"))?;

  let conflicts = settings
    .pacman()
    .conflicts
    .as_ref()
    .cloned()
    .unwrap_or_default();
  writeln!(file, "conflicts=({})", conflicts.join(" \n"))?;

  let replaces = settings
    .pacman()
    .replaces
    .as_ref()
    .cloned()
    .unwrap_or_default();
  writeln!(file, "replaces=({})", replaces.join(" \n"))?;

  writeln!(file, "options=(!lto)")?;
  writeln!(file, "source=({:?})", package_path.file_name().unwrap())?;

  // Generate SHA512 sum of the package
  let mut sha_file = File::open(package_path)?;
  let mut sha512 = Sha512::new();
  io::copy(&mut sha_file, &mut sha512)?;
  let sha_hash = sha512.finalize();

  writeln!(file, "sha512sums=(\"{:x}\")", sha_hash)?;
  writeln!(file, "package() {{\n\tcp -r ${{srcdir}}/data/* ${{pkgdir}}/\n}}")?;
  
  file.flush()?;
  Ok(())
}
