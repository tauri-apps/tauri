// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod category;
mod common;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod path_utils;
mod platform;
mod settings;
mod updater_bundle;
mod windows;

use tauri_utils::display_path;

pub use self::{
  category::AppCategory,
  settings::{
    BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings, PackageType,
    Settings, SettingsBuilder, UpdaterSettings,
  },
};
#[cfg(target_os = "macos")]
use anyhow::Context;
use log::{info, warn};
pub use settings::{NsisSettings, WindowsSettings, WixLanguage, WixLanguageConfig, WixSettings};

use std::{fmt::Write, path::PathBuf};

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
pub fn bundle_project(settings: Settings) -> crate::Result<Vec<Bundle>> {
  let mut package_types = settings.package_types()?;
  if package_types.is_empty() {
    return Ok(Vec::new());
  }

  package_types.sort_by_key(|a| a.priority());

  let mut bundles: Vec<Bundle> = Vec::new();

  let target_os = settings
    .target()
    .split('-')
    .nth(2)
    .unwrap_or(std::env::consts::OS)
    .replace("darwin", "macos");

  if target_os != std::env::consts::OS {
    warn!("Cross-platform compilation is experimental and does not support all features. Please use a matching host system for full compatibility.");
  }

  #[cfg(target_os = "windows")]
  {
    if let Some(sign_params) = settings.sign_params() {
      // Sign windows binaries before the bundling step in case neither wix and nsis bundles are enabled
      for bin in settings.binaries() {
        let bin_path = settings.binary_path(bin);
        sign_params.sign(&bin_path)?;
      }

      // Sign the sidecar binaries
      for bin in settings.external_binaries() {
        let path = bin?;
        let skip =
          std::env::var("TAURI_SKIP_SIDECAR_SIGNATURE_CHECK").map_or(false, |v| v == "true");

        if !skip && sign_params.verify(&path)? {
          info!(
            "sidecar at \"{}\" already signed. Skipping...",
            path.display()
          )
        } else {
          sign_params.sign(&path)?;
        }
      }
    }
  }

  for package_type in &package_types {
    // bundle was already built! e.g. DMG already built .app
    if bundles.iter().any(|b| b.package_type == *package_type) {
      continue;
    }

    let bundle_paths = match package_type {
      #[cfg(target_os = "macos")]
      PackageType::MacOsBundle => macos::app::bundle_project(&settings)?,
      #[cfg(target_os = "macos")]
      PackageType::IosBundle => macos::ios::bundle_project(&settings)?,
      // dmg is dependant of MacOsBundle, we send our bundles to prevent rebuilding
      #[cfg(target_os = "macos")]
      PackageType::Dmg => {
        let bundled = macos::dmg::bundle_project(&settings, &bundles)?;
        if !bundled.app.is_empty() {
          bundles.push(Bundle {
            package_type: PackageType::MacOsBundle,
            bundle_paths: bundled.app,
          });
        }
        bundled.dmg
      }

      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => windows::msi::bundle_project(&settings, false)?,
      PackageType::Nsis => windows::nsis::bundle_project(&settings, false)?,

      #[cfg(target_os = "linux")]
      PackageType::Deb => linux::debian::bundle_project(&settings)?,
      #[cfg(target_os = "linux")]
      PackageType::Rpm => linux::rpm::bundle_project(&settings)?,
      #[cfg(target_os = "linux")]
      PackageType::AppImage => linux::appimage::bundle_project(&settings)?,

      // updater is dependant of multiple bundle, we send our bundles to prevent rebuilding
      PackageType::Updater => {
        if !package_types.iter().any(|p| {
          matches!(
            p,
            PackageType::AppImage
              | PackageType::MacOsBundle
              | PackageType::Dmg
              | PackageType::Nsis
              | PackageType::WindowsMsi
          )
        }) {
          warn!("The updater bundle target exists but couldn't find any updater-enabled target, so the updater artifacts won't be generated. Please add one of these targets as well: app, appimage, msi, nsis");
          continue;
        }
        updater_bundle::bundle_project(&settings, &bundles)?
      }
      _ => {
        warn!("ignoring {}", package_type.short_name());
        continue;
      }
    };

    bundles.push(Bundle {
      package_type: package_type.to_owned(),
      bundle_paths,
    });
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
          info!(action = "Cleaning"; "{}", app_bundle_path.display());
          match app_bundle_path.is_dir() {
            true => std::fs::remove_dir_all(app_bundle_path),
            false => std::fs::remove_file(app_bundle_path),
          }
          .with_context(|| {
            format!(
              "Failed to clean the app bundle at {}",
              app_bundle_path.display()
            )
          })?
        }
      }
    }
  }

  if !bundles.is_empty() {
    let bundles_wo_updater = bundles
      .iter()
      .filter(|b| b.package_type != PackageType::Updater)
      .collect::<Vec<_>>();
    let pluralised = if bundles_wo_updater.len() == 1 {
      "bundle"
    } else {
      "bundles"
    };

    let mut printable_paths = String::new();
    for bundle in &bundles {
      for path in &bundle.bundle_paths {
        let mut note = "";
        if bundle.package_type == crate::PackageType::Updater {
          note = " (updater)";
        }
        writeln!(printable_paths, "        {}{}", display_path(path), note).unwrap();
      }
    }

    info!(action = "Finished"; "{} {} at:\n{}", bundles_wo_updater.len(), pluralised, printable_paths);

    Ok(bundles)
  } else {
    Err(anyhow::anyhow!("No bundles were built").into())
  }
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
