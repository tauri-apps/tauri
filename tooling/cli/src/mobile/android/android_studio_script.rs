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
      &get_app(
        MobileTarget::Android,
        tauri_config_,
        &AppInterface::new(tauri_config_, None)?,
      ),
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
      log::info!("Forwarding port {port} with adb");

      let devices = adb::device_list(&env).unwrap_or_default();

      // clear port forwarding for all devices
      for device in &devices {
        remove_adb_reverse(&env, device.serial_no(), &forward);
      }

      // if there's a known target, we should force use it
      if let Some(target_device) = &cli_options.target_device {
        run_adb_reverse(&env, &target_device.id, &forward, &forward).with_context(|| {
          format!(
            "failed to forward port with adb, is the {} device connected?",
            target_device.name,
          )
        })?;
      } else if devices.len() == 1 {
        let device = devices.first().unwrap();
        run_adb_reverse(&env, device.serial_no(), &forward, &forward).with_context(|| {
          format!(
            "failed to forward port with adb, is the {} device connected?",
            device.name(),
          )
        })?;
      } else if devices.len() > 1 {
        anyhow::bail!("Multiple Android devices are connected ({}), please disconnect devices you do not intend to use so Tauri can determine which to use",
      devices.iter().map(|d| d.name()).collect::<Vec<_>>().join(", "));
      }
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

fn run_adb_reverse(
  env: &cargo_mobile2::android::env::Env,
  device_serial_no: &str,
  remote: &str,
  local: &str,
) -> std::io::Result<std::process::Output> {
  adb::adb(env, ["-s", device_serial_no, "reverse", remote, local])
    .stdin_file(os_pipe::dup_stdin().unwrap())
    .stdout_file(os_pipe::dup_stdout().unwrap())
    .stderr_file(os_pipe::dup_stdout().unwrap())
    .run()
}

fn remove_adb_reverse(
  env: &cargo_mobile2::android::env::Env,
  device_serial_no: &str,
  remote: &str,
) {
  // ignore errors in case the port is not forwarded
  let _ = adb::adb(env, ["-s", device_serial_no, "reverse", "--remove", remote])
    .stdin_file(os_pipe::dup_stdin().unwrap())
    .stdout_file(os_pipe::dup_stdout().unwrap())
    .stderr_file(os_pipe::dup_stdout().unwrap())
    .run();
}
