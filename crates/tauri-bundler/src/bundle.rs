// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod category;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod platform;
mod settings;
mod updater_bundle;
mod windows;

use tauri_utils::{display_path, platform::Target as TargetPlatform};

/// Patch a binary with bundle type information
fn patch_binary(binary: &PathBuf, package_type: &PackageType) -> crate::Result<()> {
  match package_type {
    #[cfg(target_os = "linux")]
    PackageType::AppImage | PackageType::Deb | PackageType::Rpm => {
      log::info!(
        "Patching binary {:?} for type {}",
        binary,
        package_type.short_name()
      );
      linux::patch_binary(binary, package_type)?;
    }
    PackageType::Nsis | PackageType::WindowsMsi => {
      log::info!(
        "Patching binary {:?} for type {}",
        binary,
        package_type.short_name()
      );
      windows::patch_binary(binary, package_type)?;
    }
    _ => (),
  }

  Ok(())
}

pub use self::{
  category::AppCategory,
  settings::{
    AppImageSettings, BundleBinary, BundleSettings, CustomSignCommandSettings, DebianSettings,
    DmgSettings, Entitlements, IosSettings, MacOsSettings, PackageSettings, PackageType, PlistKind,
    Position, RpmSettings, Settings, SettingsBuilder, Size, UpdaterSettings,
  },
};
pub use settings::{NsisSettings, WindowsSettings, WixLanguage, WixLanguageConfig, WixSettings};

use std::{
  fmt::Write,
  io::{Seek, SeekFrom},
  path::PathBuf,
};

/// Generated bundle metadata.
#[derive(Debug)]
pub struct Bundle {
  /// The package type.
  pub package_type: PackageType,
  /// All paths for this package.
  pub bundle_paths: Vec<PathBuf>,
}

