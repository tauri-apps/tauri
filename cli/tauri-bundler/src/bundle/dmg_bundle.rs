use super::common;
use super::osx_bundle;
use crate::Settings;

use handlebars::Handlebars;
use lazy_static::lazy_static;

use std::collections::BTreeMap;
use std::fs::{self, write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::ResultExt;

// Create handlebars template for shell scripts
lazy_static! {
  static ref HANDLEBARS: Handlebars<'static> = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("bundle_dmg", include_str!("templates/bundle_dmg"))
      .expect("Failed to setup handlebars template");
    handlebars
  };
}

// create script files to bundle project and execute bundle_script.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the app.app folder
  osx_bundle::bundle_project(settings)?;

  let app_name = settings.bundle_name();

  let sh_map: BTreeMap<(), ()> = BTreeMap::new();
  let bundle_temp = HANDLEBARS
    .render("bundle_dmg", &sh_map)
    .or_else(|e| Err(e.to_string()))?;

  // get the target path
  let output_path = settings.project_out_directory().join("bundle/dmg");
  let dmg_name = format!("{}.dmg", app_name.clone());
  let dmg_path = output_path.join(&dmg_name.clone());

  let bundle_name = &format!("{}.app", app_name);
  let bundle_dir = settings.project_out_directory().join("bundle/osx");
  let bundle_path = bundle_dir.join(&bundle_name.clone());

  let support_directory_path = output_path.join("support");
  if output_path.exists() {
    fs::remove_dir_all(&output_path).chain_err(|| format!("Failed to remove old {}", dmg_name))?;
  }
  fs::create_dir_all(&support_directory_path).chain_err(|| {
    format!(
      "Failed to create output directory at {:?}",
      support_directory_path
    )
  })?;

  // create paths for script
  let bundle_script_path = output_path.join("bundle_dmg.sh");

  common::print_bundling(format!("{:?}", &dmg_path.clone()).as_str())?;

  // write the scripts
  write(&bundle_script_path, bundle_temp).or_else(|e| Err(e.to_string()))?;

  // chmod script for execution
  Command::new("chmod")
    .arg("777")
    .arg(&bundle_script_path)
    .current_dir(output_path.clone())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to chmod script");

  let args = vec![
    "--volname",
    &app_name,
    "--volicon",
    "../../../../icons/icon.icns",
    "--app-drop-link",
    "400", "185",
    "--window-size",
    "600", "400",
    "--window-pos",
    "200", "120",
    dmg_name.as_str(),
    bundle_name.as_str(),
  ];

  // execute the bundle script
  let status = Command::new(&bundle_script_path)
    .current_dir(bundle_dir.clone())
    .args(args)
    .status()
    .expect("Failed to execute shell script");

  if !status.success() {
    Err(crate::Error::from("error running bundle_dmg.sh"))
  } else {
    fs::rename(bundle_dir.join(dmg_name.clone()), dmg_path.clone())?;
    Ok(vec![bundle_path, dmg_path])
  }
}
