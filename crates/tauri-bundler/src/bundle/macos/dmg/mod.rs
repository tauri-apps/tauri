// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{app, icon::create_icns_file};
use crate::{
  bundle::{settings::Arch, Bundle},
  utils::CommandExt,
  PackageType, Settings,
};

use anyhow::Context;

use std::{
  env,
  fs::{self, write},
  path::PathBuf,
  process::{Command, Stdio},
};

pub struct Bundled {
  pub dmg: Vec<PathBuf>,
  pub app: Vec<PathBuf>,
}

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the DMG was created.
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
  let output_path = settings.project_out_directory().join("bundle/dmg");
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
          "Unsupported architecture: {:?}",
          target
        )));
      }
    }
  );
  let dmg_name = format!("{}.dmg", &package_base_name);
  let dmg_path = output_path.join(&dmg_name);

  let product_name = settings.product_name();
  let bundle_file_name = format!("{}.app", product_name);
  let bundle_dir = settings.project_out_directory().join("bundle/macos");

  let support_directory_path = output_path
    .parent()
    .unwrap()
    .join("share/create-dmg/support");

  for path in &[&support_directory_path, &output_path] {
    if path.exists() {
      fs::remove_dir_all(path).with_context(|| format!("Failed to remove old {}", dmg_name))?;
    }
    fs::create_dir_all(path)
      .with_context(|| format!("Failed to create output directory at {:?}", path))?;
  }

  // create paths for script
  let bundle_script_path = output_path.join("bundle_dmg.sh");

  log::info!(action = "Bundling"; "{} ({})", dmg_name, dmg_path.display());

  // write the scripts
  write(&bundle_script_path, include_str!("./bundle_dmg"))?;
  write(
    support_directory_path.join("template.applescript"),
    include_str!("./template.applescript"),
  )?;
  write(
    support_directory_path.join("eula-resources-template.xml"),
    include_str!("./eula-resources-template.xml"),
  )?;

  // chmod script for execution
  Command::new("chmod")
    .arg("777")
    .arg(&bundle_script_path)
    .current_dir(&output_path)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to chmod script");

  let dmg_settings = settings.dmg();

  let app_position = &dmg_settings.app_position;
  let application_folder_position = &dmg_settings.application_folder_position;
  let window_size = &dmg_settings.window_size;

  let app_position_x = app_position.x.to_string();
  let app_position_y = app_position.y.to_string();
  let application_folder_position_x = application_folder_position.x.to_string();
  let application_folder_position_y = application_folder_position.y.to_string();
  let window_size_width = window_size.width.to_string();
  let window_size_height = window_size.height.to_string();

  let mut bundle_dmg_cmd = Command::new(&bundle_script_path);

  bundle_dmg_cmd.args([
    "--volname",
    product_name,
    "--icon",
    &bundle_file_name,
    &app_position_x,
    &app_position_y,
    "--app-drop-link",
    &application_folder_position_x,
    &application_folder_position_y,
    "--window-size",
    &window_size_width,
    &window_size_height,
    "--hide-extension",
    &bundle_file_name,
  ]);

  let window_position = dmg_settings
    .window_position
    .as_ref()
    .map(|position| (position.x.to_string(), position.y.to_string()));

  if let Some(window_position) = &window_position {
    bundle_dmg_cmd.arg("--window-pos");
    bundle_dmg_cmd.arg(&window_position.0);
    bundle_dmg_cmd.arg(&window_position.1);
  }

  let background_path = if let Some(background_path) = &dmg_settings.background {
    Some(env::current_dir()?.join(background_path))
  } else {
    None
  };

  if let Some(background_path) = &background_path {
    bundle_dmg_cmd.arg("--background");
    bundle_dmg_cmd.arg(background_path);
  }

  let icns_icon_path = create_icns_file(&output_path, settings)?;
  if let Some(icon) = &icns_icon_path {
    bundle_dmg_cmd.arg("--volicon");
    bundle_dmg_cmd.arg(icon);
  }

  let license_path = if let Some(license_path) = settings.license_file() {
    Some(env::current_dir()?.join(license_path))
  } else {
    None
  };

  if let Some(license_path) = &license_path {
    bundle_dmg_cmd.arg("--eula");
    bundle_dmg_cmd.arg(license_path);
  }

  // Issue #592 - Building MacOS dmg files on CI
  // https://github.com/tauri-apps/tauri/issues/592
  if env::var_os("TAURI_BUNDLER_DMG_IGNORE_CI").unwrap_or_default() != "true" {
    if let Some(value) = env::var_os("CI") {
      if value == "true" {
        bundle_dmg_cmd.arg("--skip-jenkins");
      }
    }
  }

  log::info!(action = "Running"; "bundle_dmg.sh");

  // execute the bundle script
  bundle_dmg_cmd
    .current_dir(bundle_dir.clone())
    .args(vec![dmg_name.as_str(), bundle_file_name.as_str()])
    .output_ok()
    .context("error running bundle_dmg.sh")?;

  fs::rename(bundle_dir.join(dmg_name), dmg_path.clone())?;

  // Sign DMG if needed
  // skipping self-signing DMGs https://github.com/tauri-apps/tauri/issues/12288
  let identity = settings.macos().signing_identity.as_deref();
  if identity != Some("-") {
    if let Some(keychain) = super::sign::keychain(identity)? {
      super::sign::sign(
        &keychain,
        vec![super::sign::SignTarget {
          path: dmg_path.clone(),
          is_an_executable: false,
        }],
        settings,
      )?;
    }
  }

  Ok(Bundled {
    dmg: vec![dmg_path],
    app: app_bundle_paths,
  })
}
