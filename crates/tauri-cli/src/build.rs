// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::BundleFormat,
  error::{Context, ErrorExt},
  helpers::{
    self,
    app_paths::{frontend_dir, tauri_dir},
    config::{get as get_config, ConfigMetadata, FrontendDist},
  },
  info::plugins::check_mismatched_packages,
  interface::{rust::get_cargo_target_dir, AppInterface, Interface},
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};
use std::env::set_current_dir;
use tauri_utils::config::RunnerConfig;
use tauri_utils::platform::Target;

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode and generate bundles and installers",
  long_about = "Build your app in release mode and generate bundles and installers. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`. This will also run `build.beforeBundleCommand` before generating the bundles and installers of your app."
)]
pub struct Options {
  /// Binary to use to build the application, defaults to `cargo`
  #[clap(short, long)]
  pub runner: Option<RunnerConfig>,
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
  pub features: Vec<String>,
  /// Space or comma separated list of bundles to package.
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub bundles: Option<Vec<BundleFormat>>,
  /// Skip the bundling step even if `bundle > active` is `true` in tauri config.
  #[clap(long)]
  pub no_bundle: bool,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Command line arguments passed to the runner. Use `--` to explicitly mark the start of the arguments.
  pub args: Vec<String>,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Whether to wait for notarization to finish and `staple` the ticket onto the app.
  ///
  /// Gatekeeper will look for stapled tickets to tell whether your app was notarized without
  /// reaching out to Apple's servers which is helpful in offline environments.
  ///
  /// Enabling this option will also result in `tauri build` not waiting for notarization to finish
  /// which is helpful for the very first time your app is notarized as this can take multiple hours.
  /// On subsequent runs, it's recommended to disable this setting again.
  #[clap(long)]
  pub skip_stapling: bool,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
  /// Skip code signing when bundling the app
  #[clap(long)]
  pub no_sign: bool,
}

pub fn command(mut options: Options, verbosity: u8) -> Result<()> {
  crate::helpers::app_paths::resolve();

  if options.no_sign {
    log::warn!("--no-sign flag detected: Signing will be skipped.");
  }

  let ci = options.ci;

  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let config = get_config(
    target,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
  )?;

  let mut interface = AppInterface::new(
    config.lock().unwrap().as_ref().unwrap(),
    options.target.clone(),
  )?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  setup(&interface, &mut options, config_, false)?;

  if let Some(minimum_system_version) = &config_.bundle.macos.minimum_system_version {
    std::env::set_var("MACOSX_DEPLOYMENT_TARGET", minimum_system_version);
  }

  let app_settings = interface.app_settings();
  let interface_options = options.clone().into();

  let out_dir = app_settings.out_dir(&interface_options)?;

  let bin_path = interface.build(interface_options)?;

  log::info!(action ="Built"; "application at: {}", tauri_utils::display_path(bin_path));

  let app_settings = interface.app_settings();

  if !options.no_bundle && (config_.bundle.active || options.bundles.is_some()) {
    crate::bundle::bundle(
      &options.into(),
      verbosity,
      ci,
      &interface,
      &*app_settings,
      config_,
      &out_dir,
    )?;
  }

  Ok(())
}

pub fn setup(
  interface: &AppInterface,
  options: &mut Options,
  config: &ConfigMetadata,
  mobile: bool,
) -> Result<()> {
  let tauri_path = tauri_dir();

  // TODO: Maybe optimize this to run in parallel in the future
  // see https://github.com/tauri-apps/tauri/pull/13993#discussion_r2280697117
  log::info!("Looking up installed tauri packages to check mismatched versions...");
  if let Err(error) = check_mismatched_packages(frontend_dir(), tauri_path) {
    if options.ignore_version_mismatches {
      log::error!("{error}");
    } else {
      return Err(error);
    }
  }

  set_current_dir(tauri_path).context("failed to set current directory")?;

  let bundle_identifier_source = config
    .find_bundle_identifier_overwriter()
    .unwrap_or_else(|| "tauri.conf.json".into());

  if config.identifier == "com.tauri.dev" {
    crate::error::bail!(
      "You must change the bundle identifier in `{bundle_identifier_source} identifier`. The default value `com.tauri.dev` is not allowed as it must be unique across applications.",
    );
  }

  if config
    .identifier
    .chars()
    .any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '.'))
  {
    crate::error::bail!(
      "The bundle identifier \"{}\" set in `{bundle_identifier_source:?} identifier`. The bundle identifier string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-), and periods (.).",
      config.identifier,
    );
  }

  if config.identifier.ends_with(".app") {
    log::warn!(
      "The bundle identifier \"{}\" set in `{bundle_identifier_source:?} identifier` ends with `.app`. This is not recommended because it conflicts with the application bundle extension on macOS.",
      config.identifier,
    );
  }

  if let Some(before_build) = config.build.before_build_command.clone() {
    helpers::run_hook("beforeBuildCommand", before_build, interface, options.debug)?;
  }

  if let Some(FrontendDist::Directory(web_asset_path)) = &config.build.frontend_dist {
    if !web_asset_path.exists() {
      let absolute_path = web_asset_path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.join(web_asset_path.file_name().unwrap()))
        .unwrap_or_else(|| std::env::current_dir().unwrap().join(web_asset_path));
      crate::error::bail!(
        "Unable to find your web assets, did you forget to build your web app? Your frontendDist is set to \"{}\" (which is `{}`).",
        web_asset_path.display(), absolute_path.display(),
      );
    }
    if web_asset_path
      .canonicalize()
      .fs_context("failed to canonicalize path", web_asset_path.to_path_buf())?
      .file_name()
      == Some(std::ffi::OsStr::new("src-tauri"))
    {
      crate::error::bail!(
          "The configured frontendDist is the `src-tauri` folder. Please isolate your web assets on a separate folder and update `tauri.conf.json > build > frontendDist`.",
        );
    }

    // Issue #13287 - Allow the use of target dir inside frontendDist/distDir
    // https://github.com/tauri-apps/tauri/issues/13287
    let target_path = get_cargo_target_dir(&options.args)?;
    let mut out_folders = Vec::new();
    if let Ok(web_asset_canonical) = dunce::canonicalize(web_asset_path) {
      if let Ok(relative_path) = target_path.strip_prefix(&web_asset_canonical) {
        let relative_str = relative_path.to_string_lossy();
        if !relative_str.is_empty() {
          out_folders.push(relative_str.to_string());
        }
      }

      for folder in &["node_modules", "src-tauri"] {
        let sub_path = web_asset_canonical.join(folder);
        if sub_path.is_dir() {
          out_folders.push(folder.to_string());
        }
      }
    }

    if !out_folders.is_empty() {
      crate::error::bail!(
        "The configured frontendDist includes the `{:?}` {}. Please isolate your web assets on a separate folder and update `tauri.conf.json > build > frontendDist`.",
        out_folders,
        if out_folders.len() == 1 { "folder" } else { "folders" }
      );
    }
  }

  if options.runner.is_none() {
    options.runner = config.build.runner.clone();
  }

  options
    .features
    .extend_from_slice(config.build.features.as_deref().unwrap_or_default());
  interface.build_options(&mut options.args, &mut options.features, mobile);

  Ok(())
}
