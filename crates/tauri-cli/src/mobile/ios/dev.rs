// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  device_prompt, ensure_init, env, get_app, get_config, inject_resources, load_pbxproj,
  merge_plist, open_and_wait, synchronize_project_config, MobileTarget, ProjectConfig,
};
use crate::{
  dev::Options as DevOptions,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock,
  },
  interface::{AppInterface, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{
    use_network_address_for_dev_url, write_options, CliOptions, DevChild, DevHost, DevProcess,
  },
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  apple::{
    config::Config as AppleConfig,
    device::{Device, DeviceKind},
  },
  env::Env,
  opts::{NoiseLevel, Profile},
};

use std::env::set_current_dir;

const PHYSICAL_IPHONE_DEV_WARNING: &str = "To develop on physical phones you need the `--host` option (not required for Simulators). See the documentation for more information: https://v2.tauri.app/develop/#development-server";

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Run your app in development mode on iOS",
  long_about = "Run your app in development mode on iOS with hot-reloading for the Rust code.
It makes use of the `build.devUrl` property from your `tauri.conf.json` file.
It also runs your `build.beforeDevCommand` which usually starts your frontend devServer.

When connected to a physical iOS device, the public network address must be used instead of `localhost`
for the devUrl property. Tauri makes that change automatically, but your dev server might need
a different configuration to listen on the public address. You can check the `TAURI_DEV_HOST`
environment variable to determine whether the public network should be used or not."
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
  /// Use the public network address for the development server.
  /// If an actual address it provided, it is used instead of prompting to pick one.
  ///
  /// This option is particularly useful along the `--open` flag when you intend on running on a physical device.
  ///
  /// This replaces the devUrl configuration value to match the public network address host,
  /// it is your responsibility to set up your development server to listen on this address
  /// by using 0.0.0.0 as host for instance.
  ///
  /// When this is set or when running on an iOS device the CLI sets the `TAURI_DEV_HOST`
  /// environment variable so you can check this on your framework's configuration to expose the development server
  /// on the public network address.
  #[clap(long, default_value_t, default_missing_value(""), num_args(0..=1))]
  pub host: DevHost,
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
      host: options.host.0.unwrap_or_default(),
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  crate::helpers::app_paths::resolve();

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
  let (interface, config) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let interface = AppInterface::new(tauri_config_, Some(target_triple))?;

    let app = get_app(MobileTarget::Ios, tauri_config_, &interface);
    let (config, _metadata) = get_config(
      &app,
      tauri_config_,
      dev_options.features.as_ref(),
      &Default::default(),
    );

    (interface, config)
  };

  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::Ios,
  )?;
  inject_resources(&config, tauri_config.lock().unwrap().as_ref().unwrap())?;

  let info_plist_path = config
    .project_dir()
    .join(config.scheme())
    .join("Info.plist");
  let merged_info_plist = merge_plist(vec![
    info_plist_path.clone().into(),
    tauri_path.join("Info.plist").into(),
    tauri_path.join("Info.ios.plist").into(),
  ])?;
  merged_info_plist.to_file_xml(&info_plist_path)?;

  let mut pbxproj = load_pbxproj(&config)?;

  // synchronize pbxproj
  synchronize_project_config(
    &config,
    &tauri_config,
    &mut pbxproj,
    &mut plist::Dictionary::new(),
    &ProjectConfig {
      code_sign_identity: None,
      team_id: None,
      provisioning_profile_uuid: None,
    },
    !options.release_mode,
  )?;
  if pbxproj.has_changes() {
    pbxproj.save()?;
  }

  run_dev(
    interface,
    options,
    dev_options,
    tauri_config,
    device,
    env,
    &config,
    noise_level,
  )
}

#[allow(clippy::too_many_arguments)]
fn run_dev(
  mut interface: AppInterface,
  options: Options,
  mut dev_options: DevOptions,
  tauri_config: ConfigHandle,
  device: Option<Device>,
  env: Env,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<()> {
  // when running on an actual device we must use the network IP
  if options.host.0.is_some()
    || device
      .as_ref()
      .map(|device| !matches!(device.kind(), DeviceKind::Simulator))
      .unwrap_or(false)
  {
    use_network_address_for_dev_url(&tauri_config, &mut dev_options, options.force_ip_prompt)?;
  }

  crate::dev::setup(&interface, &mut dev_options, tauri_config.clone())?;

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(&InterfaceOptions {
    debug: !dev_options.release_mode,
    target: dev_options.target.clone(),
    ..Default::default()
  })?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ios"), "iOS")?;

  let set_host = options.host.0.is_some();

  let open = options.open;
  interface.mobile_dev(
    MobileOptions {
      debug: true,
      features: options.features,
      args: Vec::new(),
      config: dev_options.config.clone(),
      no_watch: options.no_watch,
    },
    |options| {
      let cli_options = CliOptions {
        dev: true,
        features: options.features.clone(),
        args: options.args.clone(),
        noise_level,
        vars: Default::default(),
        config: dev_options.config.clone(),
        target_device: None,
      };
      let _handle = write_options(
        &tauri_config.lock().unwrap().as_ref().unwrap().identifier,
        cli_options,
      )?;

      if open {
        if !set_host {
          log::warn!("{PHYSICAL_IPHONE_DEV_WARNING}");
        }
        open_and_wait(config, &env)
      } else if let Some(device) = &device {
        match run(device, options, config, noise_level, &env) {
          Ok(c) => Ok(Box::new(c) as Box<dyn DevProcess + Send>),
          Err(e) => {
            crate::dev::kill_before_dev_process();
            Err(e)
          }
        }
      } else {
        if !set_host {
          log::warn!("{PHYSICAL_IPHONE_DEV_WARNING}");
        }
        open_and_wait(config, &env)
      }
    },
  )
}

fn run(
  device: &Device<'_>,
  options: MobileOptions,
  config: &AppleConfig,
  noise_level: NoiseLevel,
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
      noise_level,
      false, // do not quit on app exit
      profile,
    )
    .map(DevChild::new)
    .map_err(Into::into)
}
