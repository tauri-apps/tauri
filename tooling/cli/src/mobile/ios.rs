// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  apple::{
    config::{Config as AppleConfig, Metadata as AppleMetadata},
    device::{Device, RunError},
    ios_deploy,
    target::{CompileLibError, Target},
  },
  config::Config,
  device::PromptError,
  env::{Env, Error as EnvError},
  os, util,
  util::prompt,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init, env_vars, get_config,
  init::{command as init_command, init_dot_cargo, Options as InitOptions},
  Target as MobileTarget,
};
use crate::{helpers::config::get as get_tauri_config, Result};

use std::path::PathBuf;

mod build;
mod dev;
mod open;
pub(crate) mod project;
mod xcode_script;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  EnvInitFailed(EnvError),
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
  NoHomeDir(util::NoHomeDir),
  #[error("SDK root provided by Xcode was invalid. {sdk_root} doesn't exist or isn't a directory")]
  SdkRootInvalid { sdk_root: PathBuf },
  #[error("Include dir was invalid. {include_dir} doesn't exist or isn't a directory")]
  IncludeDirInvalid { include_dir: PathBuf },
  #[error("macOS SDK root was invalid. {macos_sdk_root} doesn't exist or isn't a directory")]
  MacosSdkRootInvalid { macos_sdk_root: PathBuf },
  #[error("Arch specified by Xcode was invalid. {arch} isn't a known arch")]
  ArchInvalid { arch: String },
  #[error(transparent)]
  CompileLibFailed(CompileLibError),
  #[error(transparent)]
  FailedToPromptForDevice(PromptError<ios_deploy::DeviceListError>),
  #[error(transparent)]
  RunFailed(RunError),
  #[error("{0}")]
  TargetInvalid(String),
}

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

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Ios)?,
    Commands::Open => open::command()?,
    Commands::Dev(options) => dev::command(options)?,
    Commands::Build(options) => build::command(options)?,
    Commands::XcodeScript(options) => xcode_script::command(options)?,
  }

  Ok(())
}

fn with_config<T>(
  f: impl FnOnce(&Config, &AppleConfig, &AppleMetadata) -> Result<T, Error>,
) -> Result<T, Error> {
  let (config, metadata) = {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(tauri_config_)
  };
  f(&config, config.apple(), metadata.apple())
}

fn env() -> Result<Env, Error> {
  let env = Env::new()
    .map_err(Error::EnvInitFailed)?
    .explicit_env_vars(env_vars());
  Ok(env)
}

fn device_prompt<'a>(env: &'_ Env) -> Result<Device<'a>, PromptError<ios_deploy::DeviceListError>> {
  let device_list =
    ios_deploy::device_list(env).map_err(|cause| PromptError::detection_failed("iOS", cause))?;
  if !device_list.is_empty() {
    let index = if device_list.len() > 1 {
      prompt::list(
        concat!("Detected ", "iOS", " devices"),
        device_list.iter(),
        "device",
        None,
        "Device",
      )
      .map_err(|cause| PromptError::prompt_failed("iOS", cause))?
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
    Err(PromptError::none_detected("iOS"))
  }
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env).map(|device| device.target()).ok()
}
