#[cfg(feature = "appimage")]
mod appimage_bundle;
mod category;
mod common;
mod deb_bundle;
#[cfg(feature = "dmg")]
mod dmg_bundle;
#[cfg(feature = "ios")]
mod ios_bundle;

mod osx_bundle;
mod path_utils;
mod rpm_bundle;
mod settings;

#[cfg(target_os = "windows")]
mod msi_bundle;
#[cfg(target_os = "windows")]
mod wix;

pub use self::common::{print_error, print_finished};
pub use self::settings::{BuildArtifact, PackageType, Settings};
use std::path::PathBuf;

pub fn bundle_project(settings: Settings) -> crate::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  for package_type in settings.package_types()? {
    paths.append(&mut match package_type {
      PackageType::OsxBundle => osx_bundle::bundle_project(&settings)?,
      #[cfg(feature = "ios")]
      PackageType::IosBundle => ios_bundle::bundle_project(&settings)?,
      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => msi_bundle::bundle_project(&settings)?,
      PackageType::Deb => deb_bundle::bundle_project(&settings)?,
      PackageType::Rpm => rpm_bundle::bundle_project(&settings)?,
      #[cfg(feature = "appimage")]
      PackageType::AppImage => appimage_bundle::bundle_project(&settings)?,
      #[cfg(feature = "dmg")]
      PackageType::Dmg => dmg_bundle::bundle_project(&settings)?,
    });
  }
  Ok(paths)
}
