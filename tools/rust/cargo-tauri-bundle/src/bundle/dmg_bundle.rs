use super::common;
use super::osx_bundle;
use crate::Settings;

use handlebars::Handlebars;
use lazy_static::lazy_static;

use std::collections::BTreeMap;
use std::fs::write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

// Create handlebars template for shell scripts
lazy_static! {
  static ref HANDLEBARS: Handlebars = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("macos_launch", include_str!("templates/macos_launch"))
      .unwrap();

    handlebars
      .register_template_string("bundle_dmg", include_str!("templates/bundle_dmg"))
      .unwrap();
    handlebars
  };
}

// create script files to bundle project and execute bundle_script.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // generate the app.app folder
  osx_bundle::bundle_project(settings)?;

  // get uppercase string of app name
  let upcase = settings.binary_name().to_uppercase();

  // generate BTreeMap for templates
  let mut sh_map = BTreeMap::new();
  sh_map.insert("app_name", settings.binary_name());
  sh_map.insert("app_name_upcase", &upcase);

  // initialize templates
  let launch_temp = HANDLEBARS
    .render("macos_launch", &sh_map)
    .or_else(|e| Err(e.to_string()))?;
  let bundle_temp = HANDLEBARS
    .render("bundle_dmg", &sh_map)
    .or_else(|e| Err(e.to_string()))?;

  // get the target path
  let output_path = settings.project_out_directory();

  // create paths for scripts
  let launch_sh = output_path.join("macos_launch.sh");
  let bundle_sh = output_path.join("bundle_dmg.sh");

  common::print_bundling(format!("{:?}", &output_path.join(format!("{}.dmg", &upcase))).as_str())?;
  // write the scripts
  write(&launch_sh, launch_temp).or_else(|e| Err(e.to_string()))?;
  write(&bundle_sh, bundle_temp).or_else(|e| Err(e.to_string()))?;

  // execute the bundle script
  Command::new(&bundle_sh)
    .current_dir(output_path)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to execute shell script");

  Ok(vec![bundle_sh])
}
