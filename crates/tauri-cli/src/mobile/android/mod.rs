// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile2::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
    device::Device,
    emulator,
    env::Env,
    target::Target,
  },
  config::app::{App, DEFAULT_ASSET_DIR},
  opts::{FilterLevel, NoiseLevel},
  os,
  target::TargetTrait,
  util::prompt,
};
use clap::{Parser, Subcommand};
use std::{
  env::set_var,
  fs::{create_dir, create_dir_all, write},
  process::exit,
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
  helpers::config::{BundleResources, Config as TauriConfig},
  Result,
};

mod android_studio_script;
mod build;
mod dev;
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

#[derive(Debug, Parser)]
#[clap(about = "Initialize Android target in the project")]
pub struct InitOptions {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  /// Skips installing rust toolchains via rustup
  #[clap(long)]
  skip_targets_install: bool,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Dev(dev::Options),
  Build(build::Options),
  #[clap(hide(true))]
  AndroidStudioScript(android_studio_script::Options),
}

pub fn command(cli: Cli, verbosity: u8) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => {
      crate::helpers::app_paths::resolve();
      init_command(
        MobileTarget::Android,
        options.ci,
        false,
        options.skip_targets_install,
      )?
    }
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level)?,
    Commands::AndroidStudioScript(options) => android_studio_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: &App,
  config: &TauriConfig,
  features: Option<&Vec<String>>,
  cli_options: &CliOptions,
) -> (AndroidConfig, AndroidMetadata) {
  let mut android_options = cli_options.clone();
  if let Some(features) = features {
    android_options
      .features
      .get_or_insert(Vec::new())
      .extend_from_slice(features);
  }

  let raw = RawAndroidConfig {
    features: android_options.features.clone(),
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
    min_sdk_version: Some(config.bundle.android.min_sdk_version),
    ..Default::default()
  };
  let config = AndroidConfig::from_raw(app.clone(), Some(raw)).unwrap();

  let metadata = AndroidMetadata {
    supported: true,
    cargo_args: Some(android_options.args),
    features: android_options.features,
    ..Default::default()
  };

  set_var(
    "WRY_ANDROID_PACKAGE",
    app.android_identifier_escape_kotlin_keyword(),
  );
  set_var("TAURI_ANDROID_PACKAGE_UNESCAPED", app.identifier());
  set_var("WRY_ANDROID_LIBRARY", app.lib_name());
  set_var("TAURI_ANDROID_PROJECT_PATH", config.project_dir());

  let src_main_dir = config
    .project_dir()
    .join("app/src/main")
    .join(format!("java/{}", app.identifier().replace('.', "/"),));
  if config.project_dir().exists() {
    if src_main_dir.exists() {
      let _ = create_dir(src_main_dir.join("generated"));
    } else {
      log::error!(
      "Project directory {} does not exist. Did you update the package name in `Cargo.toml` or the bundle identifier in `tauri.conf.json > identifier`? Save your changes, delete the `gen/android` folder and run `tauri android init` to recreate the Android project.",
      src_main_dir.display()
    );
      exit(1);
    }
  }
  set_var(
    "WRY_ANDROID_KOTLIN_FILES_OUT_DIR",
    src_main_dir.join("generated"),
  );

  (config, metadata)
}

fn env() -> Result<Env> {
  let env = super::env()?;
  cargo_mobile2::android::env::Env::from_env(env).map_err(Into::into)
}

fn delete_codegen_vars() {
  for (k, _) in std::env::vars() {
    if k.starts_with("WRY_") && (k.ends_with("CLASS_EXTENSION") || k.ends_with("CLASS_INIT")) {
      std::env::remove_var(k);
    }
  }
}

fn adb_device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  let device_list = adb::device_list(env)
    .map_err(|cause| anyhow::anyhow!("Failed to detect connected Android devices: {cause}"))?;
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
        anyhow::bail!("Could not find an Android device matching {t}")
      }
    } else if device_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "Android", " devices"),
        device_list.iter(),
        "device",
        None,
        "Device",
      )
      .map_err(|cause| anyhow::anyhow!("Failed to prompt for Android device: {cause}"))?;
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
    Err(anyhow::anyhow!("No connected Android devices detected"))
  }
}

fn emulator_prompt(env: &'_ Env, target: Option<&str>) -> Result<emulator::Emulator> {
  let emulator_list = emulator::avd_list(env).unwrap_or_default();
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
        anyhow::bail!("Could not find an Android Emulator matching {t}")
      }
    } else if emulator_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "Android", " emulators"),
        emulator_list.iter(),
        "emulator",
        None,
        "Emulator",
      )
      .map_err(|cause| anyhow::anyhow!("Failed to prompt for Android Emulator device: {cause}"))?;
      emulator_list.into_iter().nth(index).unwrap()
    } else {
      emulator_list.into_iter().next().unwrap()
    };

    Ok(emulator)
  } else {
    Err(anyhow::anyhow!("No available Android Emulator detected"))
  }
}

fn device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  if let Ok(device) = adb_device_prompt(env, target) {
    Ok(device)
  } else {
    let emulator = emulator_prompt(env, target)?;
    log::info!("Starting emulator {}", emulator.name());
    emulator.start_detached(env)?;
    let mut tries = 0;
    loop {
      sleep(Duration::from_secs(2));
      if let Ok(device) = adb_device_prompt(env, Some(emulator.name())) {
        return Ok(device);
      }
      if tries >= 3 {
        log::info!("Waiting for emulator to start... (maybe the emulator is unauthorized or offline, run `adb devices` to check)");
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

fn open_and_wait(config: &AndroidConfig, env: &Env) -> ! {
  log::info!("Opening Android Studio");
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir(), &env.base) {
    log::error!("{}", e);
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}

fn inject_resources(config: &AndroidConfig, tauri_config: &TauriConfig) -> Result<()> {
  let asset_dir = config
    .project_dir()
    .join("app/src/main")
    .join(DEFAULT_ASSET_DIR);
  create_dir_all(&asset_dir)?;

  write(
    asset_dir.join("tauri.conf.json"),
    serde_json::to_string(&tauri_config)?,
  )?;

  let resources = match &tauri_config.bundle.resources {
    Some(BundleResources::List(paths)) => Some(ResourcePaths::new(paths.as_slice(), true)),
    Some(BundleResources::Map(map)) => Some(ResourcePaths::from_map(map, true)),
    None => None,
  };
  if let Some(resources) = resources {
    for resource in resources.iter() {
      let resource = resource?;
      let dest = asset_dir.join(resource.target());
      crate::helpers::fs::copy_file(resource.path(), dest)?;
    }
  }

  Ok(())
}

fn configure_cargo(env: &mut Env, config: &AndroidConfig) -> Result<()> {
  for target in Target::all().values() {
    let config = target.generate_cargo_config(config, env)?;
    let target_var_name = target.triple.replace('-', "_").to_uppercase();
    if let Some(linker) = config.linker {
      env.base.insert_env_var(
        format!("CARGO_TARGET_{target_var_name}_LINKER"),
        linker.into(),
      );
    }
    env.base.insert_env_var(
      format!("CARGO_TARGET_{target_var_name}_RUSTFLAGS"),
      config.rustflags.join(" ").into(),
    );
  }

  Ok(())
}
