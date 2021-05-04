// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  super::{common, path_utils},
  debian,
};
use crate::Settings;

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
  // prerequisite: check if mksquashfs (part of squashfs-tools) is installed
  Command::new("mksquashfs")
    .arg("-version")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .status()
    .expect("mksquashfs is not installed. Please install squashfs-tools and try again.");

  // generate the deb binary name
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };
  let package_dir = settings.project_out_directory().join("bundle/appimage_deb");

  // generate deb_folder structure
  let (_, icons) = debian::generate_data(settings, &package_dir)?;
  let icons: Vec<debian::DebIcon> = icons.into_iter().collect();

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    remove_dir_all(&output_path)?;
  }
  std::fs::create_dir_all(output_path.clone())?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.main_binary_name()));
  let appimage_filename = format!(
    "{}_{}_{}.AppImage",
    settings.main_binary_name(),
    settings.version_string(),
    arch
  );
  let appimage_path = output_path.join(&appimage_filename);
  path_utils::create(app_dir_path, true)?;

  let upcase_app_name = settings.main_binary_name().to_uppercase();

  // setup data to insert into shell script
  let mut sh_map = BTreeMap::new();
  sh_map.insert("app_name", settings.main_binary_name());
  sh_map.insert("app_name_uppercase", &upcase_app_name);
  sh_map.insert("appimage_filename", &appimage_filename);
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
  handlebars
    .register_template_string("appimage", include_str!("templates/appimage"))
    .expect("Failed to register template for handlebars");
  let temp = handlebars.render("appimage", &sh_map)?;

  // create the shell script file in the target/ folder.
  let sh_file = output_path.join("build_appimage.sh");
  common::print_bundling(&appimage_path.file_name().unwrap().to_str().unwrap())?;
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
  let mut cmd = Command::new(&sh_file);
  cmd.current_dir(output_path);

  common::execute_with_verbosity(&mut cmd, &settings).map_err(|_| {
    crate::Error::ShellScriptError(format!(
      "error running appimage.sh{}",
      if settings.is_verbose() {
        ""
      } else {
        ", try running with --verbose to see command output"
      }
    ))
  })?;

  remove_dir_all(&package_dir)?;
  Ok(vec![appimage_path])
}
