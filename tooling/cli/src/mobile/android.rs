// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
    device::Device,
    emulator,
    env::Env,
    target::Target,
  },
  config::app::App,
  device::PromptError,
  opts::NoiseLevel,
  os,
  util::prompt,
};
use clap::{Parser, Subcommand};
use std::{
  env::set_var,
  thread::{sleep, spawn},
  time::Duration,
};

use super::{
  ensure_init, get_app,
  init::{command as init_command, init_dot_cargo, Options as InitOptions},
  log_finished, read_options, CliOptions, Target as MobileTarget,
};
use crate::{
  helpers::config::{get as get_tauri_config, Config as TauriConfig},
  Result,
};

mod android_studio_script;
mod build;
mod dev;
mod open;
pub(crate) mod project;

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Android commands",
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
  /// Open project in Android Studio
  Open,
  Dev(dev::Options),
  Build(build::Options),
  #[clap(hide(true))]
  AndroidStudioScript(android_studio_script::Options),
}

pub fn command(cli: Cli, verbosity: usize) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Android)?,
    Commands::Open => open::command()?,
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level)?,
    Commands::AndroidStudioScript(options) => android_studio_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: Option<App>,
  config: &TauriConfig,
  cli_options: &CliOptions,
) -> (App, AndroidConfig, AndroidMetadata) {
  let app = app.unwrap_or_else(|| get_app(config));
  let android_options = cli_options.clone();

  let raw = RawAndroidConfig {
    features: android_options.features.clone(),
    ..Default::default()
  };
  let config = AndroidConfig::from_raw(app.clone(), Some(raw)).unwrap();

  let metadata = AndroidMetadata {
    supported: true,
    cargo_args: Some(android_options.args),
    features: android_options.features,
    ..Default::default()
  };

  set_var("WRY_ANDROID_REVERSED_DOMAIN", app.reverse_domain());
  set_var("WRY_ANDROID_APP_NAME_SNAKE_CASE", app.name());
  set_var(
    "WRY_ANDROID_KOTLIN_FILES_OUT_DIR",
    config.project_dir().join("app/src/main").join(format!(
      "java/{}/{}",
      app.reverse_domain().replace('.', "/"),
      app.name()
    )),
  );

  (app, config, metadata)
}

fn with_config<T>(
  cli_options: Option<CliOptions>,
  f: impl FnOnce(&App, &AndroidConfig, &AndroidMetadata, CliOptions) -> Result<T>,
) -> Result<T> {
  let (app, config, metadata, cli_options) = {
    let tauri_config = get_tauri_config(None)?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let cli_options =
      cli_options.unwrap_or_else(|| read_options(tauri_config_, MobileTarget::Android));
    let (app, config, metadata) = get_config(None, tauri_config_, &cli_options);
    (app, config, metadata, cli_options)
  };
  f(&app, &config, &metadata, cli_options)
}

fn env() -> Result<Env> {
  let env = super::env()?;
  cargo_mobile::android::env::Env::from_env(env).map_err(Into::into)
}

fn delete_codegen_vars() {
  for (k, _) in std::env::vars() {
    if k.starts_with("WRY_") && (k.ends_with("CLASS_EXTENSION") || k.ends_with("CLASS_INIT")) {
      std::env::remove_var(k);
    }
  }
}

fn adb_device_prompt<'a>(
  env: &'_ Env,
  target: Option<&str>,
) -> Result<Device<'a>, PromptError<adb::device_list::Error>> {
  let device_list =
    adb::device_list(env).map_err(|cause| PromptError::detection_failed("Android", cause))?;
  if !device_list.is_empty() {
    let index = if device_list.len() > 1 {
      if let Some(t) = target {
        let t = t.to_lowercase();
        device_list
          .iter()
          .position(|d| d.name().to_lowercase().starts_with(&t))
          .unwrap_or_default()
      } else {
        prompt::list(
          concat!("Detected ", "Android", " devices"),
          device_list.iter(),
          "device",
          None,
          "Device",
        )
        .map_err(|cause| PromptError::prompt_failed("Android", cause))?
      }
    } else {
      0
    };
    let device = device_list.into_iter().nth(index).unwrap();
    println!(
      "Detected connected device: {} with target {:?}",
      device,
      device.target().triple,
    );
    Ok(device)
  } else {
    Err(PromptError::none_detected("Android"))
  }
}

fn emulator_prompt(
  env: &'_ Env,
  target: Option<&str>,
) -> Result<emulator::Emulator, PromptError<adb::device_list::Error>> {
  let emulator_list = emulator::avd_list(env).unwrap_or_default();
  if emulator_list.is_empty() {
    Err(PromptError::none_detected("Android emulator"))
  } else {
    let index = if emulator_list.len() > 1 {
      if let Some(t) = target {
        let t = t.to_lowercase();
        emulator_list
          .iter()
          .position(|d| d.name().to_lowercase().starts_with(&t))
          .unwrap_or_default()
      } else {
        prompt::list(
          concat!("Detected ", "Android", " emulators"),
          emulator_list.iter(),
          "emulator",
          None,
          "Emulator",
        )
        .map_err(|cause| PromptError::prompt_failed("Android emulator", cause))?
      }
    } else {
      0
    };

    Ok(emulator_list.into_iter().nth(index).unwrap())
  }
}

fn device_prompt<'a>(
  env: &'_ Env,
  target: Option<&str>,
) -> Result<Device<'a>, PromptError<adb::device_list::Error>> {
  if let Ok(device) = adb_device_prompt(env, target) {
    Ok(device)
  } else {
    let emulator = emulator_prompt(env, target)?;
    let handle = emulator.start(env).map_err(|e| {
      PromptError::prompt_failed(
        "Android emulator",
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
      )
    })?;
    spawn(move || {
      let _ = handle.wait();
    });
    loop {
      sleep(Duration::from_secs(2));
      if let Ok(device) = adb_device_prompt(env, Some(emulator.name())) {
        return Ok(device);
      }
    }
  }
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env, None).map(|device| device.target()).ok()
}

fn open_and_wait(config: &AndroidConfig, env: &Env) -> ! {
  log::info!("Opening Android Studio");
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir(), &env.base) {
    log::error!("{}", e);
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}
