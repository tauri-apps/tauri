// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{detect_target_ok, ensure_init, env, get_app, get_config, read_options, MobileTarget};
use crate::{
  helpers::config::get as get_tauri_config,
  interface::{AppInterface, Interface},
  Result,
};
use clap::{ArgAction, Parser};

use anyhow::Context;
use cargo_mobile2::{
  android::{adb, target::Target},
  opts::Profile,
  target::{call_for_targets_with_fallback, TargetTrait},
};

use std::path::Path;

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
  crate::helpers::app_paths::resolve();

  let profile = if options.release {
    Profile::Release
  } else {
    Profile::Debug
  };

  let tauri_config = get_tauri_config(tauri_utils::platform::Target::Android, None)?;

  let (config, metadata, cli_options) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    let cli_options = read_options(&tauri_config_.identifier);
    let (config, metadata) = get_config(
      &get_app(tauri_config_, &AppInterface::new(tauri_config_, None)?),
      tauri_config_,
      None,
      &cli_options,
    );
    (config, metadata, cli_options)
  };

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::Android,
  )?;

  if let Some(config) = &cli_options.config {
    crate::helpers::config::merge_with(&config.0)?;
  }

  let env = env()?;

  if cli_options.dev {
    let dev_url = tauri_config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .dev_url
      .clone();

    if let Some(port) = dev_url.and_then(|url| url.port_or_known_default()) {
      let forward = format!("tcp:{port}");
      // ignore errors in case we do not have a device available
      let _ = adb::adb(&env, ["reverse", &forward, &forward])
        .stdin_file(os_pipe::dup_stdin().unwrap())
        .stdout_file(os_pipe::dup_stdout().unwrap())
        .stderr_file(os_pipe::dup_stdout().unwrap())
        .run();
    }
  }

  let mut validated_lib = false;

  call_for_targets_with_fallback(
    options.targets.unwrap_or_default().iter(),
    &detect_target_ok,
    &env,
    |target: &Target| {
      target.build(
        &config,
        &metadata,
        &env,
        cli_options.noise_level,
        true,
        profile,
      )?;

      if !validated_lib {
        validated_lib = true;

        let lib_path = config
          .app()
          .target_dir(target.triple, profile)
          .join(config.so_name());

        validate_lib(&lib_path)?;
      }

      Ok(())
    },
  )
  .map_err(|e| anyhow::anyhow!(e.to_string()))?
}

fn validate_lib(path: &Path) -> Result<()> {
  let so_bytes = std::fs::read(path)?;
  let elf = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(&so_bytes)
    .context("failed to parse ELF")?;
  let (symbol_table, string_table) = elf
    .dynamic_symbol_table()
    .context("failed to read dynsym section")?
    .context("missing dynsym tables")?;

  let mut symbols = Vec::new();
  for s in symbol_table.iter() {
    if let Ok(symbol) = string_table.get(s.st_name as usize) {
      symbols.push(symbol);
    }
  }

  if !symbols.contains(&"Java_app_tauri_plugin_PluginManager_handlePluginResponse") {
    anyhow::bail!(
      "Library from {} does not include required runtime symbols. This means you are likely missing the tauri::mobile_entry_point macro usage, see the documentation for more information: https://v2.tauri.app/start/migrate/from-tauri-1",
      path.display()
    );
  }

  Ok(())
}
