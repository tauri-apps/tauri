use tauri_bundler::{
  build_project,
  bundle::{bundle_project, PackageType, SettingsBuilder},
};

use crate::helpers::{app_paths::tauri_dir, config::get as get_config};
use std::env::{set_current_dir, set_var};

#[derive(Default)]
pub struct Build {
  debug: bool,
  targets: Option<Vec<String>>,
}

impl Build {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn debug(mut self) -> Self {
    self.debug = true;
    self
  }

  pub fn targets(mut self, targets: Vec<String>) -> Self {
    self.targets = Some(targets);
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let config = get_config()?;
    let feature = if config.tauri.embedded_server.active {
      "embedded-server"
    } else {
      "no-server"
    };

    let mut settings_builder = SettingsBuilder::new().features(vec![feature.to_string()]);
    if !self.debug {
      settings_builder = settings_builder.release();
    }
    if let Some(names) = self.targets {
      let mut types = vec![];
      for name in names {
        match PackageType::from_short_name(&name) {
          Some(package_type) => {
            types.push(package_type);
          }
          None => {
            return Err(anyhow::anyhow!(format!(
              "Unsupported bundle format: {}",
              name
            )));
          }
        }
      }
      settings_builder = settings_builder.package_types(types);
    }

    let tauri_path = tauri_dir();
    set_current_dir(&tauri_path)?;
    set_var("TAURI_DIR", &tauri_path);
    set_var("TAURI_DIST_DIR", tauri_path.join(&config.build.dist_dir));

    let settings = settings_builder.build()?;

    build_project(&settings)?;
    bundle_project(settings)?;
    Ok(())
  }
}
