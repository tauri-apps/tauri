mod appimage_bundle;
mod category;
mod common;
mod deb_bundle;
mod dmg_bundle;
mod ios_bundle;
mod msi_bundle;
mod osx_bundle;
mod rpm_bundle;
mod settings;
mod path_utils;
mod wix;

pub use self::common::{print_error, print_finished};
pub use self::settings::{BuildArtifact, PackageType, Settings};
use std::path::PathBuf;

pub fn bundle_project(settings: Settings) -> crate::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  for package_type in settings.package_types()? {
    paths.append(&mut match package_type {
      PackageType::OsxBundle => osx_bundle::bundle_project(&settings)?,
      PackageType::IosBundle => ios_bundle::bundle_project(&settings)?,
      // use dmg bundler
      // PackageType::OsxBundle => dmg_bundle::bundle_project(&settings)?,
      PackageType::WindowsMsi => msi_bundle::bundle_project(&settings)?,
      // force appimage on linux
      // PackageType::Deb => appimage_bundle::bundle_project(&settings)?,
      PackageType::Deb => deb_bundle::bundle_project(&settings)?,
      PackageType::Rpm => rpm_bundle::bundle_project(&settings)?,
      PackageType::AppImage => appimage_bundle::bundle_project(&settings)?,
      PackageType::Dmg => dmg_bundle::bundle_project(&settings)?,
    });
  }
  Ok(paths)
}
