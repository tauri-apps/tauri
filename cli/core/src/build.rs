use tauri_bundler::bundle::{bundle_project, PackageType, SettingsBuilder};

use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::get as get_config,
  execute_with_output,
  manifest::rewrite_manifest,
  Logger, TauriScript,
};

use std::{
  env::set_current_dir,
  fs::{rename, File},
  io::Write,
  path::PathBuf,
  process::Command,
};

mod rust;

#[derive(Default)]
pub struct Build {
  debug: bool,
  verbose: bool,
  targets: Option<Vec<String>>,
  config: Option<String>,
}

impl Build {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn debug(mut self) -> Self {
    self.debug = true;
    self
  }

  pub fn verbose(mut self) -> Self {
    self.verbose = true;
    self
  }

  pub fn targets(mut self, targets: Vec<String>) -> Self {
    self.targets = Some(targets);
    self
  }

  pub fn config(mut self, config: String) -> Self {
    self.config.replace(config);
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let logger = Logger::new("tauri:build");
    let config = get_config(self.config.as_deref())?;

    let tauri_path = tauri_dir();
    set_current_dir(&tauri_path)?;

    rewrite_manifest(config.clone())?;

    let config_guard = config.lock().unwrap();
    let config_ = config_guard.as_ref().unwrap();

    // __tauri.js
    let tauri_script = TauriScript::new()
      .global_tauri(config_.build.with_global_tauri)
      .get();
    let web_asset_path = PathBuf::from(&config_.build.dist_dir);
    if !web_asset_path.exists() {
      return Err(anyhow::anyhow!(
        "Unable to find your web assets, did you forget to build your web app? Your distDir is set to \"{:?}\".",
        web_asset_path
      ));
    }
    let tauri_script_path = web_asset_path.join("__tauri.js");
    let mut tauri_script_file = File::create(tauri_script_path)?;
    tauri_script_file.write_all(tauri_script.as_bytes())?;

    if let Some(before_build) = &config_.build.before_build_command {
      let mut cmd: Option<&str> = None;
      let mut args: Vec<&str> = vec![];
      for token in before_build.split(' ') {
        if cmd.is_none() && !token.is_empty() {
          cmd = Some(token);
        } else {
          args.push(token)
        }
      }

      if let Some(cmd) = cmd {
        logger.log(format!("Running `{}`", before_build));
        #[cfg(target_os = "windows")]
        let mut command = Command::new(
          which::which(&cmd).expect(&format!("failed to find `{}` in your $PATH", cmd)),
        );
        #[cfg(not(target_os = "windows"))]
        let mut command = Command::new(cmd);
        command.args(args).current_dir(app_dir());
        execute_with_output(&mut command)?;
      }
    }

    rust::build_project(self.debug)?;

    let app_settings = rust::AppSettings::new(&config_)?;

    let out_dir = app_settings.get_out_dir(self.debug)?;
    if let Some(product_name) = config_.package.product_name.clone() {
      let bin_name = app_settings.cargo_package_settings().name.clone();
      #[cfg(windows)]
      rename(
        out_dir.join(format!("{}.exe", bin_name)),
        out_dir.join(format!("{}.exe", product_name)),
      )?;
      #[cfg(not(windows))]
      rename(out_dir.join(bin_name), out_dir.join(product_name))?;
    }

    if config_.tauri.bundle.active {
      // move merge modules to the out dir so the bundler can load it
      #[cfg(windows)]
      {
        let (filename, vcruntime_msm) = if cfg!(target_arch = "x86") {
          let _ = std::fs::remove_file(out_dir.join("Microsoft_VC142_CRT_x64.msm"));
          (
            "Microsoft_VC142_CRT_x86.msm",
            include_bytes!("./MergeModules/Microsoft_VC142_CRT_x86.msm").to_vec(),
          )
        } else {
          let _ = std::fs::remove_file(out_dir.join("Microsoft_VC142_CRT_x86.msm"));
          (
            "Microsoft_VC142_CRT_x64.msm",
            include_bytes!("./MergeModules/Microsoft_VC142_CRT_x64.msm").to_vec(),
          )
        };
        std::fs::write(out_dir.join(filename), vcruntime_msm)?;
      }
      let mut settings_builder = SettingsBuilder::new()
        .package_settings(app_settings.get_package_settings())
        .bundle_settings(app_settings.get_bundle_settings(&config_)?)
        .binaries(app_settings.get_binaries(&config_)?)
        .project_out_directory(out_dir);

      if self.verbose {
        settings_builder = settings_builder.verbose();
      }
      if let Some(names) = self.targets {
        let mut types = vec![];
        for name in names {
          if name == "none" {
            break;
          }
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
      let settings = settings_builder.build()?;
      bundle_project(settings)?;
    }

    Ok(())
  }
}
