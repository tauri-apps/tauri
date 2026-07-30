// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  delete_codegen_vars, ensure_init, env, get_app, get_config, inject_resources, log_finished,
  open_and_wait, MobileTarget, OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  error::Context,
  helpers::{
    config::{get_config as get_tauri_config, ConfigMetadata},
    flock,
  },
  interface::{AppInterface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};

use cargo_mobile2::{
  open_harmony::{config::Config as OpenHarmonyConfig, env::Env, hap, target::Target},
  opts::{NoiseLevel, Profile},
  target::TargetTrait,
};

use std::{env::set_current_dir, path::Path};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for OpenHarmony and generate HAPs",
  long_about = "Build your app in release mode for OpenHarmony and generate HAPs. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
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
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub features: Vec<String>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json, tauri.ios.conf.json and tauri.ohos.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Open DevEco Studio
  #[clap(short, long)]
  pub open: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri ohos build -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
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
      no_binary_patching: false,
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Android,
    &options
      .config
      .iter()
      .map(|conf| &conf.0)
      .collect::<Vec<_>>(),
    dirs.tauri,
  )?;

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

  let interface = AppInterface::new(&tauri_config, build_options.target.clone(), dirs.tauri)?;
  interface.build_options(&mut Vec::new(), &mut build_options.features, true);

  let app = get_app(
    MobileTarget::OpenHarmony,
    &tauri_config,
    &interface,
    dirs.tauri,
  );
  let (config, metadata) = get_config(
    &app,
    &tauri_config,
    &build_options.features,
    &Default::default(),
  );

  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  set_current_dir(dirs.tauri).context("failed to set current directory to Tauri directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::OpenHarmony,
    options.ci,
  )?;

  let mut env = env()?;

  crate::build::setup(&interface, &mut build_options, &tauri_config, &dirs, true)?;

  // run an initial build to initialize plugins
  first_target
    .build(&config, &metadata, &env, noise_level, true, profile)
    .context("failed to build OpenHarmony app")?;

  let open = options.open;
  let _handle = run_build(
    interface,
    options,
    build_options,
    &tauri_config,
    profile,
    &config,
    &mut env,
    noise_level,
    &dirs.tauri,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_build(
  interface: AppInterface,
  _options: Options,
  build_options: BuildOptions,
  tauri_config: &ConfigMetadata,
  profile: Profile,
  config: &OpenHarmonyConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
  tauri_dir: &Path,
) -> Result<OptionsHandle> {
  let interface_options = InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    args: build_options.args.clone(),
    ..Default::default()
  };

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(&interface_options, tauri_dir)?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ohos"), "OpenHarmony")?;

  let cli_options = CliOptions {
    dev: false,
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
    config: build_options.config,
    target_device: None,
  };
  let handle = write_options(tauri_config, cli_options)?;

  inject_resources(config, tauri_config)?;

  let hap_outputs = hap::build(config, env, noise_level, profile).context("failed to build HAP")?;

  log_finished(hap_outputs, "HAP");

  Ok(handle)
}
