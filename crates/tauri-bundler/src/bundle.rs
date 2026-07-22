// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod category;
mod kmp;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod platform;
mod settings;
mod updater_bundle;
mod windows;

use crate::error::ErrorExt;
use anyhow::Context;
use std::{
  fmt::Write,
  fs::File,
  io::{Seek, SeekFrom},
  path::{Path, PathBuf},
};
use tauri_utils::{display_path, platform::Target as TargetPlatform};

#[cfg(windows)]
pub use windows::vswhere_path;
pub use {
  category::AppCategory,
  settings::{
    AppImageSettings, BundleBinary, BundleSettings, CustomSignCommandSettings, DebianSettings,
    DmgSettings, Entitlements, IosSettings, MacOsSettings, NsisSettings, PackageSettings,
    PackageType, PlistKind, Position, RpmSettings, Settings, SettingsBuilder, Size,
    UpdaterSettings, WindowsSettings, WixLanguage, WixLanguageConfig, WixSettings,
  },
};

const BUNDLE_VAR_TOKEN: &[u8] = b"__TAURI_BUNDLE_TYPE_VAR_UNK";
/// Patch a binary with bundle type information
fn patch_binary(binary: &PathBuf, package_type: &PackageType) -> crate::Result<()> {
  #[cfg(target_os = "linux")]
  let bundle_type = match package_type {
    crate::PackageType::Deb => b"__TAURI_BUNDLE_TYPE_VAR_DEB",
    crate::PackageType::Rpm => b"__TAURI_BUNDLE_TYPE_VAR_RPM",
    crate::PackageType::AppImage => b"__TAURI_BUNDLE_TYPE_VAR_APP",
    // NSIS installers can be built in linux using cargo-xwin
    crate::PackageType::Nsis => b"__TAURI_BUNDLE_TYPE_VAR_NSS",
    _ => {
      return Err(crate::Error::InvalidPackageType(
        package_type.short_name().to_owned(),
        "Linux".to_owned(),
      ));
    }
  };
  #[cfg(target_os = "windows")]
  let bundle_type = match package_type {
    crate::PackageType::Nsis => b"__TAURI_BUNDLE_TYPE_VAR_NSS",
    crate::PackageType::WindowsMsi => b"__TAURI_BUNDLE_TYPE_VAR_MSI",
    _ => {
      return Err(crate::Error::InvalidPackageType(
        package_type.short_name().to_owned(),
        "Windows".to_owned(),
      ));
    }
  };
  #[cfg(target_os = "macos")]
  let bundle_type = match package_type {
    // NSIS installers can be built in macOS using cargo-xwin
    crate::PackageType::Nsis => b"__TAURI_BUNDLE_TYPE_VAR_NSS",
    crate::PackageType::MacOsBundle | crate::PackageType::Dmg => {
      // skip patching for macOS-native bundles
      return Ok(());
    }
    _ => {
      return Err(crate::Error::InvalidPackageType(
        package_type.short_name().to_owned(),
        "macOS".to_owned(),
      ));
    }
  };

  log::info!(
    "Patching {} with bundle type information: {}",
    display_path(binary),
    package_type.short_name()
  );

  let mut file_data =
    std::fs::read(binary).fs_context("failed to read binary for patching", binary.clone())?;
  let bundle_var_index =
    kmp::index_of(BUNDLE_VAR_TOKEN, &file_data).ok_or(crate::Error::MissingBundleTypeVar)?;
  file_data[bundle_var_index..bundle_var_index + BUNDLE_VAR_TOKEN.len()]
    .copy_from_slice(bundle_type);

  std::fs::write(binary, &file_data).map_err(|e| crate::Error::BinaryWriteError(e.to_string()))?;

  Ok(())
}

/// Copies `path` into a temp file so it can be restored to its pristine
/// (unsigned, unpatched) state between per-package-type bundling steps.
fn backup_file(path: &Path) -> crate::Result<(PathBuf, File)> {
  let mut copy = tempfile::tempfile().context("failed to create temp file for binary backup")?;
  let mut original =
    File::open(path).fs_context("can't open binary to back up", path.to_path_buf())?;
  std::io::copy(&mut original, &mut copy)?;
  Ok((path.to_path_buf(), copy))
}

/// Restores each backed-up file from its temp copy.
fn restore_files(backups: &mut [(PathBuf, File)]) -> crate::Result<()> {
  for (path, copy) in backups {
    let mut original = std::fs::OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(path.as_path())?;
    copy.seek(SeekFrom::Start(0))?;
    std::io::copy(&mut *copy, &mut original)?;
  }
  Ok(())
}

/// Restores the backed-up files to their pristine state when dropped, so the
/// originals are reset regardless of how a package-type iteration ends — normal
/// completion, an early `continue`, a `?` error from a signing/bundling step, or
/// a panic. A restore failure is logged rather than propagated, since it can only
/// surface while unwinding and must not mask the original error.
struct RestoreGuard<'a> {
  backups: &'a mut [(PathBuf, File)],
}

