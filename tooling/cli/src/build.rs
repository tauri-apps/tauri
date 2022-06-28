// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, AppUrl, WindowUrl},
    manifest::rewrite_manifest,
    updater_signature::sign_file_from_env_variables,
  },
  interface::{AppInterface, AppSettings, Interface},
  CommandExt, Result,
};
use anyhow::{bail, Context};
use clap::Parser;
use log::warn;
use log::{error, info};
use std::{env::set_current_dir, path::PathBuf, process::Command};
use tauri_bundler::bundle::{bundle_project, PackageType};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Tauri build")]
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
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// Space or comma separated list of bundles to package.
  ///
  /// Each bundle must be one of `deb`, `appimage`, `msi`, `app` or `dmg` on MacOS and `updater` on all platforms.
  /// If `none` is specified, the bundler will be skipped.
  ///
  /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub bundles: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Command line arguments passed to the runner
  pub args: Vec<String>,
}

pub fn command(mut options: Options) -> Result<()> {
  let merge_config = if let Some(config) = &options.config {
    Some(if config.starts_with('{') {
      config.to_string()
    } else {
      std::fs::read_to_string(&config).with_context(|| "failed to read custom configuration")?
    })
  } else {
    None
  };

  let tauri_path = tauri_dir();
  set_current_dir(&tauri_path).with_context(|| "failed to change current working directory")?;

  let config = get_config(merge_config.as_deref())?;

  let manifest = rewrite_manifest(config.clone())?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  if config_.tauri.bundle.identifier == "com.tauri.dev" {
    error!("You must change the bundle identifier in `tauri.conf.json > tauri > bundle > identifier`. The default value `com.tauri.dev` is not allowed as it must be unique across applications.");
    std::process::exit(1);
  }

  if config_
    .tauri
    .bundle
    .identifier
    .chars()
    .any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '.'))
  {
    error!("You must change the bundle identifier in `tauri.conf.json > tauri > bundle > identifier`. The bundle identifier string must contain only alphanumeric characters (A–Z, a–z, and 0–9), hyphens (-), and periods (.).");
    std::process::exit(1);
  }

  if let Some(before_build) = &config_.build.before_build_command {
    if !before_build.is_empty() {
      info!(action = "Running"; "beforeBuildCommand `{}`", before_build);
      #[cfg(target_os = "windows")]
      let status = Command::new("cmd")
        .arg("/S")
        .arg("/C")
        .arg(before_build)
        .current_dir(app_dir())
        .envs(command_env(options.debug))
        .piped()
        .with_context(|| format!("failed to run `{}` with `cmd /C`", before_build))?;
      #[cfg(not(target_os = "windows"))]
      let status = Command::new("sh")
        .arg("-c")
        .arg(before_build)
        .current_dir(app_dir())
        .envs(command_env(options.debug))
        .piped()
        .with_context(|| format!("failed to run `{}` with `sh -c`", before_build))?;

      if !status.success() {
        bail!(
          "beforeDevCommand `{}` failed with exit code {}",
          before_build,
          status.code().unwrap_or_default()
        );
      }
    }
  }

  if let AppUrl::Url(WindowUrl::App(web_asset_path)) = &config_.build.dist_dir {
    if !web_asset_path.exists() {
      return Err(anyhow::anyhow!(
          "Unable to find your web assets, did you forget to build your web app? Your distDir is set to \"{:?}\".",
          web_asset_path
        ));
    }
    if web_asset_path.canonicalize()?.file_name() == Some(std::ffi::OsStr::new("src-tauri")) {
      return Err(anyhow::anyhow!(
            "The configured distDir is the `src-tauri` folder.
            Please isolate your web assets on a separate folder and update `tauri.conf.json > build > distDir`.",
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
            "The configured distDir includes the `{:?}` {}. Please isolate your web assets on a separate folder and update `tauri.conf.json > build > distDir`.",
            out_folders,
            if out_folders.len() == 1 { "folder" }else { "folders" }
          )
        );
    }
  }

  if options.runner.is_none() {
    options.runner = config_.build.runner.clone();
  }

  if let Some(list) = options.features.as_mut() {
    list.extend(config_.build.features.clone().unwrap_or_default());
  }

  let mut interface = AppInterface::new(config_)?;
  let app_settings = interface.app_settings();
  let interface_options = options.clone().into();

  let bin_path = app_settings.app_binary_path(&interface_options)?;
  let out_dir = bin_path.parent().unwrap();

  interface.build(interface_options)?;

  let app_settings = interface.app_settings();

  if config_.tauri.bundle.active {
    let package_types = if let Some(names) = &options.bundles {
      let mut types = vec![];
      for name in names
        .iter()
        .flat_map(|n| n.split(',').map(|s| s.to_string()).collect::<Vec<String>>())
      {
        if name == "none" {
          break;
        }
        match PackageType::from_short_name(&name) {
          Some(package_type) => {
            types.push(package_type);
          }
          None => {
            return Err(anyhow::anyhow!(format!(
              "Unsupported bundle format: {}",
              name
            )));
          }
        }
      }
      Some(types)
    } else {
      let targets = config_.tauri.bundle.targets.to_vec();
      if targets.is_empty() {
        None
      } else {
        Some(targets.into_iter().map(Into::into).collect())
      }
    };

    if let Some(types) = &package_types {
      if config_.tauri.updater.active && !types.contains(&PackageType::Updater) {
        warn!("The updater is enabled but the bundle target list does not contain `updater`, so the updater artifacts won't be generated.");
      }
    }

    let settings = app_settings
      .get_bundler_settings(&options.into(), &manifest, config_, out_dir, package_types)
      .with_context(|| "failed to build bundler settings")?;

    // set env vars used by the bundler
    #[cfg(target_os = "linux")]
    {
      use crate::helpers::config::ShellAllowlistOpen;
      if matches!(
        config_.tauri.allowlist.shell.open,
        ShellAllowlistOpen::Flag(true) | ShellAllowlistOpen::Validate(_)
      ) {
        std::env::set_var("APPIMAGE_BUNDLE_XDG_OPEN", "1");
      }
      if config_.tauri.system_tray.is_some() {
        if let Ok(tray) = std::env::var("TAURI_TRAY") {
          std::env::set_var(
            "TRAY_LIBRARY_PATH",
            if tray == "ayatana" {
              format!(
                "{}/libayatana-appindicator3.so.1",
                pkgconfig_utils::get_library_path("ayatana-appindicator3-0.1")
                  .expect("failed to get ayatana-appindicator library path using pkg-config.")
              )
            } else {
              format!(
                "{}/libappindicator3.so.1",
                pkgconfig_utils::get_library_path("appindicator3-0.1")
                  .expect("failed to get libappindicator-gtk library path using pkg-config.")
              )
            },
          );
        } else {
          std::env::set_var(
            "TRAY_LIBRARY_PATH",
            pkgconfig_utils::get_appindicator_library_path(),
          );
        }
      }
    }
    if config_.tauri.bundle.appimage.bundle_media_framework {
      std::env::set_var("APPIMAGE_BUNDLE_GSTREAMER", "1");
    }

    let bundles = bundle_project(settings).with_context(|| "failed to bundle project")?;

    // If updater is active
    if config_.tauri.updater.active {
      // make sure we have our package builts
      let mut signed_paths = Vec::new();
      for elem in bundles
        .iter()
        .filter(|bundle| bundle.package_type == PackageType::Updater)
      {
        // we expect to have only one path in the vec but we iter if we add
        // another type of updater package who require multiple file signature
        for path in elem.bundle_paths.iter() {
          // sign our path from environment variables
          let (signature_path, _signature) = sign_file_from_env_variables(path)?;
          signed_paths.append(&mut vec![signature_path]);
        }
      }

      if !signed_paths.is_empty() {
        print_signed_updater_archive(&signed_paths)?;
      }
    }
  }

  Ok(())
}

