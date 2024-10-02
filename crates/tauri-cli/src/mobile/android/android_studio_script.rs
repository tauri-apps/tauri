// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{detect_target_ok, ensure_init, env, get_app, get_config, read_options, MobileTarget};
use crate::{
  helpers::config::get as get_tauri_config,
  interface::{AppInterface, Interface},
  mobile::CliOptions,
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

    if let Some(url) = dev_url {
      let localhost = match url.host() {
        Some(url::Host::Domain(d)) => d == "localhost",
        Some(url::Host::Ipv4(i)) => i == std::net::Ipv4Addr::LOCALHOST,
        _ => false,
      };

      if localhost {
        if let Some(port) = url.port_or_known_default() {
          adb_forward_port(port, &env, &cli_options)?;
        }
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

fn adb_forward_port(
  port: u16,
  env: &cargo_mobile2::android::env::Env,
  cli_options: &CliOptions,
) -> Result<()> {
  let forward = format!("tcp:{port}");
  log::info!("Forwarding port {port} with adb");

  let mut devices = adb::device_list(env).unwrap_or_default();
  // if we could not detect any running device, let's wait a few seconds, it might be booting up
  if devices.is_empty() {
    log::warn!(
      "ADB device list is empty, waiting a few seconds to see if there's any booting device..."
    );

    let max = 5;
    let mut count = 0;
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      devices = adb::device_list(env).unwrap_or_default();
      if !devices.is_empty() {
        break;
      }

      count += 1;
      if count == max {
        break;
      }
    }
  }

  let target_device = if let Some(target_device) = &cli_options.target_device {
    Some((target_device.id.clone(), target_device.name.clone()))
  } else if devices.len() == 1 {
    let device = devices.first().unwrap();
    Some((device.serial_no().to_string(), device.name().to_string()))
  } else if devices.len() > 1 {
    anyhow::bail!("Multiple Android devices are connected ({}), please disconnect devices you do not intend to use so Tauri can determine which to use",
      devices.iter().map(|d| d.name()).collect::<Vec<_>>().join(", "));
  } else {
    // when building the app without running to a device, we might have an empty devices list
    None
  };

  if let Some((target_device_serial_no, target_device_name)) = target_device {
    let mut already_forwarded = false;

    // clear port forwarding for all devices
    for device in &devices {
      let reverse_list_output = adb_reverse_list(env, device.serial_no())?;

      // check if the device has the port forwarded
      if String::from_utf8_lossy(&reverse_list_output.stdout).contains(&forward) {
        // device matches our target, we can skip forwarding
        if device.serial_no() == target_device_serial_no {
          log::debug!(
            "device {} already has the forward for {}",
            device.name(),
            forward
          );
          already_forwarded = true;
        }
        break;
      }
    }

    // if there's a known target, we should forward the port to it
    if already_forwarded {
      log::info!("{forward} already forwarded to {target_device_name}");
    } else {
      loop {
        run_adb_reverse(env, &target_device_serial_no, &forward, &forward).with_context(|| {
          format!("failed to forward port with adb, is the {target_device_name} device connected?",)
        })?;

        let reverse_list_output = adb_reverse_list(env, &target_device_serial_no)?;
        // wait and retry until the port has actually been forwarded
        if String::from_utf8_lossy(&reverse_list_output.stdout).contains(&forward) {
          break;
        } else {
          log::warn!(
            "waiting for the port to be forwarded to {}...",
            target_device_name
          );
          std::thread::sleep(std::time::Duration::from_secs(1));
        }
      }
    }
  } else {
    log::warn!("no running devices detected with ADB; skipping port forwarding");
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

fn adb_reverse_list(
  env: &cargo_mobile2::android::env::Env,
  device_serial_no: &str,
) -> std::io::Result<std::process::Output> {
  adb::adb(env, ["-s", device_serial_no, "reverse", "--list"])
    .stdin_file(os_pipe::dup_stdin().unwrap())
    .stdout_capture()
    .stderr_capture()
    .run()
}
