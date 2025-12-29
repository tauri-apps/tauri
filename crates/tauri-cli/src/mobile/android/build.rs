// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  configure_cargo, delete_codegen_vars, ensure_init, env, get_app, get_config, inject_resources,
  log_finished, open_and_wait, MobileTarget, OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  error::Context,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock,
  },
  interface::{AppInterface, Interface, Options as InterfaceOptions},
  mobile::{android::generate_tauri_properties, write_options, CliOptions, TargetDevice},
  ConfigValue, Error, Result,
};
use clap::{ArgAction, Parser};

use cargo_mobile2::{
  android::{aab, apk, config::Config as AndroidConfig, env::Env, target::Target},
  opts::{NoiseLevel, Profile},
  target::TargetTrait,
};

use std::env::set_current_dir;

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
  pub features: Vec<String>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Whether to split the APKs and AABs per ABIs.
  #[clap(long)]
  pub split_per_abi: bool,
  /// Build APKs.
  #[clap(long)]
  pub apk: Option<bool>,
  /// Build AABs.
  #[clap(long)]
  pub aab: Option<bool>,
  /// Open Android Studio
  #[clap(short, long)]
  pub open: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri android build -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
  /// Target device of this build
  #[clap(skip)]
  pub target_device: Option<TargetDevice>,
}

impl From<Options> for BuildOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      no_bundle: false,
      config: options.config,
      args: options.args,
      ci: options.ci,
      skip_stapling: false,
      ignore_version_mismatches: options.ignore_version_mismatches,
      no_sign: false,
    }
  }
}

pub struct BuiltApplication {
  pub config: AndroidConfig,
  pub interface: AppInterface,
  // prevent drop
  #[allow(dead_code)]
  options_handle: OptionsHandle,
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<BuiltApplication> {
  crate::helpers::app_paths::resolve();

  delete_codegen_vars();

  let mut build_options: BuildOptions = options.clone().into();

  let first_target = Target::all()
    .get(
      options
        .targets
        .as_ref()
        .and_then(|l| l.first().map(|t| t.as_str()))
        .unwrap_or(Target::DEFAULT_KEY),
    )
    .unwrap();
  build_options.target = Some(first_target.triple.into());

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Android,
    &options
      .config
      .iter()
      .map(|conf| &conf.0)
      .collect::<Vec<_>>(),
  )?;
  let (interface, config, metadata) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let interface = AppInterface::new(tauri_config_, build_options.target.clone())?;
    interface.build_options(&mut Vec::new(), &mut build_options.features, true);

    let app = get_app(MobileTarget::Android, tauri_config_, &interface);
    let (config, metadata) = get_config(
      &app,
      tauri_config_,
      &build_options.features,
      &Default::default(),
    );
    (interface, config, metadata)
  };

  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).context("failed to set current directory to Tauri directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::Android,
    options.ci,
  )?;

  let mut env = env(options.ci)?;
  configure_cargo(&mut env, &config)?;

  generate_tauri_properties(
    &config,
    tauri_config.lock().unwrap().as_ref().unwrap(),
    false,
  )?;

  {
    let config_guard = tauri_config.lock().unwrap();
    let config_ = config_guard.as_ref().unwrap();

    crate::build::setup(&interface, &mut build_options, config_, true)?;
  }

  let installed_targets =
    crate::interface::rust::installation::installed_targets().unwrap_or_default();

  if !installed_targets.contains(&first_target.triple().into()) {
    log::info!("Installing target {}", first_target.triple());
    first_target
      .install()
      .map_err(|error| Error::CommandFailed {
        command: "rustup target add".to_string(),
        error,
      })
      .context("failed to install target")?;
  }
  // run an initial build to initialize plugins
  first_target
    .build(&config, &metadata, &env, noise_level, true, profile)
    .context("failed to build Android app")?;

  let open = options.open;
  let options_handle = run_build(
    &interface,
    options,
    build_options,
    tauri_config,
    profile,
    &config,
    &mut env,
    noise_level,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(BuiltApplication {
    config,
    interface,
    options_handle,
  })
}

#[allow(clippy::too_many_arguments)]
fn run_build(
  interface: &AppInterface,
  mut options: Options,
  build_options: BuildOptions,
  tauri_config: ConfigHandle,
  profile: Profile,
  config: &AndroidConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
) -> Result<OptionsHandle> {
  if !(options.apk.is_some() || options.aab.is_some()) {
    // if the user didn't specify the format to build, we'll do both
    options.apk = Some(true);
    options.aab = Some(true);
  }

  let interface_options = InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    args: build_options.args.clone(),
    ..Default::default()
  };

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(&interface_options)?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("android"), "Android")?;

  let cli_options = CliOptions {
    dev: false,
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
    config: build_options.config,
    target_device: options.target_device.clone(),
  };
  let handle = write_options(tauri_config.lock().unwrap().as_ref().unwrap(), cli_options)?;

  inject_resources(config, tauri_config.lock().unwrap().as_ref().unwrap())?;

  let apk_outputs = if options.apk.unwrap_or_default() {
    apk::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(options.targets.clone().unwrap_or_default())?,
      options.split_per_abi,
    )
    .context("failed to build APK")?
  } else {
    Vec::new()
  };

  let aab_outputs = if options.aab.unwrap_or_default() {
    aab::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(options.targets.unwrap_or_default())?,
      options.split_per_abi,
    )
    .context("failed to build AAB")?
  } else {
    Vec::new()
  };

  if !apk_outputs.is_empty() {
    log_finished(apk_outputs, "APK");
  }
  if !aab_outputs.is_empty() {
    log_finished(aab_outputs, "AAB");
  }

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
      let target = Target::for_name(&t).with_context(|| {
        format!("Target {t} is invalid; the possible targets are {possible_targets}",)
      })?;
      outs.push(target);
    }
    Ok(outs)
  }
}
