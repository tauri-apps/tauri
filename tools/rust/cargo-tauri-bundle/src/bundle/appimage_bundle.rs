use super::common;
use super::deb_bundle;
use crate::Settings;

use handlebars::Handlebars;
use lazy_static::lazy_static;

use std::collections::BTreeMap;
use std::fs::write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

lazy_static! {
  static ref HANDLEBARS: Handlebars = {
    let mut handlebars = Handlebars::new();

    handlebars
      .register_template_string("appimage", include_str!("templates/appimage"))
      .unwrap();
    handlebars
  };
}

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // execute deb bundler
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
  deb_bundle::bundle_project(&settings)?;

  let upcase = settings.binary_name().to_uppercase();
  let mut sh_map = BTreeMap::new();
  sh_map.insert("app_name", settings.binary_name());
  sh_map.insert("bundle_name", package_base_name.as_str());
  sh_map.insert("app_name_uppercase", upcase.as_str());

  let temp = HANDLEBARS
    .render("appimage", &sh_map)
    .or_else(|e| Err(e.to_string()))?;
  let output_path = settings.project_out_directory();

  let sh_file = output_path.join("build_appimage");
  common::print_bundling(
    format!(
      "{:?}",
      &output_path.join(format!("{}.AppImage", settings.binary_name()))
    )
    .as_str(),
  )?;
  write(&sh_file, temp).or_else(|e| Err(e.to_string()))?;

  Command::new(&sh_file)
    .current_dir(output_path)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .expect("Failed to execute shell script");

  Ok(vec![sh_file])
}
