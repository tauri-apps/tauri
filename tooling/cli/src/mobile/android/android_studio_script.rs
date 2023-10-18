// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{detect_target_ok, ensure_init, env, get_app, get_config, read_options, MobileTarget};
use crate::{helpers::config::get as get_tauri_config, Result};
use clap::{ArgAction, Parser};

use cargo_mobile2::{
  android::target::Target,
  opts::Profile,
  target::{call_for_targets_with_fallback, TargetTrait},
};

#[derive(Debug, Parser)]
pub struct Options {
  /// Targets to build.
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  targets: Option<Vec<String>>,
  /// Builds with the release flag
  #[clap(short, long)]
  release: bool,
}

pub fn command(options: Options) -> Result<()> {
  let profile = if options.release {
    Profile::Release
  } else {
    Profile::Debug
  };

  let tauri_config = get_tauri_config(tauri_utils::platform::Target::Android, None)?;

  let (config, metadata, cli_options) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let cli_options = read_options(&tauri_config_.tauri.bundle.identifier);
    let (config, metadata) = get_config(&get_app(tauri_config_), tauri_config_, &cli_options);
    (config, metadata, cli_options)
  };
  ensure_init(config.project_dir(), MobileTarget::Android)?;

  let env = env()?;

  call_for_targets_with_fallback(
    options.targets.unwrap_or_default().iter(),
    &detect_target_ok,
    &env,
    |target: &Target| {
      target
        .build(
          &config,
          &metadata,
          &env,
          cli_options.noise_level,
          true,
          profile,
        )
        .map_err(Into::into)
    },
  )
  .map_err(|e| anyhow::anyhow!(e.to_string()))?
}
