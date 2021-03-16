mod appimage_bundle;
mod category;
pub mod common;
mod deb_bundle;
mod dmg_bundle;
mod ios_bundle;
#[cfg(target_os = "windows")]
mod msi_bundle;
mod osx_bundle;
mod path_utils;
mod platform;
mod rpm_bundle;
mod settings;
mod updater_bundle;
#[cfg(target_os = "windows")]
mod wix;

pub use self::{
  category::AppCategory,
  common::{print_error, print_info},
  settings::{
    BundleBinary, BundleSettings, PackageSettings, PackageType, Settings, SettingsBuilder,
    UpdaterSettings,
  },
};
use common::print_finished;

use std::path::PathBuf;

/// Bundles the project.
/// Returns the list of paths where the bundles can be found.
pub fn bundle_project(settings: Settings) -> crate::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  let package_types = settings.package_types()?;

  for package_type in &package_types {
    let mut bundle_paths = match package_type {
      PackageType::OsxBundle => {
        if package_types.clone().iter().any(|&t| t == PackageType::Dmg) {
          vec![]
        } else {
          osx_bundle::bundle_project(&settings)?
        }
      }
      PackageType::IosBundle => ios_bundle::bundle_project(&settings)?,
      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => msi_bundle::bundle_project(&settings)?,
      PackageType::Deb => deb_bundle::bundle_project(&settings)?,
      PackageType::Rpm => rpm_bundle::bundle_project(&settings)?,
      PackageType::AppImage => appimage_bundle::bundle_project(&settings)?,
      PackageType::Dmg => dmg_bundle::bundle_project(&settings, paths.clone())?,
      // we pass already existing builded app to updater
      // this way if they are already built we skip rebuilding
      PackageType::Updater => updater_bundle::bundle_project(&settings, paths.clone())?,
    };
    paths.append(&mut bundle_paths);
  }

  settings.copy_resources(settings.project_out_directory())?;
  settings.copy_binaries(settings.project_out_directory())?;

  print_finished(&paths)?;

  Ok(paths)
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
