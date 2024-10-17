// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::BundleFormat,
  helpers::{
    self,
    app_paths::tauri_dir,
    config::{get as get_config, ConfigHandle, FrontendDist},
  },
  interface::{AppInterface, Interface},
  ConfigValue, Result,
};
use anyhow::Context;
use clap::{ArgAction, Parser};
use std::env::set_current_dir;
use tauri_utils::platform::Target;

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode and generate bundles and installers",
  long_about = "Build your app in release mode and generate bundles and installers. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`. This will also run `build.beforeBundleCommand` before generating the bundles and installers of your app."
)]
pub struct Options {
  /// Binary to use to build the application, defaults to `cargo`
  #[clap(short, long)]
  pub runner: Option<String>,
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Target triple to build against.
  ///
  /// It must be one of the values outputted by `$rustc --print target-list` or `universal-apple-darwin` for an universal macOS application.
  ///
  /// Note that compiling an universal macOS application requires both `aarch64-apple-darwin` and `x86_64-apple-darwin` targets to be installed.
  #[clap(short, long)]
  pub target: Option<String>,
  /// Space or comma separated list of features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// Space or comma separated list of bundles to package.
  ///
  /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub bundles: Option<Vec<BundleFormat>>,
  /// Skip the bundling step even if `bundle > active` is `true` in tauri config.
  #[clap(long)]
  pub no_bundle: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<ConfigValue>,
  /// Command line arguments passed to the runner. Use `--` to explicitly mark the start of the arguments.
  pub args: Vec<String>,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
}

pub fn command(mut options: Options, verbosity: u8) -> Result<()> {
  crate::helpers::app_paths::resolve();

  let ci = options.ci;

  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let config = get_config(target, options.config.as_ref().map(|c| &c.0))?;

  let mut interface = AppInterface::new(
    config.lock().unwrap().as_ref().unwrap(),
    options.target.clone(),
  )?;

  setup(&interface, &mut options, config.clone(), false)?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  let app_settings = interface.app_settings();
  let interface_options = options.clone().into();

  let out_dir = app_settings.out_dir(&interface_options)?;

  let bin_path = interface.build(interface_options)?;

  log::info!(action ="Built"; "application at: {}", tauri_utils::display_path(&bin_path));

  let app_settings = interface.app_settings();

  if !options.no_bundle && (config_.bundle.active || options.bundles.is_some()) {
    crate::bundle::bundle(
      &options.into(),
      verbosity,
      ci,
      &interface,
      &app_settings,
      config_,
      &out_dir,
    )?;
  }

  Ok(())
}

pub fn setup(
  interface: &AppInterface,
  options: &mut Options,
  config: ConfigHandle,
  mobile: bool,
) -> Result<()> {
  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  let bundle_identifier_source = config_
    .find_bundle_identifier_overwriter()
    .unwrap_or_else(|| "tauri.conf.json".into());

  if config_.identifier == "com.tauri.dev" {
    log::error!(
      "You must change the bundle identifier in `{} identifier`. The default value `com.tauri.dev` is not allowed as it must be unique across applications.",
      bundle_identifier_source
    );
    std::process::exit(1);
  }

  if config_
    .identifier
    .chars()
    .any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '.'))
  {
    log::error!(
      "The bundle identifier \"{}\" set in `{} identifier`. The bundle identifier string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-), and periods (.).",
      config_.identifier,
      bundle_identifier_source
    );
    std::process::exit(1);
  }

  if let Some(before_build) = config_.build.before_build_command.clone() {
    helpers::run_hook("beforeBuildCommand", before_build, interface, options.debug)?;
  }

  if let Some(FrontendDist::Directory(web_asset_path)) = &config_.build.frontend_dist {
    if !web_asset_path.exists() {
      let absolute_path = web_asset_path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.join(web_asset_path.file_name().unwrap()))
        .unwrap_or_else(|| std::env::current_dir().unwrap().join(web_asset_path));
      return Err(anyhow::anyhow!(
          "Unable to find your web assets, did you forget to build your web app? Your frontendDist is set to \"{}\" (which is `{}`).",
          web_asset_path.display(), absolute_path.display(),
        ));
    }
    if web_asset_path.canonicalize()?.file_name() == Some(std::ffi::OsStr::new("src-tauri")) {
      return Err(anyhow::anyhow!(
          "The configured frontendDist is the `src-tauri` folder. Please isolate your web assets on a separate folder and update `tauri.conf.json > build > frontendDist`.",
        ));
    }

    let mut out_folders = Vec::new();
    for folder in &["node_modules", "src-tauri", "target"] {
      if web_asset_path.join(folder).is_dir() {
        out_folders.push(folder.to_string());
      }
    }
    if !out_folders.is_empty() {
      return Err(anyhow::anyhow!(
          "The configured frontendDist includes the `{:?}` {}. Please isolate your web assets on a separate folder and update `tauri.conf.json > build > frontendDist`.",
          out_folders,
          if out_folders.len() == 1 { "folder" }else { "folders" }
        )
      );
    }
  }

  if options.runner.is_none() {
    options.runner.clone_from(&config_.build.runner);
  }

  options
    .features
    .get_or_insert(Vec::new())
    .extend(config_.build.features.clone().unwrap_or_default());
  interface.build_options(&mut options.args, &mut options.features, mobile);

  Ok(())
}
