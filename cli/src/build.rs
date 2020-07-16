use tauri_bundler::{
  build_project,
  bundle::{bundle_project, PackageType, SettingsBuilder},
};

use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::get as get_config,
  execute_with_output, TauriHtml,
};
use std::env::{set_current_dir, set_var};
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

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

    let index_html_path = PathBuf::from(&config.build.dist_dir).join("index.html");
    let tauri_html = TauriHtml::new(&config.build.dist_dir, read_to_string(index_html_path)?)
      .inliner_enabled(config.tauri.inliner.active)
      .generate()?;
    let tauri_index_html_path = PathBuf::from(&config.build.dist_dir).join("index.tauri.html");
    let mut tauri_index_html_file = File::create(tauri_index_html_path)?;
    tauri_index_html_file.write_all(tauri_html.as_bytes())?;

    let settings = settings_builder.build()?;

    if let Some(before_build) = &config.build.before_build_command {
      let mut cmd: Option<&str> = None;
      let mut args: Vec<&str> = vec![];
      for token in before_build.split(" ") {
        if cmd.is_none() {
          cmd = Some(token);
        } else {
          args.push(token)
        }
      }

      if let Some(cmd) = cmd {
        let mut command = Command::new(cmd);
        command.args(args).current_dir(app_dir());
        execute_with_output(&mut command)?;
      }
    }

    build_project(&settings)?;
    bundle_project(settings)?;
    Ok(())
  }
}
