// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  configure_cargo, delete_codegen_vars, ensure_init, env, get_app, get_config, inject_assets,
  log_finished, open_and_wait, MobileTarget, OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock, resolve_merge_config,
  },
  interface::{AppSettings, Interface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  android::{aab, apk, config::Config as AndroidConfig, env::Env, target::Target},
  opts::{NoiseLevel, Profile},
  target::TargetTrait,
};

use std::env::{set_current_dir, set_var};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for Android and generate APKs and AABs",
  long_about = "Build your app in release mode for Android and generate APKs and AABs. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
)]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build (all by default).
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
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
  /// Open Android Studio
  #[clap(short, long)]
  pub open: bool,
}

impl From<Options> for BuildOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      config: options.config,
      args: Vec::new(),
      ci: false,
    }
  }
}

pub fn command(mut options: Options, noise_level: NoiseLevel) -> Result<()> {
  delete_codegen_vars();

  let (merge_config, _merge_config_path) = resolve_merge_config(&options.config)?;
  options.config = merge_config;

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Android,
    options.config.as_deref(),
  )?;
  let (app, config, metadata) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let app = get_app(tauri_config_);
    let (config, metadata) = get_config(&app, tauri_config_, &Default::default());
    (app, config, metadata)
  };

  set_var("WRY_RUSTWEBVIEWCLIENT_CLASS_EXTENSION", "");
  set_var("WRY_RUSTWEBVIEW_CLASS_INIT", "");

  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  ensure_init(config.project_dir(), MobileTarget::Android)?;

  let mut env = env()?;
  configure_cargo(&app, Some((&mut env, &config)))?;

  // run an initial build to initialize plugins
  Target::all().values().next().unwrap().build(
    &config,
    &metadata,
    &env,
    noise_level,
    true,
    if options.debug {
      Profile::Debug
    } else {
      Profile::Release
    },
  )?;

  let open = options.open;
  let _handle = run_build(
    options,
    tauri_config,
    profile,
    &config,
    &mut env,
    noise_level,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(())
}

fn run_build(
  mut options: Options,
  tauri_config: ConfigHandle,
  profile: Profile,
  config: &AndroidConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
) -> Result<OptionsHandle> {
  if !(options.apk || options.aab) {
    // if the user didn't specify the format to build, we'll do both
    options.apk = true;
    options.aab = true;
  }

  let mut build_options: BuildOptions = options.clone().into();
  build_options.target = Some(
    Target::all()
      .get(Target::DEFAULT_KEY)
      .unwrap()
      .triple
      .into(),
  );
  let interface = crate::build::setup(
    tauri_utils::platform::Target::Android,
    &mut build_options,
    true,
  )?;

  let interface_options = InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    ..Default::default()
  };

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&interface_options)?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("android"), "Android")?;

  let cli_options = CliOptions {
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
  };
  let handle = write_options(
    &tauri_config.lock().unwrap().as_ref().unwrap().identifier,
    cli_options,
  )?;

  options
    .features
    .get_or_insert(Vec::new())
    .push("custom-protocol".into());

  inject_assets(config, tauri_config.lock().unwrap().as_ref().unwrap())?;

  let apk_outputs = if options.apk {
    apk::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(options.targets.clone().unwrap_or_default())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  let aab_outputs = if options.aab {
    aab::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(options.targets.unwrap_or_default())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  log_finished(apk_outputs, "APK");
  log_finished(aab_outputs, "AAB");

  Ok(handle)
}

fn get_targets_or_all<'a>(targets: Vec<String>) -> Result<Vec<&'a Target<'a>>> {
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
        anyhow::anyhow!(
          "Target {} is invalid; the possible targets are {}",
          t,
          possible_targets
        )
      })?;
      outs.push(target);
    }
    Ok(outs)
  }
}
