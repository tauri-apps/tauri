// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The build directory structure like this:
// foobar-1.2.3-x86_64
//         PKGBUILD # Pacman build conf

// The structure of a Pacman package looks something like this:
// foobar-1.2.3-x86_64.pkg.tar.zst  # Actually an ar archive
//         usr/bin/foobar                            # Binary executable file
//         usr/share/applications/foobar.desktop     # Desktop file (for apps)
//         usr/share/icons/hicolor/...               # Icon files (for apps)
//         usr/lib/foobar/...                        # Other resource files
//
// For cargo-bundle, we put bundle resource files under /usr/lib/package_name/,
// and then generate the desktop file and control file from the bundle
// metadata, as well as generating the md5sums file.  Currently we do not
// generate postinst or prerm files.

use super::super::common;
use crate::Settings;
use crate::bundle::linux::util;
use anyhow::Context;
use heck::ToKebabCase;
use log::info;
use std::process::Command;

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // unimplemented!();
  let arch = match settings.binary_arch() {
    "x86" => "x86",
    "x86_64" => "x86_64",
    "arm" => "arm",
    "aarch64" => "aarch64",
    _ => "any",
  };
  let pkgrel = 1;
  let package_base_name = format!(
    "{}-{}-{}-{}",
    settings.product_name().to_kebab_case().to_ascii_lowercase(),
    settings.version_string(),
    pkgrel,
    arch
  );

  let package_name = format!("{}.pkg.tar.zst", package_base_name);

  let package_dir = settings.project_out_directory().join("bundle/pacman");
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir)
      .with_context(|| format!("Failed to remove old {}", package_base_name))?;
  }
  let package_path = package_dir.join(&package_name);

  info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  copy_bin_lib(settings, &package_dir).with_context(|| "Failed to copy lib and bin file")?;
  util::generate_desktop_file(settings, &package_dir).with_context(|| "Failed to create desktop file")?;
  util::generate_icon_files(settings, &package_dir).with_context(|| "Failed to create icon file")?;
  generate_pkgbuild(settings, &package_dir, arch, pkgrel).with_context(|| "Failed to create PKGBUILD file")?;
  run_pkgbuild(&package_dir).with_context(|| "Failed to run PKGBUILD file")?;

  Ok(vec![package_path])
}

fn copy_bin_lib(settings: &Settings, package_dir: &Path) -> crate::Result<()> {
  let bin_dir = package_dir.join("usr/bin");
  let resource_dir = package_dir.join("usr/lib").join(settings.main_binary_name());

  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    common::copy_file(&bin_path, &bin_dir.join(bin.name()))
      .with_context(|| format!("Failed to copy binary from {:?}", bin_path))?;
  }

  settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;
  settings.copy_resources(&resource_dir).with_context(|| "Failed to copy resource files")?;

  Ok(())
}

fn generate_pkgbuild(settings: &Settings, package_dir: &Path, arch: &str, pkgrel: i32) -> crate::Result<()> {
  let dest_path = package_dir.join("PKGBUILD");
  let mut file = common::create_file(&dest_path)?;
  writeln!(file, "pkgname={}", settings.product_name().to_kebab_case().to_ascii_lowercase())?;
  writeln!(file, "pkgver={}", settings.version_string())?;
  writeln!(file, "pkgrel={}", pkgrel)?;
  writeln!(file, "pkgdesc=\"{}\"", settings.short_description().trim())?;
  writeln!(file, "arch=('{}')", arch)?;
  writeln!(file, "url=\"{}\"", settings.homepage_url())?;
  writeln!(file, "options=(!lto)")?;
  writeln!(file, "source=()")?;
  writeln!(file, "sha512sums=()")?;

  writeln!(file, "package() {{")?;
  writeln!(file, "  cp -r {} \"$pkgdir\"/", package_dir.join("usr").display())?;
  writeln!(file, "  chmod -R 755 \"$pkgdir\"/usr")?;
  writeln!(file, "}}")?;
  Ok(())
}

fn run_pkgbuild(package_dir: &Path) -> crate::Result<()> {
  Command::new("makepkg")
    .current_dir(package_dir)
    .output()
    .expect("failed to execute process");
  Ok(())
}

