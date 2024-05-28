// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  super::{
    common::{self, CommandExt},
    path_utils,
  },
  debian,
};
use crate::Settings;
use anyhow::Context;
use handlebars::Handlebars;
use std::{
  collections::BTreeMap,
  fs::{remove_dir_all, write},
  path::PathBuf,
  process::{Command, Stdio},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the AppImage was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the deb binary name
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };
  let package_dir = settings.project_out_directory().join("bundle/appimage_deb");

  // generate deb_folder structure
  let (data_dir, icons) = debian::generate_data(settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  common::copy_custom_files(&settings.deb().files, &data_dir)
    .with_context(|| "Failed to copy custom files")?;

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    remove_dir_all(&output_path)?;
  }
  std::fs::create_dir_all(output_path.clone())?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.product_name()));
  let appimage_filename = format!(
    "{}_{}_{}.AppImage",
    settings.product_name(),
    settings.version_string(),
    arch
  );
  let appimage_path = output_path.join(&appimage_filename);
  path_utils::create(app_dir_path, true)?;

  let upcase_app_name = settings.product_name().to_uppercase();

  // setup data to insert into shell script
  let mut sh_map = BTreeMap::new();
  sh_map.insert("arch", settings.target().split('-').next().unwrap());
  sh_map.insert("app_name", settings.product_name());
  sh_map.insert("app_name_uppercase", &upcase_app_name);
  sh_map.insert("appimage_filename", &appimage_filename);
  let tauri_tools_path = dirs_next::cache_dir().map_or_else(
    || output_path.to_path_buf(),
    |mut p| {
      p.push("tauri");
      p
    },
  );
  std::fs::create_dir_all(&tauri_tools_path)?;
  let tauri_tools_path_str = tauri_tools_path.to_string_lossy();
  sh_map.insert("tauri_tools_path", &tauri_tools_path_str);
  let larger_icon = icons
    .iter()
    .filter(|i| i.width == i.height)
    .max_by_key(|i| i.width)
    .expect("couldn't find a square icon to use as AppImage icon");
  let larger_icon_path = larger_icon
    .path
    .strip_prefix(package_dir.join("data"))
    .unwrap()
    .to_string_lossy()
    .to_string();
  sh_map.insert("icon_path", &larger_icon_path);

  // initialize shell script template.
  let mut handlebars = Handlebars::new();
  handlebars.register_escape_fn(handlebars::no_escape);
  handlebars
    .register_template_string("appimage", include_str!("templates/appimage"))
    .expect("Failed to register template for handlebars");
  let temp = handlebars.render("appimage", &sh_map)?;

  // create the shell script file in the target/ folder.
  let sh_file = output_path.join("build_appimage.sh");

  log::info!(action = "Bundling"; "{} ({})", appimage_filename, appimage_path.display());

  write(&sh_file, temp)?;

  // chmod script for execution
  Command::new("chmod")
    .arg("777")
    .arg(&sh_file)
    .current_dir(output_path.clone())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to chmod script");

  // execute the shell script to build the appimage.
  Command::new(&sh_file)
    .current_dir(output_path)
    .output_ok()
    .context("error running build_appimage.sh")?;

  remove_dir_all(&package_dir)?;
  Ok(vec![appimage_path])
}
