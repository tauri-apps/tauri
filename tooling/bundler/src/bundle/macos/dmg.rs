// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{app, icon::create_icns_file};
use crate::{
  bundle::{common::CommandExt, Bundle},
  PackageType, Settings,
};

use anyhow::Context;
use log::info;

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
    settings.main_binary_name(),
    settings.version_string(),
    match settings.binary_arch() {
      "x86_64" => "x64",
      other => other,
    }
  );
  let dmg_name = format!("{}.dmg", &package_base_name);
  let dmg_path = output_path.join(&dmg_name);

  let product_name = settings.main_binary_name();
  let bundle_file_name = format!("{}.app", product_name);
  let bundle_dir = settings.project_out_directory().join("bundle/macos");

  let support_directory_path = output_path.join("support");
  if output_path.exists() {
    fs::remove_dir_all(&output_path)
      .with_context(|| format!("Failed to remove old {}", dmg_name))?;
  }
  fs::create_dir_all(&support_directory_path).with_context(|| {
    format!(
      "Failed to create output directory at {:?}",
      support_directory_path
    )
  })?;

  // create paths for script
  let bundle_script_path = output_path.join("bundle_dmg.sh");

  info!(action = "Bundling"; "{} ({})", dmg_name, dmg_path.display());

  // write the scripts
  write(
    &bundle_script_path,
    include_str!("templates/dmg/bundle_dmg"),
  )?;
  write(
    support_directory_path.join("template.applescript"),
    include_str!("templates/dmg/template.applescript"),
  )?;
  write(
    support_directory_path.join("eula-resources-template.xml"),
    include_str!("templates/dmg/eula-resources-template.xml"),
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

  let mut args = vec![
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
  ];

  let window_position = dmg_settings.window_position.as_ref().map(|position| {
    (position.x.to_string(), position.y.to_string())
  });

  if let Some(window_position) = &window_position {
    args.push("--window-pos");
    args.push(&window_position.0);
    args.push(&window_position.1);
  }

  let background_path_string = if let Some(background_path) = &dmg_settings.background {
    Some(
      env::current_dir()?
        .join(background_path)
        .to_string_lossy()
        .to_string(),
    )
  } else {
    None
  };

  if let Some(background_path_string) = &background_path_string {
    args.push("--background");
    args.push(background_path_string);
  }

  let icns_icon_path =
    create_icns_file(&output_path, settings)?.map(|path| path.to_string_lossy().to_string());
  if let Some(icon) = &icns_icon_path {
    args.push("--volicon");
    args.push(icon);
  }

  let license_path_string = if let Some(license_path) = &settings.macos().license {
    Some(
      env::current_dir()?
        .join(license_path)
        .to_string_lossy()
        .to_string(),
    )
  } else {
    None
  };

  if let Some(license_path) = &license_path_string {
    args.push("--eula");
    args.push(license_path);
  }

  // Issue #592 - Building MacOS dmg files on CI
  // https://github.com/tauri-apps/tauri/issues/592
  if let Some(value) = env::var_os("CI") {
    if value == "true" {
      args.push("--skip-jenkins");
    }
  }

  info!(action = "Running"; "bundle_dmg.sh");

  // execute the bundle script
  Command::new(&bundle_script_path)
    .current_dir(bundle_dir.clone())
    .args(args)
    .args(vec![dmg_name.as_str(), bundle_file_name.as_str()])
    .output_ok()
    .context("error running bundle_dmg.sh")?;

  fs::rename(bundle_dir.join(dmg_name), dmg_path.clone())?;

  // Sign DMG if needed
  if let Some(identity) = &settings.macos().signing_identity {
    super::sign::sign(
      vec![super::sign::SignTarget {
        path: dmg_path.clone(),
        is_an_executable: false,
      }],
      identity,
      settings,
    )?;
  }

  Ok(Bundled {
    dmg: vec![dmg_path],
    app: app_bundle_paths,
  })
}
