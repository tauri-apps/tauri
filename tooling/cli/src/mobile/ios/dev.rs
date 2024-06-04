// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  configure_cargo, device_prompt, ensure_init, env, get_app, get_config, inject_assets,
  merge_plist, open_and_wait, setup_dev_config, MobileTarget,
};
use crate::{
  dev::Options as DevOptions,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock,
  },
  interface::{AppInterface, AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  apple::{config::Config as AppleConfig, device::Device},
  config::app::App,
  env::Env,
  opts::{NoiseLevel, Profile},
};

use std::env::set_current_dir;

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Run your app in development mode on iOS",
  long_about = "Run your app in development mode on iOS with hot-reloading for the Rust code. It makes use of the `build.devUrl` property from your `tauri.conf.json` file. It also runs your `build.beforeDevCommand` which usually starts your frontend devServer."
)]
pub struct Options {
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// Exit on panic
  #[clap(short, long)]
  exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<ConfigValue>,
  /// Run the code in release mode
  #[clap(long = "release")]
  pub release_mode: bool,
  /// Skip waiting for the frontend dev server to start before building the tauri application.
  #[clap(long, env = "TAURI_CLI_NO_DEV_SERVER_WAIT")]
  pub no_dev_server_wait: bool,
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
  /// Open Xcode instead of trying to run on a connected device
  #[clap(short, long)]
  pub open: bool,
  /// Runs on the given device name
  pub device: Option<String>,
  /// Force prompting for an IP to use to connect to the dev server on mobile.
  #[clap(long)]
  pub force_ip_prompt: bool,
  /// Disable the built-in dev server for static files.
  #[clap(long)]
  pub no_dev_server: bool,
  /// Specify port for the built-in dev server for static files. Defaults to 1430.
  #[clap(long, env = "TAURI_CLI_PORT")]
  pub port: Option<u16>,
}

impl From<Options> for DevOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      target: None,
      features: options.features,
      exit_on_panic: options.exit_on_panic,
      config: options.config,
      release_mode: options.release_mode,
      args: Vec::new(),
      no_watch: options.no_watch,
      no_dev_server: options.no_dev_server,
      no_dev_server_wait: options.no_dev_server_wait,
      port: options.port,
      force_ip_prompt: options.force_ip_prompt,
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  let result = run_command(options, noise_level);
  if result.is_err() {
    crate::dev::kill_before_dev_process();
  }
  result
}

fn run_command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  let env = env()?;
  let device = if options.open {
    None
  } else {
    match device_prompt(&env, options.device.as_deref()) {
      Ok(d) => Some(d),
      Err(e) => {
        log::error!("{e}");
        None
      }
    }
  };

  let mut dev_options: DevOptions = options.clone().into();
  let target_triple = device
    .as_ref()
    .map(|d| d.target().triple.to_string())
    .unwrap_or_else(|| "aarch64-apple-ios".into());
  dev_options.target = Some(target_triple.clone());

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Ios,
    options.config.as_ref().map(|c| &c.0),
  )?;
  let (interface, app, config) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let interface = AppInterface::new(tauri_config_, Some(target_triple))?;

    let app = get_app(tauri_config_, &interface);
    let (config, _metadata) = get_config(
      &app,
      tauri_config_,
      dev_options.features.as_ref(),
      &Default::default(),
    );
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
    vec![
      tauri_path.join("Info.plist").into(),
      tauri_path.join("Info.ios.plist").into(),
    ],
    &info_plist_path,
  )?;

  run_dev(
    interface,
    options,
    dev_options,
    tauri_config,
    device,
    env,
    &app,
    &config,
    noise_level,
  )
}

#[allow(clippy::too_many_arguments)]
fn run_dev(
  mut interface: AppInterface,
  mut options: Options,
  mut dev_options: DevOptions,
  tauri_config: ConfigHandle,
  device: Option<Device>,
  env: Env,
  app: &App,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<()> {
  setup_dev_config(
    MobileTarget::Ios,
    &mut options.config,
    options.force_ip_prompt,
  )?;

  crate::dev::setup(&interface, &mut dev_options, tauri_config.clone(), true)?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: !dev_options.release_mode,
    target: dev_options.target.clone(),
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ios"), "iOS")?;

  configure_cargo(app, None)?;

  let open = options.open;
  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
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
        noise_level,
        vars: Default::default(),
      };
      let _handle = write_options(
        &tauri_config.lock().unwrap().as_ref().unwrap().identifier,
        cli_options,
      )?;

      if open {
        open_and_wait(config, &env)
      } else if let Some(device) = &device {
        match run(device, options, config, &env) {
          Ok(c) => {
            crate::dev::wait_dev_process(c.clone(), move |status, reason| {
              crate::dev::on_app_exit(status, reason, exit_on_panic, no_watch)
            });
            Ok(Box::new(c) as Box<dyn DevProcess + Send>)
          }
          Err(e) => {
            crate::dev::kill_before_dev_process();
            Err(e)
          }
        }
      } else {
        open_and_wait(config, &env)
      }
    },
  )
}

fn run(
  device: &Device<'_>,
  options: MobileOptions,
  config: &AppleConfig,
  env: &Env,
) -> crate::Result<DevChild> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  device
    .run(
      config,
      env,
      NoiseLevel::FranklyQuitePedantic,
      false, // do not quit on app exit
      profile,
    )
    .map(DevChild::new)
    .map_err(Into::into)
}
