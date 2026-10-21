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

use super::super::{app, icon::create_icns_file};
use super::{Bundled, apple_ext, ds_store};
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

  let volume_icon =
    create_icns_file(&output_path, settings).context("failed to create the dmg volume icon")?;
  if let Some(volume_icon) = &volume_icon {
    // `bundle_dmg.sh` also runs `SetFile -a C` on the volume root so Finder
    // picks this up. That flag lives in the root folder's Finder info, which
    // genisoimage does not emit for directories, so the file is present but
    // still needs an HFS-aware writer to take effect.
    fs_utils::copy_file(volume_icon, &staging_path.join(".VolumeIcon.icns"))
      .context("failed to copy the dmg volume icon")?;
  }

  // Without this file Finder shows a plain window with the icons in whatever
  // position it likes; it is what carries the window size, icon size, and the
  // app/Applications coordinates.
  let dmg_settings = settings.dmg();
  ds_store::write(
    staging_path,
    &ds_store::Layout {
      app_name: &bundle_file_name,
      window_size: (
        dmg_settings.window_size.width,
        dmg_settings.window_size.height,
      ),
      window_position: dmg_settings.window_position.clone(),
      app_position: dmg_settings.app_position.clone(),
      application_folder_position: dmg_settings.application_folder_position.clone(),
    },
  )?;

  // The ISO must live outside the staging directory, otherwise genisoimage
  // would include its own output in the image.
  let iso_path = output_path.join("dmg-staging.iso");
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

  if volume_icon.is_some() {
    // The flag only affects the Finder icon; a failure here should not fail
    // the whole build.
    if let Err(error) = apple_ext::set_volume_custom_icon(&iso_path) {
      log::warn!("failed to mark the dmg volume as having a custom icon: {error}");
    }
  }

  let dmg_tool = super::super::tools::resolve(settings, "dmg", DMG_ENV)?;

  log::info!(action = "Running"; "{}", dmg_tool.display());
  Command::new(&dmg_tool)
    .arg(&iso_path)
    .arg(&dmg_path)
    .output_ok()
    .with_context(|| format!("failed to run {}", dmg_tool.display()))?;

  fs::remove_file(&iso_path).fs_context("failed to remove dmg staging image", &iso_path)?;

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
  use super::{bundle_project, volume_name};
  use crate::{
    PackageType, Settings,
    bundle::{
      Bundle, BundleBinary, BundleSettings, MacOsSettings, PackageSettings, SettingsBuilder,
    },
  };
  use std::{fs, path::Path};

  fn settings(project_dir: &Path, no_sign: bool) -> Settings {
    SettingsBuilder::new()
      .project_out_directory(project_dir)
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        ..Default::default()
      })
      .target("aarch64-apple-darwin".into())
      .no_sign(no_sign)
      .build()
      .expect("failed to build settings")
  }

  fn write_binary(project_dir: &Path) {
    let executable = project_dir.join("Test");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"#!/bin/sh\n").unwrap();
  }

  fn settings_with_icon(project_dir: &Path) -> Settings {
    let icon = project_dir.join("icon.icns");
    fs::write(&icon, b"icns-not-really").unwrap();

    SettingsBuilder::new()
      .project_out_directory(project_dir)
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        icon: Some(vec![icon.to_string_lossy().into_owned()]),
        ..Default::default()
      })
      .target("aarch64-apple-darwin".into())
      .no_sign(true)
      .build()
      .expect("failed to build settings")
  }

  /// Read the `AA` entry of the volume root from an ISO 9660 image.
  fn root_apple_extension(image: &[u8]) -> Option<Vec<u8>> {
    let pvd = 16 * 2048;
    let extent = u32::from_le_bytes(image[pvd + 158..pvd + 162].try_into().unwrap()) as usize;
    let start = extent * 2048;
    let length = image[start] as usize;
    let record = &image[start..start + length];
    let name_len = record[32] as usize;
    let mut offset = 33 + name_len + usize::from(name_len.is_multiple_of(2));
    while offset + 4 <= record.len() {
      let entry_len = record[offset + 2] as usize;
      if entry_len < 4 {
        break;
      }
      if &record[offset..offset + 2] == b"AA" {
        return Some(record[offset..offset + entry_len].to_vec());
      }
      offset += entry_len;
    }
    None
  }

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

  #[test]
  fn builds_dmg_from_an_existing_app_bundle() {
    if which::which("genisoimage").is_err() || which::which("dmg").is_err() {
      eprintln!("skipping dmg bundling test: genisoimage or dmg is missing");
      return;
    }

    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());
    let app_path = project_dir.path().join("bundle/macos/Test.app");
    fs::create_dir_all(app_path.join("Contents/MacOS")).unwrap();
    fs::write(app_path.join("Contents/MacOS/Test"), b"#!/bin/sh\n").unwrap();
    let bundles = vec![Bundle {
      package_type: PackageType::MacOsBundle,
      bundle_paths: vec![app_path],
    }];

    let bundled = bundle_project(&settings(project_dir.path(), true), &bundles).unwrap();

    assert_eq!(bundled.dmg.len(), 1);
    let dmg_path = &bundled.dmg[0];
    assert_eq!(
      dmg_path.file_name().unwrap().to_string_lossy(),
      "Test_0.1.0_aarch64.dmg"
    );
    let bytes = fs::read(dmg_path).unwrap();
    assert!(!bytes.is_empty());
    // UDIF images end with the `koly` trailer.
    assert_eq!(&bytes[bytes.len() - 512..bytes.len() - 508], b"koly");

    // The intermediate ISO must not leak into the output directory.
    assert!(!dmg_path.with_file_name("dmg-staging.iso").exists());

    // The staged image must carry the Finder layout, otherwise the volume
    // opens without icon positions.
    if which::which("dmg2img").is_ok() && which::which("isoinfo").is_ok() {
      let image = project_dir.path().join("verify.img");
      let status = std::process::Command::new("dmg2img")
        .arg("-i")
        .arg(dmg_path)
        .arg("-o")
        .arg(&image)
        .output()
        .unwrap();
      assert!(status.status.success());

      let listing = std::process::Command::new("isoinfo")
        .args(["-R", "-f", "-i"])
        .arg(&image)
        .output()
        .unwrap();
      assert!(listing.status.success());
      let listing = String::from_utf8_lossy(&listing.stdout);
      assert!(
        listing.lines().any(|line| line == "/.DS_Store"),
        "{listing}"
      );
    }
  }

  #[test]
  fn builds_app_bundle_and_dmg_from_scratch() {
    if which::which("genisoimage").is_err() || which::which("dmg").is_err() {
      eprintln!("skipping dmg bundling test: genisoimage or dmg is missing");
      return;
    }

    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());

    let settings = SettingsBuilder::new()
      .project_out_directory(project_dir.path())
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        ..Default::default()
      })
      .binaries(vec![BundleBinary::with_path(
        "Test".into(),
        true,
        Some("src/main.rs".into()),
      )])
      .target("aarch64-apple-darwin".into())
      .no_sign(true)
      .build()
      .expect("failed to build settings");

    // No prebuilt bundles: the dmg path has to build the app itself.
    let bundled = bundle_project(&settings, &[]).unwrap();

    assert_eq!(bundled.app.len(), 1);
    let app_path = &bundled.app[0];
    assert!(app_path.join("Contents/Info.plist").is_file());
    assert!(app_path.join("Contents/MacOS/Test").is_file());
    assert_eq!(bundled.dmg.len(), 1);
  }

  #[test]
  fn marks_the_volume_root_when_an_icon_is_bundled() {
    if which::which("genisoimage").is_err()
      || which::which("dmg").is_err()
      || which::which("dmg2img").is_err()
    {
      eprintln!("skipping dmg volume icon test: tooling is missing");
      return;
    }

    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());
    let app_path = project_dir.path().join("bundle/macos/Test.app");
    fs::create_dir_all(app_path.join("Contents/MacOS")).unwrap();
    fs::write(app_path.join("Contents/MacOS/Test"), b"#!/bin/sh\n").unwrap();
    let bundles = vec![Bundle {
      package_type: PackageType::MacOsBundle,
      bundle_paths: vec![app_path],
    }];

    let settings = settings_with_icon(project_dir.path());
    let bundled = bundle_project(&settings, &bundles).unwrap();

    let image = project_dir.path().join("verify.img");
    let status = std::process::Command::new("dmg2img")
      .arg("-i")
      .arg(&bundled.dmg[0])
      .arg("-o")
      .arg(&image)
      .output()
      .unwrap();
    assert!(status.status.success());

    let bytes = fs::read(&image).unwrap();
    let entry = root_apple_extension(&bytes).expect("root record has no AA entry");
    assert_eq!(entry[0..2], *b"AA");
    assert_eq!(u16::from_be_bytes([entry[12], entry[13]]), 0x0400);
  }
}
