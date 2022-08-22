// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::{
    aab, adb, apk,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata},
    device::{Device, RunError},
    env::{Env, Error as EnvError},
    target::{BuildError, Target},
  },
  config::Config,
  device::PromptError,
  opts::{NoiseLevel, Profile},
  os,
  target::{call_for_targets_with_fallback, TargetTrait},
  util::prompt,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init, get_config,
  init::{command as init_command, Options as InitOptions},
  write_options, CliOptions, DevChild, Target as MobileTarget,
};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, DevProcess, Interface, MobileOptions, Options as InterfaceOptions},
  Result,
};

use std::{fmt::Write, path::PathBuf};

pub(crate) mod project;

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

#[derive(Debug, Parser)]
pub struct AndroidStudioScriptOptions {
  /// Targets to build.
  #[clap(
    short,
    long = "target",
    multiple_occurrences(true),
    multiple_values(true),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
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
  /// Open Android Studio instead of trying to run on a connected device
  #[clap(short, long)]
  pub open: bool,
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

#[derive(Debug, Clone, Parser)]
#[clap(about = "Android build")]
pub struct BuildOptions {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build (all by default).
  #[clap(
    short,
    long = "target",
    multiple_occurrences(true),
    multiple_values(true),
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Whether to split the APKs and AABs per ABIs.
  #[clap(long)]
  pub split_per_abi: bool,
  /// Build APKs.
  #[clap(long)]
  pub apk: bool,
  /// Build AABs.
  #[clap(long)]
  pub aab: bool,
}

impl From<BuildOptions> for crate::build::Options {
  fn from(options: BuildOptions) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      config: options.config,
      args: Vec::new(),
    }
  }
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  /// Open project in Android Studio
  Open,
  Dev(DevOptions),
  Build(BuildOptions),
  #[clap(hide(true))]
  AndroidStudioScript(AndroidStudioScriptOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Android)?,
    Commands::Open => open()?,
    Commands::Dev(options) => dev(options)?,
    Commands::Build(options) => build(options)?,
    Commands::AndroidStudioScript(options) => android_studio_script(options)?,
  }

  Ok(())
}

fn with_config<T>(
  f: impl FnOnce(&Config, &AndroidConfig, &AndroidMetadata) -> Result<T, Error>,
) -> Result<T, Error> {
  let (config, metadata) = {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(tauri_config_)
  };
  f(&config, config.android(), metadata.android())
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

fn get_targets_or_all<'a>(targets: Vec<String>) -> Result<Vec<&'a Target<'a>>, Error> {
  if targets.is_empty() {
    Ok(Target::all().iter().map(|t| t.1).collect())
  } else {
    let mut outs = Vec::new();

    let possible_targets = Target::all()
      .keys()
      .map(|key| key.to_string())
      .collect::<Vec<String>>()
      .join(",");

    for t in targets {
      let target = Target::for_name(&t).ok_or_else(|| {
        Error::TargetInvalid(format!(
          "Target {} is invalid; the possible targets are {}",
          t, possible_targets
        ))
      })?;
      outs.push(target);
    }
    Ok(outs)
  }
}

fn build(options: BuildOptions) -> Result<()> {
  with_config(|root_conf, config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = Env::new().map_err(Error::EnvInitFailed)?;
    super::init::init_dot_cargo(root_conf, Some(&env)).map_err(Error::InitDotCargo)?;

    run_build(options, config, env).map_err(|e| Error::BuildFailed(e.to_string()))
  })
  .map_err(Into::into)
}

