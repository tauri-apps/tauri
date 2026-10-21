// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! DMG creation for cross-compiled macOS bundles.
//!
//! The macOS implementation drives `hdiutil` and Finder through AppleScript.
//! Neither exists on Linux, so this path writes a real HFS+ image
//! (`mkfs.hfsplus` + libdmg-hfsplus' `hfsplus`), patches the Finder metadata
//! that Finder refuses to read from anything but HFS+, and converts the result
//! to UDIF with libdmg-hfsplus' `dmg`.
//!
//! An ISO 9660 image with Apple's extensions is not enough: macOS's cd9660
//! driver does not expose the Finder flags, so the volume icon and the window
//! layout stored in `.DS_Store` are both ignored.
//!
//! Tools are resolved from the bundler cache first so CI images can ship them
//! without downloading anything while the bundler runs.

use super::super::{app, icon::create_icns_file};
use super::{Bundled, ds_store, hfs};
use crate::{
  PackageType, Settings,
  bundle::{Bundle, settings::Arch},
  error::{Context, ErrorExt},
  utils::{CommandExt, fs_utils},
};

use std::{fs, path::Path, process::Command};

const MKFS_HFSPLUS_ENV: &str = "TAURI_BUNDLER_MKFS_HFSPLUS";
const HFSPLUS_ENV: &str = "TAURI_BUNDLER_HFSPLUS";
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

  let image_path = output_path.join("dmg-staging.hfs");
  create_volume(settings, staging_path, &image_path)?;
  // The Finder flags that drive the volume icon live in the HFS+ catalog, so
  // they are patched after the files are in place.
  if let Err(error) = hfs::finalize(&image_path, volume_icon.is_some()) {
    log::warn!("failed to set the dmg volume Finder metadata: {error}");
  }

  let dmg_tool = super::super::tools::resolve(settings, "dmg", DMG_ENV)?;

  log::info!(action = "Running"; "{}", dmg_tool.display());
  Command::new(&dmg_tool)
    .arg(&image_path)
    .arg(&dmg_path)
    .output_ok()
    .with_context(|| format!("failed to run {}", dmg_tool.display()))?;

  fs::remove_file(&image_path).fs_context("failed to remove dmg staging image", &image_path)?;

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

/// Create an HFS+ volume and copy the staged files into it.
fn create_volume(settings: &Settings, staging: &Path, image_path: &Path) -> crate::Result<()> {
  let mkfs = super::super::tools::resolve(settings, "mkfs.hfsplus", MKFS_HFSPLUS_ENV)?;
  let hfsplus = super::super::tools::resolve(settings, "hfsplus", HFSPLUS_ENV)?;

  // HFS+ volumes cannot be resized without a mounted HFS+ writer, so leave
  // room for the catalog and allocation bitmaps on top of the payload.
  let payload = total_size(staging)?;
  let image_len = (payload * 12 / 10 + 16 * 1024 * 1024).max(32 * 1024 * 1024);
  fs::File::create(image_path)
    .and_then(|file| file.set_len(image_len))
    .fs_context("failed to create the dmg volume image", image_path)?;

  log::info!(action = "Running"; "{}", mkfs.display());
  Command::new(&mkfs)
    .arg("-v")
    .arg(volume_name(settings.product_name()))
    .arg(image_path)
    .output_ok()
    .with_context(|| format!("failed to run {}", mkfs.display()))?;

  copy_tree(&hfsplus, staging, image_path)?;

  Ok(())
}

/// Copy the staging tree entry by entry.
///
/// `addall` is not used: it follows symlinks (which breaks the code signature
/// seal) and drops Unix permissions. Walking the tree explicitly keeps both.
fn copy_tree(hfsplus: &Path, staging: &Path, image_path: &Path) -> crate::Result<()> {
  let mut directories = Vec::new();
  let mut files = Vec::new();
  let mut symlinks = Vec::new();

  for entry in walkdir::WalkDir::new(staging)
    .min_depth(1)
    .sort_by_file_name()
  {
    let entry = entry?;
    let path = entry.path();
    let relative = path.strip_prefix(staging)?;
    let destination = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
    if entry.file_type().is_symlink() {
      symlinks.push((
        destination,
        fs::read_link(path)?.to_string_lossy().into_owned(),
      ));
    } else if entry.file_type().is_dir() {
      directories.push(destination);
    } else if entry.file_type().is_file() {
      files.push((destination, path.to_path_buf()));
    }
  }

  for directory in directories {
    run_hfsplus(hfsplus, image_path, &["mkdir", &directory])?;
  }
  for (destination, source) in files {
    run_hfsplus(
      hfsplus,
      image_path,
      &["add", &source.to_string_lossy(), &destination],
    )?;
  }
  for (destination, target) in symlinks {
    run_hfsplus(hfsplus, image_path, &["symlink", &destination, &target])?;
  }

  // `add` copies the permission bits but not the mode, so restore the
  // executable bit for everything that had it.
  for entry in walkdir::WalkDir::new(staging).min_depth(1) {
    let entry = entry?;
    if entry.file_type().is_symlink() || !entry.file_type().is_file() {
      continue;
    }
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      if entry.metadata()?.permissions().mode() & 0o111 != 0 {
        let relative = entry.path().strip_prefix(staging)?;
        let destination = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
        run_hfsplus(hfsplus, image_path, &["chmod", "755", &destination])?;
      }
    }
  }

  Ok(())
}