/// Bundles the project.
/// Returns the list of paths where the bundles can be found.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<Bundle>> {
  let mut package_types = settings.package_types()?;
  if package_types.is_empty() {
    return Ok(Vec::new());
  }

  package_types.sort_by_key(|a| a.priority());

  let target_os = settings.target_platform();

  if *target_os != TargetPlatform::current() {
    log::warn!("Cross-platform compilation is experimental and does not support all features. Please use a matching host system for full compatibility.");
  }

  // Sign windows binaries before the bundling step in case neither wix and nsis bundles are enabled
  sign_binaries_if_needed(settings, target_os)?;

  let main_binary = settings
    .binaries()
    .iter()
    .find(|b| b.main())
    .expect("Main binary missing in settings");
  let main_binary_path = settings.binary_path(main_binary);

  // When packaging multiple binary types, we make a copy of the unsigned main_binary so that we can
  // restore it after each package_type step. This avoids two issues:
  //  - modifying a signed binary without updating its PE checksum can break signature verification
  //    - codesigning tools should handle calculating+updating this, we just need to ensure
  //      (re)signing is performed after every `patch_binary()` operation
  //  - signing an already-signed binary can result in multiple signatures, causing verification errors
  let main_binary_reset_required = matches!(target_os, TargetPlatform::Windows)
    && settings.windows().can_sign()
    && package_types.len() > 1;
  let mut unsigned_main_binary_copy = tempfile::tempfile()?;
  if main_binary_reset_required {
    let mut unsigned_main_binary = std::fs::File::open(&main_binary_path)?;
    std::io::copy(&mut unsigned_main_binary, &mut unsigned_main_binary_copy)?;
  }

  let mut main_binary_signed = false;
  let mut bundles = Vec::<Bundle>::new();
  for package_type in &package_types {
    // bundle was already built! e.g. DMG already built .app
    if bundles.iter().any(|b| b.package_type == *package_type) {
      continue;
    }

    if let Err(e) = patch_binary(&main_binary_path, package_type) {
      log::warn!("Failed to add bundler type to the binary: {e}. Updater plugin may not be able to update this package. This shouldn't normally happen, please report it to https://github.com/tauri-apps/tauri/issues");
    }

    // sign main binary for every package type after patch
    if matches!(target_os, TargetPlatform::Windows) && settings.windows().can_sign() {
      if main_binary_signed && main_binary_reset_required {
        let mut signed_main_binary = std::fs::OpenOptions::new()
          .write(true)
          .truncate(true)
          .open(&main_binary_path)?;
        unsigned_main_binary_copy.seek(SeekFrom::Start(0))?;
        std::io::copy(&mut unsigned_main_binary_copy, &mut signed_main_binary)?;
      }
      windows::sign::try_sign(&main_binary_path, settings)?;
      main_binary_signed = true;
    }

    let bundle_paths = match package_type {
      #[cfg(target_os = "macos")]
      PackageType::MacOsBundle => macos::app::bundle_project(settings)?,
      #[cfg(target_os = "macos")]
      PackageType::IosBundle => macos::ios::bundle_project(settings)?,
      // dmg is dependent of MacOsBundle, we send our bundles to prevent rebuilding
      #[cfg(target_os = "macos")]
      PackageType::Dmg => {
        let bundled = macos::dmg::bundle_project(settings, &bundles)?;
        if !bundled.app.is_empty() {
          bundles.push(Bundle {
            package_type: PackageType::MacOsBundle,
            bundle_paths: bundled.app,
          });
        }
        bundled.dmg
      }

      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => windows::msi::bundle_project(settings, false)?,
      // note: don't restrict to windows as NSIS installers can be built in linux using cargo-xwin
      PackageType::Nsis => windows::nsis::bundle_project(settings, false)?,

      #[cfg(target_os = "linux")]
      PackageType::Deb => linux::debian::bundle_project(settings)?,
      #[cfg(target_os = "linux")]
      PackageType::Rpm => linux::rpm::bundle_project(settings)?,
      #[cfg(target_os = "linux")]
      PackageType::AppImage => linux::appimage::bundle_project(settings)?,
      _ => {
        log::warn!("ignoring {}", package_type.short_name());
        continue;
      }
    };

    bundles.push(Bundle {
      package_type: package_type.to_owned(),
      bundle_paths,
    });
  }

  if let Some(updater) = settings.updater() {
    if package_types.iter().any(|package_type| {
      if updater.v1_compatible {
        matches!(
          package_type,
          PackageType::AppImage
            | PackageType::MacOsBundle
            | PackageType::Nsis
            | PackageType::WindowsMsi
            | PackageType::Deb
        )
      } else {
        matches!(package_type, PackageType::MacOsBundle)
      }
    }) {
      let updater_paths = updater_bundle::bundle_project(settings, &bundles)?;
      bundles.push(Bundle {
        package_type: PackageType::Updater,
        bundle_paths: updater_paths,
      });
    } else if updater.v1_compatible
      || !package_types.iter().any(|package_type| {
        // Self contained updater, no need to zip
        matches!(
          package_type,
          PackageType::AppImage | PackageType::Nsis | PackageType::WindowsMsi | PackageType::Deb
        )
      })
    {
      log::warn!("The bundler was configured to create updater artifacts but no updater-enabled targets were built. Please enable one of these targets: app, appimage, msi, nsis");
    }
    if updater.v1_compatible {
      log::warn!("Legacy v1 compatible updater is deprecated and will be removed in v3, change bundle > createUpdaterArtifacts to true when your users are updated to the version with v2 updater plugin");
    }
  }

  #[cfg(target_os = "macos")]
  {
    // Clean up .app if only building dmg or updater
    if !package_types.contains(&PackageType::MacOsBundle) {
      if let Some(app_bundle_paths) = bundles
        .iter()
        .position(|b| b.package_type == PackageType::MacOsBundle)
        .map(|i| bundles.remove(i))
        .map(|b| b.bundle_paths)
      {
        for app_bundle_path in &app_bundle_paths {
          use crate::error::ErrorExt;

          log::info!(action = "Cleaning"; "{}", app_bundle_path.display());
          match app_bundle_path.is_dir() {
            true => std::fs::remove_dir_all(app_bundle_path),
            false => std::fs::remove_file(app_bundle_path),
          }
          .fs_context(
            "failed to clean the app bundle",
            app_bundle_path.to_path_buf(),
          )?;
        }
      }
    }
  }

  if bundles.is_empty() {
    return Ok(bundles);
  }

  let finished_bundles = bundles
    .iter()
    .filter(|b| b.package_type != PackageType::Updater)
    .count();
  let pluralised = if finished_bundles == 1 {
    "bundle"
  } else {
    "bundles"
  };

  let mut printable_paths = String::new();
  for bundle in &bundles {
    for path in &bundle.bundle_paths {
      let note = if bundle.package_type == crate::PackageType::Updater {
        " (updater)"
      } else {
        ""
      };
      let path_display = display_path(path);
      writeln!(printable_paths, "        {path_display}{note}").unwrap();
    }
  }

  log::info!(action = "Finished"; "{finished_bundles} {pluralised} at:\n{printable_paths}");

  Ok(bundles)
}

fn sign_binaries_if_needed(settings: &Settings, target_os: &TargetPlatform) -> crate::Result<()> {
  if matches!(target_os, TargetPlatform::Windows) {
    if settings.windows().can_sign() {
      if settings.no_sign() {
        log::warn!("Skipping binary signing due to --no-sign flag.");
        return Ok(());
      }

      for bin in settings.binaries() {
        if bin.main() {
          // we will sign the main binary after patching per "package type"
          continue;
        }
        let bin_path = settings.binary_path(bin);
        windows::sign::try_sign(&bin_path, settings)?;
      }

      // Sign the sidecar binaries
      for bin in settings.external_binaries() {
        let path = bin?;
        let skip = std::env::var("TAURI_SKIP_SIDECAR_SIGNATURE_CHECK").is_ok_and(|v| v == "true");
        if skip {
          continue;
        }

        #[cfg(windows)]
        if windows::sign::verify(&path)? {
          log::info!(
            "sidecar at \"{}\" already signed. Skipping...",
            path.display()
          );
          continue;
        }

        windows::sign::try_sign(&path, settings)?;
      }
    } else {
      #[cfg(not(target_os = "windows"))]
      log::warn!("Signing, by default, is only supported on Windows hosts, but you can specify a custom signing command in `bundler > windows > sign_command`, for now, skipping signing the installer...");
    }
  }

  Ok(())
}

/// Check to see if there are icons in the settings struct
pub fn check_icons(settings: &Settings) -> crate::Result<bool> {
  // make a peekable iterator of the icon_files
  let mut iter = settings.icon_files().peekable();

  // if iter's first value is a None then there are no Icon files in the settings struct
  if iter.peek().is_none() {
    Ok(false)
  } else {
    Ok(true)
  }
}