impl Drop for RestoreGuard<'_> {
  fn drop(&mut self) {
    if let Err(e) = restore_files(self.backups) {
      log::warn!("failed to restore the original binary after bundling: {e}");
    }
  }
}

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
    log::warn!(
      "Cross-platform compilation is experimental and does not support all features. Please use a matching host system for full compatibility."
    );
  }

  // Sign windows binaries before the bundling step in case neither wix and nsis bundles are enabled
  sign_binaries_if_needed(settings, target_os)?;

  let main_binary = settings.main_binary()?;
  let main_binary_path = settings.binary_path(main_binary);

  // The file that carries the `__TAURI_BUNDLE_TYPE` marker patched with the
  // bundle type. A Rust app embeds it in the main binary; a bindings app's main
  // binary is the language runtime, so the marker lives in the embedded
  // `tauri-ffi` library staged as a resource — we patch that instead.
  let patch_target = settings.bundle_type_binary()?;

  // We back up the pristine (unsigned, unpatched) copies of every file mutated in
  // the per-package-type loop so we can restore them after each step. This lets
  // us patch the marker correctly and avoids two signing issues:
  //  - modifying a signed binary without updating its PE checksum can break signature verification
  //    - codesigning tools should handle calculating+updating this, we just need to ensure
  //      (re)signing is performed after every `patch_binary()` operation
  //  - signing an already-signed binary can result in multiple signatures, causing verification errors
  // The marker patch targets `patch_target`; signing targets `main_binary_path`.
  // For a Rust app these are the same file; for a bindings app they differ, so we
  // track both. A missing `patch_target` is not fatal — `patch_binary` below just
  // warns, matching the behavior when the marker itself can't be found.
  // TODO: change this to work on a copy while preserving the originals unchanged
  let mut restore_targets = Vec::new();
  if patch_target.exists() {
    restore_targets.push(backup_file(&patch_target)?);
  }
  if patch_target != main_binary_path {
    restore_targets.push(backup_file(&main_binary_path)?);
  }

  let mut bundles = Vec::<Bundle>::new();
  for package_type in &package_types {
    // bundle was already built! e.g. DMG already built .app
    if bundles.iter().any(|b| b.package_type == *package_type) {
      continue;
    }

    if let Err(e) = patch_binary(&patch_target, package_type) {
      log::warn!(
        "Failed to add bundler type to the binary: {e}. Updater plugin may not be able to update this package. This shouldn't normally happen, please report it to https://github.com/tauri-apps/tauri/issues"
      );
    }

    // Reset the patched (and, on Windows, signed) originals to their pristine
    // state when this iteration ends, so the next package type re-finds the
    // untouched `_UNK` marker — even if a signing or bundling step below errors
    // out. Without this, a failed build would leave the original patched, and a
    // retry would find no `_UNK` to patch and silently ship the stale type.
    let _restore = RestoreGuard {
      backups: &mut restore_targets,
    };

    // sign main binary for every package type after patch
    if matches!(target_os, TargetPlatform::Windows) && settings.windows().can_sign() {
      windows::sign::try_sign(&main_binary_path, settings)?;
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
      // don't restrict to windows as NSIS installers can be built in linux+macOS using cargo-xwin
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
      log::warn!(
        "The bundler was configured to create updater artifacts but no updater-enabled targets were built. Please enable one of these targets: app, appimage, msi, nsis"
      );
    }
    if updater.v1_compatible {
      log::warn!(
        "Legacy v1 compatible updater is deprecated and will be removed in v3, change bundle > createUpdaterArtifacts to true when your users are updated to the version with v2 updater plugin"
      );
    }
  }

  #[cfg(target_os = "macos")]
  {
    // Clean up .app if only building dmg or updater
    if !package_types.contains(&PackageType::MacOsBundle)
      && let Some(app_bundle_paths) = bundles
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
      log::warn!(
        "Signing, by default, is only supported on Windows hosts, but you can specify a custom signing command in `bundler > windows > sign_command`, for now, skipping signing the installer..."
      );
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

#[cfg(test)]
mod tests {
  use super::*;

  // The marker patch mutates the original in place; the guard must reset it even
  // when the enclosing iteration exits early (an `?` error from a bundling step),
  // otherwise a retry would find no pristine `_UNK` token to patch.
  #[test]
  fn restore_guard_resets_file_when_dropped_early() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("libtauri.so");
    std::fs::write(&path, BUNDLE_VAR_TOKEN).unwrap();

    let mut backups = vec![backup_file(&path).unwrap()];
    {
      let _restore = RestoreGuard {
        backups: &mut backups,
      };
      // stand in for `patch_binary` writing a bundle type over the marker
      std::fs::write(&path, b"__TAURI_BUNDLE_TYPE_VAR_DEB").unwrap();
      assert_ne!(std::fs::read(&path).unwrap(), BUNDLE_VAR_TOKEN);
      // dropping `_restore` here mirrors an early return out of the loop body
    }

    assert_eq!(std::fs::read(&path).unwrap(), BUNDLE_VAR_TOKEN);
  }
}
