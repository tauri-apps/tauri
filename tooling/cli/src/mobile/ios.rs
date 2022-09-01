// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  apple::{
    config::{
      Config as AppleConfig, Metadata as AppleMetadata, Platform as ApplePlatform,
      Raw as RawAppleConfig,
    },
    device::Device,
    ios_deploy, simctl,
    target::Target,
  },
  config::app::App,
  env::Env,
  opts::NoiseLevel,
  os,
  util::prompt,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init, env, get_app,
  init::{command as init_command, init_dot_cargo, Options as InitOptions},
  log_finished, read_options, CliOptions, Target as MobileTarget,
};
use crate::{
  helpers::config::{get as get_tauri_config, Config as TauriConfig},
  Result,
};

use std::{
  thread::{sleep, spawn},
  time::Duration,
};

mod build;
mod dev;
mod open;
pub(crate) mod project;
mod xcode_script;

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "iOS commands",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Open,
  Dev(dev::Options),
  Build(build::Options),
  #[clap(hide(true))]
  XcodeScript(xcode_script::Options),
}

pub fn command(cli: Cli, verbosity: usize) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Ios)?,
    Commands::Open => open::command()?,
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level)?,
    Commands::XcodeScript(options) => xcode_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: Option<App>,
  config: &TauriConfig,
  cli_options: &CliOptions,
) -> (App, AppleConfig, AppleMetadata) {
  let app = app.unwrap_or_else(|| get_app(config));
  let ios_options = cli_options.clone();

  let raw = RawAppleConfig {
    development_team: std::env::var("TAURI_APPLE_DEVELOPMENT_TEAM")
        .ok()
        .or_else(|| config.tauri.bundle.ios.development_team.clone())
        .expect("you must set `tauri > iOS > developmentTeam` config value or the `TAURI_APPLE_DEVELOPMENT_TEAM` environment variable"),
    ios_features: ios_options.features.clone(),
    bundle_version: config.package.version.clone(),
    bundle_version_short: config.package.version.clone(),
    ..Default::default()
  };
  let config = AppleConfig::from_raw(app.clone(), Some(raw)).unwrap();

  let metadata = AppleMetadata {
    supported: true,
    ios: ApplePlatform {
      cargo_args: Some(ios_options.args),
      features: ios_options.features,
      ..Default::default()
    },
    macos: Default::default(),
  };

  (app, config, metadata)
}

fn with_config<T>(
  cli_options: Option<CliOptions>,
  f: impl FnOnce(&App, &AppleConfig, &AppleMetadata, CliOptions) -> Result<T>,
) -> Result<T> {
  let (app, config, metadata, cli_options) = {
    let tauri_config = get_tauri_config(None)?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let cli_options = cli_options.unwrap_or_else(|| read_options(tauri_config_, MobileTarget::Ios));
    let (app, config, metadata) = get_config(None, tauri_config_, &cli_options);
    (app, config, metadata, cli_options)
  };
  f(&app, &config, &metadata, cli_options)
}

fn ios_deploy_device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  let device_list = ios_deploy::device_list(env)
    .map_err(|cause| anyhow::anyhow!("Failed to detect connected iOS devices: {cause}"))?;
  if !device_list.is_empty() {
    let index = if let Some(t) = target {
      let target = t.to_lowercase();
      device_list
        .iter()
        .position(|d| {
          d.name().to_lowercase().starts_with(&target)
            || d.model().to_lowercase().starts_with(&target)
        })
        .ok_or_else(|| anyhow::anyhow!("Could not find an iOS device matching {t}"))?
    } else {
      if device_list.len() > 1 {
        prompt::list(
          concat!("Detected ", "iOS", " devices"),
          device_list.iter(),
          "device",
          None,
          "Device",
        )
        .map_err(|cause| anyhow::anyhow!("Failed to prompt for iOS device: {cause}"))?
      } else {
        0
      }
    };
    let device = device_list.into_iter().nth(index).unwrap();
    println!(
      "Detected connected device: {} with target {:?}",
      device,
      device.target().triple,
    );
    Ok(device)
  } else {
    Err(anyhow::anyhow!("No connected iOS devices detected"))
  }
}

fn simulator_prompt(env: &'_ Env, target: Option<&str>) -> Result<simctl::Device> {
  let simulator_list = simctl::device_list(env).map_err(|cause| {
    anyhow::anyhow!("Failed to detect connected iOS Simulator devices: {cause}")
  })?;
  if !simulator_list.is_empty() {
    let index = if simulator_list.len() > 1 {
      if let Some(t) = target {
        let t = t.to_lowercase();
        simulator_list
          .iter()
          .position(|d| d.name().to_lowercase().starts_with(&t))
          .ok_or_else(|| anyhow::anyhow!("Could not find an iOS Simulator matching {}", t))?
      } else {
        prompt::list(
          concat!("Detected ", "iOS", " simulators"),
          simulator_list.iter(),
          "simulator",
          None,
          "Simulator",
        )
        .map_err(|cause| anyhow::anyhow!("Failed to prompt for iOS Simulator device: {cause}"))?
      }
    } else {
      0
    };
    let device = simulator_list.into_iter().nth(index).unwrap();

    log::info!("Starting simulator {}", device.name());
    let handle = device.start(env)?;
    spawn(move || {
      let _ = handle.wait();
    });

    Ok(device)
  } else {
    Err(anyhow::anyhow!("No available iOS Simulator detected"))
  }
}

fn device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  match ios_deploy_device_prompt(env, target) {
    Ok(device) => Ok(device),
    Err(e) => {
      if let Ok(simulator) = simulator_prompt(env, target) {
        let handle = simulator.start(env)?;
        spawn(move || {
          let _ = handle.wait();
        });
        Ok(simulator.into())
      } else {
        Err(e)
      }
    }
  }
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env, None).map(|device| device.target()).ok()
}

fn open_and_wait(config: &AppleConfig, env: &Env) -> ! {
  log::info!("Opening Xcode");
  if let Err(e) = os::open_file_with("Xcode", config.project_dir(), env) {
    log::error!("{}", e);
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}
