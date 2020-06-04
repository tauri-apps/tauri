use super::common;
use super::osx_bundle;
use crate::Settings;

use anyhow::Context;

use std::env;
use std::fs::{self, write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

// create script files to bundle project and execute bundle_script.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the app.app folder
  osx_bundle::bundle_project(settings)?;

  let app_name = settings.bundle_name();

  // get the target path
  let output_path = settings.project_out_directory().join("bundle/dmg");
  let dmg_name = format!("{}.dmg", app_name.clone());
  let dmg_path = output_path.join(&dmg_name.clone());

  let bundle_name = &format!("{}.app", app_name);
  let bundle_dir = settings.project_out_directory().join("bundle/osx");
  let bundle_path = bundle_dir.join(&bundle_name.clone());

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
  let license_script_path = support_directory_path.join("dmg-license.py");

  common::print_bundling(format!("{:?}", &dmg_path.clone()).as_str())?;

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
    &license_script_path,
    include_str!("templates/dmg/dmg-license.py"),
  )?;

  // chmod script for execution
  Command::new("chmod")
    .arg("777")
    .arg(&bundle_script_path)
    .arg(&license_script_path)
    .current_dir(output_path.clone())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to chmod script");

  let mut args = vec![
    "--volname",
    &app_name,
    "--volicon",
    "../../../../icons/icon.icns",
    "--icon",
    &bundle_name,
    "180",
    "170",
    "--app-drop-link",
    "480",
    "170",
    "--window-size",
    "660",
    "400",
    "--hide-extension",
    &bundle_name,
  ];

  if let Some(license_path) = settings.osx_license() {
    args.push("--eula");
    args.push(license_path);
  }

  // Issue #592 - Building MacOS dmg files on CI
  // https://github.com/tauri-apps/tauri/issues/592
  match env::var_os("CI") {
    Some(value) => {
      if value == "true" {
        args.push("--skip-jenkins");
      }
    }
    None => (),
  }

  // execute the bundle script
  let status = Command::new(&bundle_script_path)
    .current_dir(bundle_dir.clone())
    .args(args)
    .args(vec![dmg_name.as_str(), bundle_name.as_str()])
    .status()
    .expect("Failed to execute shell script");

  if !status.success() {
    Err(crate::Error::ShellScriptError(
      "error running bundle_dmg.sh".to_owned(),
    ))
  } else {
    fs::rename(bundle_dir.join(dmg_name.clone()), dmg_path.clone())?;
    Ok(vec![bundle_path, dmg_path])
  }
}