fn run_hfsplus(tool: &Path, image: &Path, args: &[&str]) -> crate::Result<()> {
  Command::new(tool)
    .arg(image)
    .args(args)
    .output_ok()
    .with_context(|| format!("failed to run {} {}", tool.display(), args.join(" ")))?;
  Ok(())
}

fn total_size(directory: &Path) -> crate::Result<u64> {
  let mut total = 0;
  for entry in walkdir::WalkDir::new(directory) {
    let entry = entry?;
    if entry.file_type().is_file() {
      total += entry.metadata()?.len();
    }
  }
  Ok(total)
}

fn volume_name(product_name: &str) -> String {
  // HFS+ stores the name as UTF-16 and caps it at 255 characters.
  product_name.chars().take(255).collect()
}

#[cfg(test)]
mod tests {
  use super::{DMG_ENV, HFSPLUS_ENV, MKFS_HFSPLUS_ENV, bundle_project, volume_name};
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

  fn tooling_available() -> bool {
    // Use the same lookup the bundler uses so the cache and the env overrides
    // are honoured here too.
    let settings = settings(Path::new("."), true);
    crate::bundle::macos::tools::resolve(&settings, "mkfs.hfsplus", MKFS_HFSPLUS_ENV).is_ok()
      && crate::bundle::macos::tools::resolve(&settings, "hfsplus", HFSPLUS_ENV).is_ok()
      && crate::bundle::macos::tools::resolve(&settings, "dmg", DMG_ENV).is_ok()
  }

  #[test]
  fn volume_name_is_passed_through() {
    assert_eq!(volume_name("TestApp"), "TestApp");
    assert_eq!(volume_name("网络工具"), "网络工具");
    assert_eq!(volume_name(&"a".repeat(300)).chars().count(), 255);
  }

  #[test]
  fn builds_dmg_from_an_existing_app_bundle() {
    if !tooling_available() {
      eprintln!("skipping dmg bundling test: hfs+ or dmg tooling is missing");
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

    // The intermediate HFS+ image must not leak into the output directory.
    assert!(!dmg_path.with_file_name("dmg-staging.hfs").exists());
  }

  #[test]
  fn builds_app_bundle_and_dmg_from_scratch() {
    if !tooling_available() {
      eprintln!("skipping dmg bundling test: hfs+ or dmg tooling is missing");
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
    if !tooling_available() || which::which("dmg2img").is_err() {
      eprintln!("skipping dmg volume icon test: tooling is missing");
      return;
    }

    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());
    let app_path = project_dir.path().join("bundle/macos/Test.app");
    fs::create_dir_all(app_path.join("Contents/MacOS")).unwrap();
    let executable = app_path.join("Contents/MacOS/Test");
    fs::write(&executable, b"#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
      // Frameworks link to their siblings; the link must stay a link.
      std::os::unix::fs::symlink("Test", app_path.join("Contents/MacOS/TestLink")).unwrap();
    }
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
    assert_eq!(&bytes[1024..1026], b"H+");
    let flags = super::hfs::root_folder_flags(&bytes).unwrap();
    assert_eq!(flags & 0x0400, 0x0400);

    // `hfsplus` reports the Unix mode as a 6-digit prefix, which shows both the
    // preserved executable bit and that the symlink was not copied as a file.
    let hfsplus = crate::bundle::macos::tools::resolve(&settings, "hfsplus", HFSPLUS_ENV).unwrap();
    let listing = std::process::Command::new(hfsplus)
      .arg(&image)
      .args(["ls", "/Test.app/Contents/MacOS"])
      .output()
      .unwrap();
    let listing = String::from_utf8_lossy(&listing.stdout);
    assert!(listing.contains("100755"), "{listing}");
    assert!(listing.contains("120644"), "{listing}");
  }
}
