// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  configure_cargo, detect_target_ok, ensure_init, env, get_app, get_config, inject_assets,
  log_finished, merge_plist, open_and_wait, MobileTarget, OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock, resolve_merge_config,
  },
  interface::{AppInterface, AppSettings, Interface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  apple::{config::Config as AppleConfig, target::Target},
  env::Env,
  opts::{NoiseLevel, Profile},
  target::{call_for_targets_with_fallback, TargetInvalid, TargetTrait},
};

use std::{env::set_current_dir, fs};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for iOS and generate IPAs",
  long_about = "Build your app in release mode for iOS and generate IPAs. It makes use of the `build.distDir` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.distDir`."
)]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build.
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Vec<String>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Build number to append to the app version.
  #[clap(long)]
  pub build_number: Option<u32>,
  /// Open Xcode
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
  let (merge_config, _merge_config_path) = resolve_merge_config(&options.config)?;
  options.config = merge_config;

  let mut build_options: BuildOptions = options.clone().into();
  build_options.target = Some(
    Target::all()
      .get(Target::DEFAULT_KEY)
      .unwrap()
      .triple
      .into(),
  );

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Ios,
    options.config.as_deref(),
  )?;
  let (interface, app, config) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let interface = AppInterface::new(tauri_config_, build_options.target.clone())?;

    let app = get_app(tauri_config_, &interface);
    let (config, _metadata) = get_config(&app, tauri_config_, &Default::default());
    (interface, app, config)
  };

  let tauri_path = tauri_dir();
  set_current_dir(&tauri_path).with_context(|| "failed to change current working directory")?;

  ensure_init(config.project_dir(), MobileTarget::Ios)?;
  inject_assets(&config)?;

  let info_plist_path = config
    .project_dir()
    .join(config.scheme())
    .join("Info.plist");
  merge_plist(
    &[
      tauri_path.join("Info.plist"),
      tauri_path.join("Info.ios.plist"),
    ],
    &info_plist_path,
  )?;

  let mut env = env()?;
  configure_cargo(&app, None)?;

  let open = options.open;
  let _handle = run_build(
    interface,
    options,
    build_options,
    tauri_config,
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
  interface: AppInterface,
  options: Options,
  mut build_options: BuildOptions,
  tauri_config: ConfigHandle,
  config: &AppleConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
) -> Result<OptionsHandle> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  crate::build::setup(
    tauri_utils::platform::Target::Ios,
    &interface,
    &mut build_options,
    true,
  )?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ios"), "iOS")?;

  let cli_options = CliOptions {
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
  };
  let handle = write_options(
    &tauri_config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .tauri
      .bundle
      .identifier,
    cli_options,
  )?;

  options
    .features
    .get_or_insert(Vec::new())
    .push("custom-protocol".into());

  let mut out_files = Vec::new();

  call_for_targets_with_fallback(
    options.targets.iter(),
    &detect_target_ok,
    env,
    |target: &Target| -> Result<()> {
      let mut app_version = config.bundle_version().clone();
      if let Some(build_number) = options.build_number {
        app_version.push_extra(build_number);
      }

      target.build(config, env, NoiseLevel::FranklyQuitePedantic, profile)?;
      target.archive(config, env, noise_level, profile, Some(app_version))?;
      target.export(config, env, noise_level)?;

      if let Ok(ipa_path) = config.ipa_path() {
        let out_dir = config.export_dir().join(target.arch);
        fs::create_dir_all(&out_dir)?;
        let path = out_dir.join(ipa_path.file_name().unwrap());
        fs::rename(&ipa_path, &path)?;
        out_files.push(path);
      }

      Ok(())
    },
  )
  .map_err(|e: TargetInvalid| anyhow::anyhow!(e.to_string()))??;

  log_finished(out_files, "IPA");

  Ok(handle)
}
