// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use cargo_mobile2::opts::{NoiseLevel, Profile};
use clap::{ArgAction, Parser};

use super::{device_prompt, env};
use crate::{
  error::Context,
  interface::{DevProcess, Interface, WatcherOptions},
  mobile::{DevChild, TargetDevice},
  ConfigValue, Result,
};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Run your app in production mode on iOS",
  long_about = "Run your app in production mode on iOS. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
)]
pub struct Options {
  /// Run the app in release mode
  #[clap(short, long)]
  pub release: bool,
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
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
  /// Additional paths to watch for changes.
  #[clap(long)]
  pub additional_watch_folders: Vec<PathBuf>,
  /// Open Xcode
  #[clap(short, long)]
  pub open: bool,
  /// Runs on the given device name
  pub device: Option<String>,
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
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  let env = env().context("failed to load iOS environment")?;
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

  let mut built_application = super::build::command(
    super::build::Options {
      debug: !options.release,
      targets: Some(vec![]), /* skips IPA build since there's no target */
      features: Vec::new(),
      config: options.config.clone(),
      build_number: None,
      open: options.open,
      ci: false,
      export_method: None,
      args: options.args,
      ignore_version_mismatches: options.ignore_version_mismatches,
      target_device: device.as_ref().map(|d| TargetDevice {
        id: d.id().to_string(),
        name: d.name().to_string(),
      }),
    },
    noise_level,
  )?;

  // options.open is handled by the build command
  // so all we need to do here is run the app on the selected device
  if let Some(device) = device {
    let runner = move || {
      device
        .run(
          &built_application.config,
          &env,
          noise_level,
          false, // do not quit on app exit
          if !options.release {
            Profile::Debug
          } else {
            Profile::Release
          },
        )
        .map(|c| Box::new(DevChild::new(c)) as Box<dyn DevProcess + Send>)
        .context("failed to run iOS app")
    };

    if options.no_watch {
      runner()?;
    } else {
      built_application.interface.watch(
        WatcherOptions {
          config: options.config,
          additional_watch_folders: options.additional_watch_folders,
        },
        runner,
      )?;
    }
  }

  Ok(())
}
