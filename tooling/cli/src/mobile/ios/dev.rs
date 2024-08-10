// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  configure_cargo, device_prompt, ensure_init, env, get_app, get_config, inject_assets,
  merge_plist, open_and_wait, MobileTarget,
};
use crate::{
  dev::Options as DevOptions,
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, reload as reload_config, ConfigHandle},
    flock,
  },
  interface::{AppInterface, AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  apple::{
    config::Config as AppleConfig,
    device::{Device, DeviceKind},
  },
  config::app::App,
  env::Env,
  opts::{NoiseLevel, Profile},
};

use std::{
  env::set_current_dir,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::OnceLock,
};

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
  /// it is your responsability to set up your development server to listen on this address
  /// by using 0.0.0.0 as host for instance.
  ///
  /// When this is set or when running on an iOS device the CLI sets the `TAURI_DEV_HOST`
  /// environment variable so you can check this on your framework's configuration to expose the development server
  /// on the public network address.
  #[clap(long)]
  pub host: Option<Option<IpAddr>>,
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
      host: None,
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

fn local_ip_address(force: bool) -> &'static IpAddr {
  static LOCAL_IP: OnceLock<IpAddr> = OnceLock::new();
  LOCAL_IP.get_or_init(|| {
    let prompt_for_ip = || {
      let addresses: Vec<IpAddr> = local_ip_address::list_afinet_netifas()
        .expect("failed to list networks")
        .into_iter()
        .map(|(_, ipaddr)| ipaddr)
        .filter(|ipaddr| match ipaddr {
          IpAddr::V4(i) => i != &Ipv4Addr::LOCALHOST,
          IpAddr::V6(i) => i.to_string().ends_with("::2"),

        })
        .collect();
      match addresses.len() {
        0 => panic!("No external IP detected."),
        1 => {
          let ipaddr = addresses.first().unwrap();
          *ipaddr
        }
        _ => {
          let selected = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(
              "Failed to detect external IP, What IP should we use to access your development server?",
            )
            .items(&addresses)
            .default(0)
            .interact()
            .expect("failed to select external IP");
          *addresses.get(selected).unwrap()
        }
      }
    };

    let ip = if force {
      prompt_for_ip()
    } else {
      local_ip_address::local_ip().unwrap_or_else(|_| prompt_for_ip())
    };
    log::info!("Using {ip} to access the development server.");
    ip
  })
}

fn use_network_address_for_dev_url(
  config: &ConfigHandle,
  options: &mut Options,
  dev_options: &mut DevOptions,
) -> crate::Result<()> {
  let mut dev_url = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .dev_url
    .clone();

  let ip = if let Some(url) = &mut dev_url {
    let localhost = match url.host() {
      Some(url::Host::Domain(d)) => d == "localhost",
      Some(url::Host::Ipv4(i)) => {
        i == std::net::Ipv4Addr::LOCALHOST || i == std::net::Ipv4Addr::UNSPECIFIED
      }
      _ => false,
    };

    if localhost {
      let ip = options
        .host
        .unwrap_or_default()
        .unwrap_or_else(|| *local_ip_address(options.force_ip_prompt));
      log::info!(
        "Replacing devUrl host with {ip}. {}.",
        "If your frontend is not listening on that address, try configuring your development server to use the `TAURI_DEV_HOST` environment variable or 0.0.0.0 as host"
      );

      *url = url::Url::parse(&format!(
        "{}://{}{}",
        url.scheme(),
        SocketAddr::new(ip, url.port_or_known_default().unwrap()),
        url.path()
      ))?;

      if let Some(c) = &mut options.config {
        if let Some(build) = c
          .0
          .as_object_mut()
          .and_then(|root| root.get_mut("build"))
          .and_then(|build| build.as_object_mut())
        {
          build.insert("devUrl".into(), url.to_string().into());
        }
      } else {
        let mut build = serde_json::Map::new();
        build.insert("devUrl".into(), url.to_string().into());

        options
          .config
          .replace(crate::ConfigValue(serde_json::json!({
            "build": build
          })));
      }
      reload_config(options.config.as_ref().map(|c| &c.0))?;

      Some(ip)
    } else {
      None
    }
  } else if !dev_options.no_dev_server {
    let ip = options
      .host
      .unwrap_or_default()
      .unwrap_or_else(|| *local_ip_address(options.force_ip_prompt));
    dev_options.host.replace(ip);
    Some(ip)
  } else {
    None
  };

  if let Some(ip) = ip {
    std::env::set_var("TAURI_DEV_HOST", ip.to_string());
    if ip.is_ipv6() {
      // in this case we can't ping the server for some reason
      dev_options.no_dev_server_wait = true;
    }
  }

  Ok(())
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
  // when running on an actual device we must use the network IP
  if options.host.is_some()
    || device
      .as_ref()
      .map(|device| !matches!(device.kind(), DeviceKind::Simulator))
      .unwrap_or(false)
  {
    use_network_address_for_dev_url(&tauri_config, &mut options, &mut dev_options)?;
  }

  crate::dev::setup(&interface, &mut dev_options, tauri_config.clone())?;

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
