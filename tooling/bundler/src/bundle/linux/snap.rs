// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use handlebars::Handlebars;
use log::info;

use crate::Settings;

use std::{path::PathBuf, path::Path, process::Command, collections::BTreeMap, fs::{write, self}};

/// Bundles the project with snapcraft
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let release_folder = settings.project_out_directory();
  let target_folder = release_folder.clone().parent().expect("Unable to find target folder in your project");
  let snap_folder = target_folder.clone().join("snap");
  info!("Generating yaml files to target/snap folder...");
  generate_yaml(&settings, &snap_folder).expect("Failed to generate yaml files.");
  //check for snapcraft on system
  check_snapcraft().expect("Snaft cannot be found on your system. Please install Snapcraft");
  info!("Running 'snapcraft' bundler...");
  let snapcraft_output = run_snapcraft(&target_folder).expect("Error running Snapcraft").wait();
  info!("✅ Successfully bundled snap package in target folder!");
  let snap_file = fs::read_dir(&target_folder)
    .expect("Unable to read target folder")
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.path().extension().unwrap_or_default() == "snap")
    .max_by_key(|entry| entry.metadata().expect("Unable to get metadata").modified()
    .expect("Unable to get modified date"))
    .expect("Unable to find snap file in target folder")
    .path();
  Ok(vec![snap_file])
}

fn generate_yaml(settings: &Settings, snap_folder: &PathBuf) -> crate::Result<()> {
  let mut sh_map = BTreeMap::new();
  sh_map.insert("package-name", settings.main_binary_name());
  sh_map.insert("package-summary", settings.short_description());
  if let Some(long_description) = settings.long_description() {
    sh_map.insert("package-description", long_description);
  }
  else {
    sh_map.insert("package-description", settings.short_description());
  }
  sh_map.insert("app-name", settings.main_binary_name());
  sh_map.insert("version", settings.version_string());
  let mut handlebars = Handlebars::new();
  handlebars
    .register_template_string("snapcraft", include_str!("templates/snapcore22"))
    .expect("Failed to register template for handlebars");
  let temp = handlebars.render("snapcraft", &sh_map)?;

  // create the yaml file in snap/ folder.
  fs::create_dir_all(&snap_folder)?;
  let sh_file = snap_folder.join("snapcraft.yaml");
  write(&sh_file, temp)?;
  return Ok(());
}


//Checks if the user has snapcraft installed or not
fn check_snapcraft() -> Result<std::process::Output, std::io::Error> {
  let output = Command::new("snapcraft")
        .arg("--version")
        .output();
      output
}

fn run_snapcraft(target_folder: &Path) -> Result<std::process::Child, std::io::Error> {
  let output = Command::new("/bin/sh")
      .current_dir(&target_folder)
      .arg("-c")
      .arg("snapcraft")
      .spawn();
  output
}
