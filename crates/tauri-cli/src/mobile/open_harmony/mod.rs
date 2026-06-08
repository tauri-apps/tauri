// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

use cargo_mobile2::{
  config::app::{App, DEFAULT_ASSET_DIR},
  open_harmony::{
    config::{
      Config as OpenHarmonyConfig, Metadata as OpenHarmonyMetadata, Raw as RawOpenHarmonyConfig,
    },
    device::Device,
    emulator,
    env::Env,
    hdc,
    target::Target,
  },
  opts::{FilterLevel, NoiseLevel},
  os,
  util::prompt,
};
use clap::{Parser, Subcommand};
use std::{
  env::set_var,
  fs::{create_dir_all, write},
  thread::sleep,
  time::Duration,
};
use sublime_fuzzy::best_match;
use tauri_utils::resources::ResourcePaths;

use super::{
  ensure_init, get_app, init::command as init_command, log_finished, read_options, CliOptions,
  OptionsHandle, Target as MobileTarget, MIN_DEVICE_MATCH_SCORE,
};
use crate::{
  error::{bail, Context},
  helpers::config::{BundleResources, Config as TauriConfig},
  ConfigValue, ErrorExt, Result,
};

mod build;
mod dev;
mod dev_eco_studio_script;
pub(crate) mod project;

#[derive(Deserialize)]
pub struct AppConfig {
  pub app: AppConfigObject,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigObject {
  pub bundle_name: String,
  // TODO: impl versioning
  //pub version_code: u32,
  //pub version_name: String,
}

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "OpenHarmony commands",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Debug, Parser)]
#[clap(about = "Initialize OpenHarmony target in the project")]
pub struct InitOptions {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  /// Skips installing rust toolchains via rustup
  #[clap(long)]
  skip_targets_install: bool,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json, tauri.ios.conf.json and tauri.ohos.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Dev(dev::Options),
  Build(build::Options),
  #[clap(hide(true))]
  DevEcoStudioScript(dev_eco_studio_script::Options),
}

pub fn command(cli: Cli, verbosity: u8) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => init_command(
      MobileTarget::OpenHarmony,
      options.ci,
      false,
      options.skip_targets_install,
      options.config,
    )?,
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level)?,
    Commands::DevEcoStudioScript(options) => dev_eco_studio_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: &App,
  _config: &TauriConfig,
  features: &[String],
  cli_options: &CliOptions,
) -> (OpenHarmonyConfig, OpenHarmonyMetadata) {
  let mut open_harmony_options = cli_options.clone();
  open_harmony_options.features.extend_from_slice(features);

  let raw = RawOpenHarmonyConfig {
    features: Some(open_harmony_options.features.clone()),
    logcat_filter_specs: vec![
      "RustStdoutStderr".into(),
      format!(
        "*:{}",
        match cli_options.noise_level {
          NoiseLevel::Polite => FilterLevel::Info,
          NoiseLevel::LoudAndProud => FilterLevel::Debug,
          NoiseLevel::FranklyQuitePedantic => FilterLevel::Verbose,
        }
        .logcat()
      ),
    ],
    ..Default::default()
  };
  let config = OpenHarmonyConfig::from_raw(app.clone(), Some(raw)).unwrap();

  let metadata = OpenHarmonyMetadata {
    supported: true,
    cargo_args: Some(open_harmony_options.args),
    features: Some(open_harmony_options.features),
    ..Default::default()
  };

  set_var(
    "WRY_OHOS_PACKAGE",
    app.android_identifier_escape_kotlin_keyword(),
  );
  set_var("TAURI_OHOS_PACKAGE_UNESCAPED", app.identifier());
  set_var("WRY_OHOS_LIBRARY", app.lib_name());
  set_var("TAURI_OHOS_PROJECT_PATH", config.project_dir());

  (config, metadata)
}

pub fn env() -> Result<Env> {
  let env = super::env().context("failed to setup OpenHarmony environment")?;
  cargo_mobile2::open_harmony::env::Env::from_env(env)
    .context("failed to load OpenHarmony environment")
}

fn delete_codegen_vars() {
  for (k, _) in std::env::vars() {
    if k.starts_with("WRY_") && (k.ends_with("CLASS_EXTENSION") || k.ends_with("CLASS_INIT")) {
      std::env::remove_var(k);
    }
  }
}

