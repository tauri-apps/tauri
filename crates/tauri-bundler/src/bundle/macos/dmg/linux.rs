// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! DMG creation for cross-compiled macOS bundles.
//!
//! The macOS implementation drives `hdiutil` and Finder through AppleScript.
//! Neither exists on Linux, so this path creates an HFS/ISO hybrid with
//! `genisoimage` and converts it to UDIF with libdmg-hfsplus' `dmg` tool.
//!
//! Both tools are resolved from the bundler cache first so CI images can ship
//! them without downloading anything while the bundler runs.

use super::super::app;
use super::Bundled;
use crate::{
  PackageType, Settings,
  bundle::{Bundle, settings::Arch},
  error::{Context, ErrorExt},
  utils::{CommandExt, fs_utils},
};

use std::{fs, process::Command};

const GENISOIMAGE_ENV: &str = "TAURI_BUNDLER_GENISOIMAGE";
const DMG_ENV: &str = "TAURI_BUNDLER_DMG";

pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Bundled> {
  let app_bundle_paths = if !bundles
    .iter()
    .any(|bundle| bundle.package_type == PackageType::MacOsBundle)
  {
    app::bundle_project(settings)?
  } else {
    Vec::new()
  };

  let output_path = settings.project_out_directory().join("bundle/dmg");
  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    match settings.binary_arch() {
      Arch::X86_64 => "x64",
      Arch::AArch64 => "aarch64",
      Arch::Universal => "universal",
      target => {
        return Err(crate::Error::ArchError(format!(
          "Unsupported architecture: {target:?}"
        )));
      }
    }
  );
  let dmg_name = format!("{package_base_name}.dmg");
  let dmg_path = output_path.join(&dmg_name);
  let bundle_file_name = format!("{}.app", settings.product_name());
  let app_path = settings
    .project_out_directory()
    .join("bundle/macos")
    .join(&bundle_file_name);

  if !app_path.is_dir() {
    return Err(crate::Error::GenericError(format!(
      "failed to find app bundle at {}",
      app_path.display()
    )));
  }

  if output_path.exists() {
    fs::remove_dir_all(&output_path).fs_context("failed to remove old dmg", &output_path)?;
  }
  fs::create_dir_all(&output_path).fs_context("failed to create output directory", &output_path)?;

  if settings.dmg().background.is_some() {
    log::warn!("DMG background images are not supported on Linux; using the default background");
  }
  if settings.license_file().is_some() {
    log::warn!("DMG license/EULA is not supported on Linux");
  }

  log::info!(action = "Bundling"; "{} ({})", dmg_name, dmg_path.display());

  let staging = tempfile::tempdir_in(&output_path)
    .fs_context("failed to create dmg staging directory", &output_path)?;
  let staging_path = staging.path();

  fs_utils::copy_dir(&app_path, &staging_path.join(&bundle_file_name))
    .context("failed to copy app bundle into dmg staging directory")?;

  // The /Applications link is what makes the mounted image drag-installable.
  std::os::unix::fs::symlink("/Applications", staging_path.join("Applications")).fs_context(
    "failed to create Applications link",
    staging_path.join("Applications"),
  )?;

  let iso_path = staging_path.join("dmg-staging.iso");
  let genisoimage = super::super::tools::resolve(settings, "genisoimage", GENISOIMAGE_ENV)?;

  log::info!(action = "Running"; "{}", genisoimage.display());
  Command::new(&genisoimage)
    .args(["-no-cache-inodes", "-D", "-l", "-probe", "-V"])
    .arg(volume_name(settings.product_name()))
    .args(["-no-pad", "-r", "-dir-mode", "0755", "-apple", "-o"])
    .arg(&iso_path)
    .arg(staging_path)
    .output_ok()
    .with_context(|| format!("failed to run {}", genisoimage.display()))?;

  let dmg_tool = super::super::tools::resolve(settings, "dmg", DMG_ENV)?;

  log::info!(action = "Running"; "{}", dmg_tool.display());
  Command::new(&dmg_tool)
    .arg(&iso_path)
    .arg(&dmg_path)
    .output_ok()
    .with_context(|| format!("failed to run {}", dmg_tool.display()))?;

  if !dmg_path.is_file() {
    return Err(crate::Error::GenericError(format!(
      "dmg tool did not create {}",
      dmg_path.display()
    )));
  }

  Ok(Bundled {
    dmg: vec![dmg_path],
    app: app_bundle_paths,
  })
}

fn volume_name(product_name: &str) -> String {
  // ISO 9660 volume identifiers are limited to 32 bytes.
  let mut name = String::new();
  for character in product_name.chars() {
    if name.len() + character.len_utf8() > 32 {
      break;
    }
    name.push(character);
  }
  name
}

#[cfg(test)]
mod tests {
  use super::volume_name;

  #[test]
  fn volume_name_is_limited_to_iso_bytes() {
    assert_eq!(volume_name("TestApp"), "TestApp");
    assert_eq!(
      volume_name("a-very-long-product-name-that-exceeds-the-limit"),
      "a-very-long-product-name-that-ex"
    );
    assert_eq!(
      volume_name("网络工具网络工具网络工具网络工具网络工具"),
      "网络工具网络工具网络"
    );
  }
}
