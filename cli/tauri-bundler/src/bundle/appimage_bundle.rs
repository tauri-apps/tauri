use super::{common, deb_bundle, path_utils};
use crate::Settings;

use handlebars::Handlebars;
use lazy_static::lazy_static;

use std::{
  collections::BTreeMap,
  fs::{remove_dir_all, write},
  path::PathBuf,
  process::{Command, Stdio},
};

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
  deb_bundle::generate_data(settings, &package_dir)?;

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

  // initialize shell script template.
  let temp = HANDLEBARS.render("appimage", &sh_map)?;

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