fn print_signed_updater_archive(output_paths: &[PathBuf]) -> crate::Result<()> {
  let pluralised = if output_paths.len() == 1 {
    "updater archive"
  } else {
    "updater archives"
  };
  let msg = format!("{} {} at:", output_paths.len(), pluralised);
  info!("{}", msg);
  for path in output_paths {
    info!("        {}", path.display());
  }
  Ok(())
}

#[cfg(target_os = "linux")]
mod pkgconfig_utils {
  use std::{path::PathBuf, process::Command};

  pub fn get_appindicator_library_path() -> PathBuf {
    match get_library_path("ayatana-appindicator3-0.1") {
      Some(p) => format!("{}/libayatana-appindicator3.so.1", p).into(),
      None => match get_library_path("appindicator3-0.1") {
        Some(p) => format!("{}/libappindicator3.so.1", p).into(),
        None => panic!("Can't detect any appindicator library"),
      },
    }
  }

  /// Gets the folder in which a library is located using `pkg-config`.
  pub fn get_library_path(name: &str) -> Option<String> {
    let mut cmd = Command::new("pkg-config");
    cmd.env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1");
    cmd.arg("--libs-only-L");
    cmd.arg(name);
    if let Ok(output) = cmd.output() {
      if !output.stdout.is_empty() {
        // output would be "-L/path/to/library\n"
        let word = output.stdout[2..].to_vec();
        return Some(String::from_utf8_lossy(&word).trim().to_string());
      } else {
        None
      }
    } else {
      None
    }
  }
}