fn hdc_device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  let device_list =
    hdc::device_list(env).context("Failed to detect connected OpenHarmony devices")?;
  if !device_list.is_empty() {
    let device = if let Some(t) = target {
      let (device, score) = device_list
        .into_iter()
        .rev()
        .map(|d| {
          let score = best_match(t, d.name()).map_or(0, |m| m.score());
          (d, score)
        })
        .max_by_key(|(_, score)| *score)
        // we already checked the list is not empty
        .unwrap();
      if score > MIN_DEVICE_MATCH_SCORE {
        device
      } else {
        bail!("Could not find an OpenHarmony device matching {t}")
      }
    } else if device_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "OpenHarmony", " devices"),
        device_list.iter(),
        "device",
        None,
        "Device",
      )
      .context("failed to prompt for device")?;
      device_list.into_iter().nth(index).unwrap()
    } else {
      device_list.into_iter().next().unwrap()
    };

    log::info!(
      "Detected connected device: {} with target {:?}",
      device,
      device.target().triple,
    );
    Ok(device)
  } else {
    Err(crate::Error::GenericError(
      "No connected OpenHarmony devices detected".to_string(),
    ))
  }
}

fn emulator_prompt(_env: &'_ Env, target: Option<&str>) -> Result<emulator::Emulator> {
  let emulator_list = emulator::hvd_list().unwrap_or_default();
  if !emulator_list.is_empty() {
    let emulator = if let Some(t) = target {
      let (device, score) = emulator_list
        .into_iter()
        .rev()
        .map(|d| {
          let score = best_match(t, d.name()).map_or(0, |m| m.score());
          (d, score)
        })
        .max_by_key(|(_, score)| *score)
        // we already checked the list is not empty
        .unwrap();
      if score > MIN_DEVICE_MATCH_SCORE {
        device
      } else {
        bail!("Could not find an OpenHarmony Emulator matching {t}")
      }
    } else if emulator_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "OpenHarmony", " emulators"),
        emulator_list.iter(),
        "emulator",
        None,
        "Emulator",
      )
      .context("Failed to prompt for OpenHarmony Emulator device")?;
      emulator_list.into_iter().nth(index).unwrap()
    } else {
      emulator_list.into_iter().next().unwrap()
    };

    Ok(emulator)
  } else {
    Err(crate::Error::GenericError(
      "No available OpenHarmony Emulator detected".to_owned(),
    ))
  }
}

fn device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  if let Ok(device) = hdc_device_prompt(env, target) {
    Ok(device)
  } else {
    let emulator = emulator_prompt(env, target)?;
    log::info!("Starting emulator {}", emulator.name());
    emulator
      .start_detached(env)
      .context("failed to start emulator")?;
    let mut tries = 0;
    loop {
      sleep(Duration::from_secs(2));
      if let Ok(device) = hdc_device_prompt(env, Some(emulator.name())) {
        return Ok(device);
      }
      if tries >= 3 {
        log::info!("Waiting for emulator to start... (maybe the emulator is unauthorized or offline, run `hdc list targets` to check)");
      } else {
        log::info!("Waiting for emulator to start...");
      }
      tries += 1;
    }
  }
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env, None).map(|device| device.target()).ok()
}

fn open_and_wait(config: &OpenHarmonyConfig, env: &Env) -> ! {
  log::info!("Opening DevEco Studio");
  if let Err(e) = os::open_file_with("DevEco-Studio", config.project_dir(), &env.base) {
    log::error!("{e}");
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}

fn inject_resources(config: &OpenHarmonyConfig, tauri_config: &TauriConfig) -> Result<()> {
  let asset_dir = config.project_dir().join(DEFAULT_ASSET_DIR);
  create_dir_all(&asset_dir).fs_context("failed to create asset directory", asset_dir.clone())?;

  write(
    asset_dir.join("tauri.conf.json"),
    serde_json::to_string(&tauri_config).context("failed to serialize tauri config")?,
  )
  .fs_context(
    "failed to write tauri config",
    asset_dir.join("tauri.conf.json"),
  )?;

  let resources = match &tauri_config.bundle.resources {
    Some(BundleResources::List(paths)) => Some(ResourcePaths::new(paths.as_slice(), true)),
    Some(BundleResources::Map(map)) => Some(ResourcePaths::from_map(map, true)),
    None => None,
  };
  if let Some(resources) = resources {
    for resource in resources.iter() {
      let resource = resource.context("failed to get resource")?;
      let dest = asset_dir.join(resource.target());
      crate::helpers::fs::copy_file(resource.path(), dest)?;
    }
  }

  Ok(())
}
