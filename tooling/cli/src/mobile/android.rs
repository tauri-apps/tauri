// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
    device::{Device, RunError},
    env::{Env, Error as AndroidEnvError},
    target::{BuildError, Target},
  },
  config::app::App,
  device::PromptError,
  env::Error as EnvError,
  opts::NoiseLevel,
  os,
  util::prompt,
};
use clap::{Parser, Subcommand};
use std::env::set_var;

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

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  EnvInitFailed(EnvError),
  #[error(transparent)]
  AndroidEnvInitFailed(AndroidEnvError),
  #[error(transparent)]
  InitDotCargo(super::init::Error),
  #[error("invalid tauri configuration: {0}")]
  InvalidTauriConfig(String),
  #[error("{0}")]
  ProjectNotInitialized(String),
  #[error(transparent)]
  OpenFailed(os::OpenFileError),
  #[error("{0}")]
  DevFailed(String),
  #[error("{0}")]
  BuildFailed(String),
  #[error(transparent)]
  AndroidStudioScriptFailed(BuildError),
  #[error(transparent)]
  RunFailed(RunError),
  #[error("{0}")]
  TargetInvalid(String),
  #[error(transparent)]
  FailedToPromptForDevice(PromptError<adb::device_list::Error>),
}

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
  config: &TauriConfig,
  cli_options: &CliOptions,
) -> (App, AndroidConfig, AndroidMetadata) {
  let app = get_app(config);
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
  f: impl FnOnce(&App, &AndroidConfig, &AndroidMetadata, CliOptions) -> Result<T, Error>,
) -> Result<T, Error> {
  let (app, config, metadata, cli_options) = {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let cli_options =
      cli_options.unwrap_or_else(|| read_options(tauri_config_, MobileTarget::Android));
    let (app, config, metadata) = get_config(tauri_config_, &cli_options);
    (app, config, metadata, cli_options)
  };
  f(&app, &config, &metadata, cli_options)
}

fn env() -> Result<Env, Error> {
  let env = super::env().map_err(Error::EnvInitFailed)?;
  cargo_mobile::android::env::Env::from_env(env).map_err(Error::AndroidEnvInitFailed)
}

fn delete_codegen_vars() {
  for (k, _) in std::env::vars() {
    if k.starts_with("WRY_") && (k.ends_with("CLASS_EXTENSION") || k.ends_with("CLASS_INIT")) {
      std::env::remove_var(k);
    }
  }
}

fn device_prompt<'a>(env: &'_ Env) -> Result<Device<'a>, PromptError<adb::device_list::Error>> {
  let device_list =
    adb::device_list(env).map_err(|cause| PromptError::detection_failed("Android", cause))?;
  if !device_list.is_empty() {
    let index = if device_list.len() > 1 {
      prompt::list(
        concat!("Detected ", "Android", " devices"),
        device_list.iter(),
        "device",
        None,
        "Device",
      )
      .map_err(|cause| PromptError::prompt_failed("Android", cause))?
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

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env).map(|device| device.target()).ok()
}

fn open_and_wait(config: &AndroidConfig, env: &Env) -> ! {
  log::info!("Opening Android Studio");
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir(), &env.base) {
    log::error!("{}", e);
  }
  loop {
    std::thread::sleep(std::time::Duration::from_secs(24 * 60 * 60));
  }
}
