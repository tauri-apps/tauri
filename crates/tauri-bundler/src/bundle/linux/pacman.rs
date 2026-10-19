// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::debian::{generate_data, tar_and_gzip_dir};
use crate::{
  Settings,
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::fs_utils,
};
use flate2::Compression;
use heck::AsKebabCase;
use sha2::{Digest, Sha512};

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the archive.tar.gz was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // Arch Linux uses its own naming for some architectures.
  let arch = match settings.binary_arch() {
    Arch::X86_64 => "x86_64",
    Arch::X86 => "i686",
    Arch::AArch64 => "aarch64",
    Arch::Armhf => "armv7h",
    Arch::Armel => "arm",
    Arch::Riscv64 => "riscv64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };
  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    arch
  );

  let base_dir = settings.project_out_directory().join("bundle/pacman");

  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).fs_context(
      "Failed to Remove old package directory",
      package_dir.clone(),
    )?;
  }

  let package_name = format!("{package_base_name}.tar.gz");
  let package_path = base_dir.join(&package_name);
  log::info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  let (data_dir, _) =
    generate_data(settings, &package_dir).context("Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.deb().files, &data_dir)
    .context("Failed to copy custom files")?;

  // Apply tar/gzip to create the final package file.
  let package_path = tar_and_gzip_dir(package_dir.clone(), Compression::default())
    .context("Failed to create the tar.gz package")?;

  // Generate PKGBUILD file.
  generate_pkgbuild_file(settings, arch, &package_dir, &package_path)
    .context("Failed to create PKGBUILD file")?;

  Ok(vec![package_path])
}

/// Generates the pacman PKGBUILD file.
/// For more information about the format of this file, see
/// <https://wiki.archlinux.org/title/PKGBUILD>
fn generate_pkgbuild_file(
  settings: &Settings,
  arch: &str,
  dest_dir: &Path,
  package_path: &Path,
) -> crate::Result<()> {
  let pkgbuild_path = dest_dir.with_file_name("PKGBUILD");
  let mut file = fs_utils::create_file(&pkgbuild_path)?;

  let authors = settings.authors_comma_separated().unwrap_or_default();
  writeln!(file, "# Maintainer: {authors}")?;
  writeln!(file, "pkgname={}", AsKebabCase(settings.product_name()))?;
  writeln!(file, "pkgver={}", settings.version_string())?;
  writeln!(file, "pkgrel=1")?;
  writeln!(file, "epoch=")?;
  writeln!(file, "pkgdesc=\"{}\"", settings.short_description().trim())?;
  writeln!(file, "arch=('{arch}')")?;
  writeln!(
    file,
    "url=\"{}\"",
    settings.homepage_url().unwrap_or_default()
  )?;

  let pacman = settings.pacman();

  let dependencies = pacman.depends.clone().unwrap_or_default();
  writeln!(file, "depends=({})", dependencies.join(" \n"))?;

  let provides = pacman.provides.clone().unwrap_or_default();
  writeln!(file, "provides=({})", provides.join(" \n"))?;

  let conflicts = pacman.conflicts.clone().unwrap_or_default();
  writeln!(file, "conflicts=({})", conflicts.join(" \n"))?;

  let replaces = pacman.replaces.clone().unwrap_or_default();
  writeln!(file, "replaces=({})", replaces.join(" \n"))?;

  writeln!(file, "options=(!lto)")?;
  let source = pacman.source.clone().unwrap_or_default();

  if source.is_empty() {
    writeln!(file, "source=({:?})", package_path.file_name().unwrap())?;
  } else {
    writeln!(file, "source=({})", source.join(" \n"))?;
  }

  // Generate SHA512 sum of the package
  let sha_hash = hex::encode(Sha512::digest(fs::read(package_path)?));

  writeln!(file, "sha512sums=(\"{sha_hash}\")")?;
  writeln!(
    file,
    "package() {{\n\tcp -r ${{srcdir}}/data/* ${{pkgdir}}/\n}}"
  )?;

  file.flush()?;
  Ok(())
}
