// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata},
    device::Device,
    env::{Env, Error as EnvError},
    target::{BuildError, Target},
  },
  device::PromptError,
  opts::{NoiseLevel, Profile},
  os,
  target::call_for_targets_with_fallback,
  util::prompt,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init, get_config, get_metadata,
  init::{command as init_command, Options as InitOptions},
  Target as MobileTarget,
};
use crate::helpers::config::get as get_tauri_config;
use crate::Result;

pub(crate) mod project;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  EnvInitFailed(EnvError),
  #[error("invalid tauri configuration: {0}")]
  InvalidTauriConfig(String),
  #[error("{0}")]
  ProjectNotInitialized(String),
  #[error(transparent)]
  OpenFailed(os::OpenFileError),
  #[error(transparent)]
  BuildFailed(BuildError),
  #[error("{0}")]
  TargetInvalid(String),
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

#[derive(Debug, Parser)]
pub struct BuildOptions {
  /// Targets to build.
  #[clap(
    short,
    long = "target",
    multiple_occurrences(true),
    multiple_values(true),
    value_parser(clap::builder::PossibleValuesParser::new(["aarch64", "armv7", "i686", "x86_64"]))
  )]
  targets: Option<Vec<String>>,
  /// Builds with the debug flag
  #[clap(short, long)]
  debug: bool,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  /// Open project in Android Studio
  Open,
  #[clap(hide(true))]
  Build(BuildOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Android)?,
    Commands::Open => open()?,
    Commands::Build(options) => build(options)?,
  }

  Ok(())
}

fn with_config(
  f: impl FnOnce(&AndroidConfig, &AndroidMetadata) -> Result<(), Error>,
) -> Result<(), Error> {
  let config = get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();
  let mobile_config = get_config(config_);
  let metadata = get_metadata(config_);
  f(mobile_config.android(), metadata.android())
}

fn open() -> Result<()> {
  with_config(|config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Android Studio", config.project_dir()).map_err(Error::OpenFailed)
  })?;
  Ok(())
}

fn build(options: BuildOptions) -> Result<()> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  fn device_prompt<'a>(env: &'_ Env) -> Result<Device<'a>, PromptError<adb::device_list::Error>> {
    let device_list =
      adb::device_list(env).map_err(|cause| PromptError::detection_failed("Android", cause))?;
    if device_list.len() > 0 {
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

  with_config(|config, metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = Env::new().map_err(Error::EnvInitFailed)?;

    call_for_targets_with_fallback(
      options.targets.unwrap_or_default().iter(),
      &detect_target_ok,
      &env,
      |target: &Target| {
        target
          .build(
            config,
            metadata,
            &env,
            NoiseLevel::Polite,
            true.into(),
            profile,
          )
          .map_err(Error::BuildFailed)
      },
    )
    .map_err(|e| Error::TargetInvalid(e.to_string()))?
  })?;
  Ok(())
}
