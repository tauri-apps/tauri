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

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let bundle_path = osx_bundle::bundle_project(settings)?;

  Ok(bundle_path)
}
