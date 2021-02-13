mod appimage_bundle;
mod category;
mod common;
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
pub mod tauri_config;
#[cfg(target_os = "windows")]
mod wix;

#[cfg(windows)]
use std::process::Command;
#[cfg(windows)]
use tauri_config::get as get_tauri_config;

pub use self::{
  common::{print_error, print_info},
  settings::{PackageType, Settings, SettingsBuilder},
};
use common::print_finished;

use std::path::PathBuf;

/// Bundles the project.
/// Returns the list of paths where the bundles can be found.
pub fn bundle_project(settings: Settings) -> crate::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  let mut package_types = settings.package_types()?;
  // The AppImage bundle script requires that the Deb bundle be run first
  if package_types.contains(&PackageType::AppImage) {
    if let Some(deb_pos) = package_types.iter().position(|&p| p == PackageType::Deb) {
      package_types.remove(deb_pos);
    }
    package_types.insert(0, PackageType::Deb);
  }
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
      PackageType::Dmg => dmg_bundle::bundle_project(&settings)?,
    };
    paths.append(&mut bundle_paths);
  }

  settings.copy_resources(settings.project_out_directory())?;
  settings.copy_binaries(settings.project_out_directory())?;

  #[cfg(windows)]
  {
    if get_tauri_config().is_ok() {
      let exempt_output = Command::new("CheckNetIsolation")
        .args(&vec!["LoopbackExempt", "-s"])
        .output()
        .expect("failed to read LoopbackExempt -s");

      if !exempt_output.status.success() {
        panic!("Failed to execute CheckNetIsolation LoopbackExempt -s");
      }

      let output_str = String::from_utf8_lossy(&exempt_output.stdout).to_lowercase();
      if !output_str.contains("win32webviewhost_cw5n1h2txyewy") {
        println!("Running Loopback command");
        runas::Command::new("powershell")
          .args(&[
            "CheckNetIsolation LoopbackExempt -a -n=\"Microsoft.Win32WebViewHost_cw5n1h2txyewy\"",
          ])
          .force_prompt(true)
          .status()
          .expect("failed to run Loopback command");
      }
    }
  }

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
