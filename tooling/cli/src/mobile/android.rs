// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata},
    device::{Device, RunError},
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
  ensure_init, get_config,
  init::{command as init_command, Options as InitOptions},
  write_options, CliOptions, DevChild, Target as MobileTarget,
};
use crate::{
  helpers::config::get as get_tauri_config,
  interface::{DevProcess, Interface, MobileOptions},
  Result,
};

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
  #[error("{0}")]
  DevFailed(String),
  #[error(transparent)]
  BuildFailed(BuildError),
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
  /// Builds with the release flag
  #[clap(short, long)]
  release: bool,
}

#[derive(Debug, Clone, Parser)]
#[clap(about = "Android dev")]
pub struct DevOptions {
  /// List of cargo features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// Exit on panic
  #[clap(short, long)]
  exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
}

impl From<DevOptions> for crate::dev::Options {
  fn from(options: DevOptions) -> Self {
    Self {
      runner: None,
      target: None,
      features: options.features,
      exit_on_panic: options.exit_on_panic,
      config: options.config,
      release_mode: false,
      args: Vec::new(),
      no_watch: options.no_watch,
    }
  }
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  /// Open project in Android Studio
  Open,
  Dev(DevOptions),
  #[clap(hide(true))]
  Build(BuildOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Android)?,
    Commands::Open => open()?,
    Commands::Build(options) => build(options)?,
    Commands::Dev(options) => dev(options)?,
  }

  Ok(())
}

fn with_config<T>(
  f: impl FnOnce(&AndroidConfig, &AndroidMetadata) -> Result<T, Error>,
) -> Result<T, Error> {
  let (config, metadata) = {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(tauri_config_)
  };
  f(config.android(), metadata.android())
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

fn dev(options: DevOptions) -> Result<()> {
  with_config(|config, _metadata| {
    run_dev(options, config).map_err(|e| Error::DevFailed(e.to_string()))
  })
  .map_err(Into::into)
}

fn run_dev(options: DevOptions, config: &AndroidConfig) -> Result<()> {
  let mut dev_options = options.clone().into();
  let mut interface = crate::dev::setup(&mut dev_options)?;

  {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let cli_options = CliOptions {
      features: dev_options.features,
      args: dev_options.args,
      vars: Default::default(),
    };
    write_options(
      cli_options,
      &tauri_config_.tauri.bundle.identifier,
      MobileTarget::Android,
    )?;
  }

  interface.mobile_dev(
    MobileOptions {
      debug: true,
      features: options.features,
      args: Vec::new(),
      config: options.config,
      no_watch: options.no_watch,
    },
    |options| match run(options) {
      Ok(c) => Ok(Box::new(c) as Box<dyn DevProcess>),
      Err(Error::FailedToPromptForDevice(_)) => open_dev(config),
      Err(e) => Err(e.into()),
    },
  )
}

fn open_dev(config: &AndroidConfig) -> ! {
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir()) {
    log::error!("{}", e);
  }
  loop {
    std::thread::sleep(std::time::Duration::from_secs(24 * 60 * 60));
  }
}

fn open() -> Result<()> {
  with_config(|config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Android Studio", config.project_dir()).map_err(Error::OpenFailed)
  })
  .map_err(Into::into)
}

fn run(options: MobileOptions) -> Result<DevChild, Error> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  with_config(|config, metadata| {
    let build_app_bundle = metadata.asset_packs().is_some();

    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = Env::new().map_err(Error::EnvInitFailed)?;

    device_prompt(&env)
      .map_err(Error::FailedToPromptForDevice)?
      .run(
        config,
        &env,
        NoiseLevel::Polite,
        profile,
        None,
        build_app_bundle,
        false.into(),
        ".MainActivity".into(),
      )
      .map_err(Error::RunFailed)
  })
  .map(|c| DevChild(Some(c)))
}

fn build(options: BuildOptions) -> Result<()> {
  let profile = if options.release {
    Profile::Release
  } else {
    Profile::Debug
  };

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
  })
  .map_err(Into::into)
}
