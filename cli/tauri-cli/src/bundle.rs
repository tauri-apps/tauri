#[cfg(feature = "appimage")]
mod appimage_bundle;
mod category;
mod common;
mod deb_bundle;
#[cfg(feature = "dmg")]
mod dmg_bundle;
#[cfg(feature = "ios")]
mod ios_bundle;
#[cfg(target_os = "windows")]
mod msi_bundle;
mod osx_bundle;
mod path_utils;
mod rpm_bundle;
mod settings;
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
      // use dmg bundler
      // PackageType::OsxBundle => dmg_bundle::bundle_project(&settings)?,
      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => msi_bundle::bundle_project(&settings)?,
      // force appimage on linux
      // PackageType::Deb => appimage_bundle::bundle_project(&settings)?,
      PackageType::Deb => deb_bundle::bundle_project(&settings)?,
      PackageType::Rpm => rpm_bundle::bundle_project(&settings)?,
      #[cfg(feature = "appimage")]
      PackageType::AppImage => appimage_bundle::bundle_project(&settings)?,
      #[cfg(feature = "dmg")]
      PackageType::Dmg => dmg_bundle::bundle_project(&settings)?,
    });
  }

  // copy external binaries to out dir for testing
  let out_dir = settings.project_out_directory();
  for src in settings.external_binaries() {
    let src = src?;
    let dest = out_dir.join(src.file_name().expect("failed to extract external binary filename"));
    common::copy_file(&src, &dest)
      .map_err(|_| format!("Failed to copy external binary {:?}", src))?;
  }

  Ok(paths)
}
