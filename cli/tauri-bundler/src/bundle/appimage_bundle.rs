use super::common;
use super::deb_bundle;
use super::path_utils;
use crate::ResultExt;
use crate::Settings;

use handlebars::Handlebars;
use lazy_static::lazy_static;

use std::collections::BTreeMap;
use std::fs::{remove_dir_all, write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

// Create handlebars template for shell script
lazy_static! {
  static ref HANDLEBARS: Handlebars<'static> = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("appimage", include_str!("templates/appimage"))
      .expect("Failed to register template for handlebars");
    handlebars
  };
}

// bundle the project.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the deb binary name
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };
  let package_base_name = format!(
    "{}_{}_{}",
    settings.binary_name(),
    settings.version_string(),
    arch
  );
  let base_dir = settings.project_out_directory().join("bundle/deb");
  let package_dir = base_dir.join(&package_base_name);

  // generate deb_folder structure
  deb_bundle::generate_folders(settings, &package_dir)?;

  let output_path = settings.project_out_directory().join("bundle/appimage");
  if output_path.exists() {
    remove_dir_all(&output_path)
      .chain_err(|| format!("Failed to remove old {}", package_base_name))?;
  }
  std::fs::create_dir_all(output_path.clone())?;
  let app_dir_path = output_path.join(format!("{}.AppDir", settings.binary_name()));
  let appimage_path = output_path.join(format!("{}.AppImage", settings.binary_name()));
  path_utils::create(app_dir_path.clone(), true)?;

  let upcase = settings.binary_name().to_uppercase();

  // setup data to insert into shell script
  let mut sh_map = BTreeMap::new();
  sh_map.insert("app_name", settings.binary_name());
  sh_map.insert("bundle_name", package_base_name.as_str());
  sh_map.insert("app_name_uppercase", upcase.as_str());

  // initialize shell script template.
  let temp = HANDLEBARS
    .render("appimage", &sh_map)
    .or_else(|e| Err(e.to_string()))?;

  // create the shell script file in the target/ folder.
  let sh_file = output_path.join("build_appimage");
  common::print_bundling(format!("{:?}", &appimage_path).as_str())?;
  write(&sh_file, temp).or_else(|e| Err(e.to_string()))?;

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
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to execute shell script");

  Ok(vec![appimage_path])
}