fn run_build(mut options: BuildOptions, config: &AndroidConfig, env: Env) -> Result<()> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  if !(options.apk || options.aab) {
    // if the user didn't specify the format to build, we'll do both
    options.apk = true;
    options.aab = true;
  }

  let bundle_identifier = {
    let tauri_config = get_tauri_config(None)?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    tauri_config_.tauri.bundle.identifier.clone()
  };

  let mut build_options = options.clone().into();
  let interface = crate::build::setup(&mut build_options)?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: build_options.debug,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("android"), "Android")?;

  let cli_options = CliOptions {
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    vars: Default::default(),
  };
  write_options(cli_options, &bundle_identifier, MobileTarget::Android)?;

  options
    .features
    .get_or_insert(Vec::new())
    .push("custom-protocol".into());

  let apk_outputs = if options.apk {
    apk::build(
      config,
      &env,
      NoiseLevel::Polite,
      profile,
      get_targets_or_all(Vec::new())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  let aab_outputs = if options.aab {
    aab::build(
      config,
      &env,
      NoiseLevel::Polite,
      profile,
      get_targets_or_all(Vec::new())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  log_finished(apk_outputs, "APK");
  log_finished(aab_outputs, "AAB");

  Ok(())
}

fn log_finished(outputs: Vec<PathBuf>, kind: &str) {
  if !outputs.is_empty() {
    let mut printable_paths = String::new();
    for path in &outputs {
      writeln!(printable_paths, "        {}", path.display()).unwrap();
    }

    log::info!(action = "Finished"; "{} {}{} at:\n{}", outputs.len(), kind, if outputs.len() == 1 { "" } else { "s" }, printable_paths);
  }
}

fn dev(options: DevOptions) -> Result<()> {
  with_config(|_, config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    run_dev(options, config).map_err(|e| Error::DevFailed(e.to_string()))
  })
  .map_err(Into::into)
}

fn run_dev(options: DevOptions, config: &AndroidConfig) -> Result<()> {
  let mut dev_options = options.clone().into();
  let mut interface = crate::dev::setup(&mut dev_options)?;

  let bundle_identifier = {
    let tauri_config = get_tauri_config(None)?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    tauri_config_.tauri.bundle.identifier.clone()
  };

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: !dev_options.release_mode,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("android"), "Android")?;

  let open = options.open;
  interface.mobile_dev(
    MobileOptions {
      debug: true,
      features: options.features,
      args: Vec::new(),
      config: options.config,
      no_watch: options.no_watch,
    },
    |options| {
      let cli_options = CliOptions {
        features: options.features.clone(),
        args: options.args.clone(),
        vars: Default::default(),
      };
      write_options(cli_options, &bundle_identifier, MobileTarget::Android)?;

      if open {
        open_dev(config)
      } else {
        match run(options) {
          Ok(c) => Ok(Box::new(c) as Box<dyn DevProcess>),
          Err(Error::FailedToPromptForDevice(e)) => {
            log::error!("{}", e);
            open_dev(config)
          }
          Err(e) => Err(e.into()),
        }
      }
    },
  )
}

fn open_dev(config: &AndroidConfig) -> ! {
  log::info!("Opening Android Studio");
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir()) {
    log::error!("{}", e);
  }
  loop {
    std::thread::sleep(std::time::Duration::from_secs(24 * 60 * 60));
  }
}

fn open() -> Result<()> {
  with_config(|_, config, _metadata| {
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

  with_config(|root_conf, config, metadata| {
    let build_app_bundle = metadata.asset_packs().is_some();

    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = Env::new().map_err(Error::EnvInitFailed)?;
    super::init::init_dot_cargo(root_conf, Some(&env)).map_err(Error::InitDotCargo)?;

    device_prompt(&env)
      .map_err(Error::FailedToPromptForDevice)?
      .run(
        config,
        &env,
        NoiseLevel::Polite,
        profile,
        None,
        build_app_bundle,
        false,
        ".MainActivity".into(),
      )
      .map_err(Error::RunFailed)
  })
  .map(|c| DevChild(Some(c)))
}

fn android_studio_script(options: AndroidStudioScriptOptions) -> Result<()> {
  let profile = if options.release {
    Profile::Release
  } else {
    Profile::Debug
  };

  fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
    device_prompt(env).map(|device| device.target()).ok()
  }

  with_config(|root_conf, config, metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = Env::new().map_err(Error::EnvInitFailed)?;
    super::init::init_dot_cargo(root_conf, Some(&env)).map_err(Error::InitDotCargo)?;

    call_for_targets_with_fallback(
      options.targets.unwrap_or_default().iter(),
      &detect_target_ok,
      &env,
      |target: &Target| {
        target
          .build(config, metadata, &env, NoiseLevel::Polite, true, profile)
          .map_err(Error::AndroidStudioScriptFailed)
      },
    )
    .map_err(|e| Error::TargetInvalid(e.to_string()))?
  })
  .map_err(Into::into)
}
