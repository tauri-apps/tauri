// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::app;
use crate::{
  bundle::{settings::Arch, Bundle},
  utils::CommandExt,
  PackageType, Settings,
};

use std::{
  fs,
  path::PathBuf,
  process::Command,
};

pub struct Bundled {
  pub pkg: Vec<PathBuf>,
  pub app: Vec<PathBuf>,
}

/// Bundles the project into a macOS PKG installer.
/// Returns a vector of PathBuf that shows where the PKG was created.
pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Bundled> {
  // generate the .app bundle if needed
  let app_bundle_paths = if !bundles
    .iter()
    .any(|bundle| bundle.package_type == PackageType::MacOsBundle)
  {
    app::bundle_project(settings)?
  } else {
    Vec::new()
  };

  // get the target path
  let output_path = settings.project_out_directory().join("bundle/macos");
  let pkg_output_path = output_path.parent().unwrap().join("pkg");

  fs::create_dir_all(&pkg_output_path)?;

  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    match settings.binary_arch() {
      Arch::X86_64 => "x64",
      Arch::AArch64 => "aarch64",
      Arch::Universal => "universal",
      target => {
        return Err(crate::Error::ArchError(format!(
          "Unsupported architecture: {target:?}"
        )));
      }
    }
  );

  let pkg_name = format!("{}.pkg", &package_base_name);
  let pkg_path = pkg_output_path.join(&pkg_name);

  let product_name = settings.product_name();
  let bundle_file_name = format!("{product_name}.app");
  let app_bundle_path = output_path.join(&bundle_file_name);

  log::info!(action = "Bundling"; "{} ({})", pkg_name, pkg_path.display());

  // Step 1: Create a component package using pkgbuild
  // This packages the .app bundle into a component package
  let component_pkg_path = pkg_output_path.join("component.pkg");

  let mut pkgbuild_cmd = Command::new("pkgbuild");
  pkgbuild_cmd
    .arg("--component")
    .arg(&app_bundle_path)
    .arg("--install-location")
    .arg("/Applications")
    .arg(&component_pkg_path);

  log::info!(action = "Running"; "pkgbuild (component package)");
  pkgbuild_cmd
    .output_ok()
    .map_err(|e| crate::Error::ShellScriptError(format!("pkgbuild failed: {}", e)))?;

  // Step 2: Read distribution.xml from project root
  // User must provide this file for PKG bundling
  let distribution_xml_path = std::env::current_dir()?.join("distribution.xml");
  if !distribution_xml_path.exists() {
    return Err(crate::Error::GenericError(
      "distribution.xml not found in project root. PKG bundling requires a distribution.xml file.".to_string()
    ));
  }

  log::info!(action = "Using"; "distribution.xml from {}", distribution_xml_path.display());

  // Step 3: Create the distribution package using productbuild
  // This combines the component package(s) into a final installer
  let mut productbuild_cmd = Command::new("productbuild");
  productbuild_cmd
    .arg("--distribution")
    .arg(&distribution_xml_path)
    .arg("--package-path")
    .arg(&pkg_output_path)
    .arg(&pkg_path);

  log::info!(action = "Running"; "productbuild (distribution package)");
  productbuild_cmd
    .output_ok()
    .map_err(|e| crate::Error::ShellScriptError(format!("productbuild failed: {}", e)))?;

  // Sign PKG if needed
  let identity = settings.macos().signing_identity.as_deref();
  if !settings.no_sign() && identity != Some("-") {
    if let Some(identity) = identity {
      super::sign::sign_pkg(&pkg_path, identity, settings)?;
    }
  }

  log::info!(action = "Finished"; "PKG installer at {}", pkg_path.display());

  Ok(Bundled {
    pkg: vec![pkg_path],
    app: app_bundle_paths,
  })
}
