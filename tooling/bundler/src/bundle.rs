// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
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
#[cfg(target_os = "windows")]
mod windows;

pub use self::{
  category::AppCategory,
  settings::{
    BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings, PackageType,
    Settings, SettingsBuilder, UpdaterSettings,
  },
};
use log::{info, warn};
pub use settings::{WindowsSettings, WixLanguage, WixLanguageConfig, WixSettings};

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
  let mut bundles = Vec::new();
  let package_types = settings.package_types()?;

  for package_type in &package_types {
    let bundle_paths = match package_type {
      #[cfg(target_os = "macos")]
      PackageType::MacOsBundle => macos::app::bundle_project(&settings)?,
      #[cfg(target_os = "macos")]
      PackageType::IosBundle => macos::ios::bundle_project(&settings)?,
      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => windows::msi::bundle_project(&settings, false)?,
      #[cfg(target_os = "linux")]
      PackageType::Deb => linux::debian::bundle_project(&settings)?,
      #[cfg(target_os = "linux")]
      PackageType::Rpm => linux::rpm::bundle_project(&settings)?,
      #[cfg(target_os = "linux")]
      PackageType::AppImage => linux::appimage::bundle_project(&settings)?,
      // dmg is dependant of MacOsBundle, we send our bundles to prevent rebuilding
      #[cfg(target_os = "macos")]
      PackageType::Dmg => macos::dmg::bundle_project(&settings, &bundles)?,
      // updater is dependant of multiple bundle, we send our bundles to prevent rebuilding
      PackageType::Updater => updater_bundle::bundle_project(&settings, &bundles)?,
      _ => {
        warn!("ignoring {:?}", package_type);
        continue;
      }
    };

    bundles.push(Bundle {
      package_type: package_type.to_owned(),
      bundle_paths,
    });
  }

  let pluralised = if bundles.len() == 1 {
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
      writeln!(printable_paths, "        {}{}", path.display(), note).unwrap();
    }
  }

  info!(action = "Finished"; "{} {} at:\n{}", bundles.len(), pluralised, printable_paths);

  Ok(bundles)
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
