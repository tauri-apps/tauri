use super::common;
use super::settings::Settings;
use super::wix;

use std;
use std::path::PathBuf;

/// Runs all of the commands to build the MSI installer.
/// Returns a vector of PathBuf that shows where the MSI was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let wix_path = PathBuf::from("./WixTools");

  if !wix_path.exists() {
    wix::get_and_extract_wix(&wix_path)?;
  }

  let msi_path = wix::build_wix_app_installer(&settings, &wix_path)?;

  Ok(vec![msi_path])
}
