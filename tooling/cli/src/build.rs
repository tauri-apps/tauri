// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  command_env,
  config::{get as get_config, AppUrl, WindowUrl},
  manifest::rewrite_manifest,
  updater_signature::sign_file_from_env_variables,
};
use crate::{CommandExt, Result};
use anyhow::{bail, Context};
use clap::Parser;
#[cfg(target_os = "linux")]
use heck::ToKebabCase;
use log::{error, info};
use std::{env::set_current_dir, fs::rename, path::PathBuf, process::Command};
use tauri_bundler::bundle::{bundle_project, PackageType};

#[derive(Debug, Parser)]
#[clap(about = "Tauri build")]
pub struct Options {
  /// Binary to use to build the application, defaults to `cargo`
  #[clap(short, long)]
  runner: Option<String>,
  /// Builds with the debug flag
  #[clap(short, long)]
  debug: bool,
  /// Target triple to build against.
  /// It must be one of the values outputted by `$rustc --print target-list` or `universal-apple-darwin` for an universal macOS application.
  /// Note that compiling an universal macOS application requires both `aarch64-apple-darwin` and `x86_64-apple-darwin` targets to be installed.
  #[clap(short, long)]
  target: Option<String>,
  /// Space or comma separated list of features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  features: Option<Vec<String>>,
  /// Space or comma separated list of bundles to package
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  bundles: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  config: Option<String>,
  /// Command line arguments passed to the runner
  args: Vec<String>,
}

pub fn command(options: Options) -> Result<()> {
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
        .pipe()?
        .status()
        .with_context(|| format!("failed to run `{}` with `cmd /C`", before_build))?;
      #[cfg(not(target_os = "windows"))]
      let status = Command::new("sh")
        .arg("-c")
        .arg(before_build)
        .current_dir(app_dir())
        .envs(command_env(options.debug))
        .pipe()?
        .status()
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

  let runner_from_config = config_.build.runner.clone();
  let runner = options
    .runner
    .or(runner_from_config)
    .unwrap_or_else(|| "cargo".to_string());

  let mut features = config_.build.features.clone().unwrap_or_default();
  if let Some(list) = options.features {
    features.extend(list);
  }

  let mut args = Vec::new();
  if !options.args.is_empty() {
    args.extend(options.args);
  }

  if !features.is_empty() {
    args.push("--features".into());
    args.push(features.join(","));
  }

  if !options.debug {
    args.push("--release".into());
  }

  let app_settings = crate::interface::rust::AppSettings::new(config_)?;

  let out_dir = app_settings
    .get_out_dir(options.target.clone(), options.debug)
    .with_context(|| "failed to get project out directory")?;

  let bin_name = app_settings
    .cargo_package_settings()
    .name
    .clone()
    .expect("Cargo manifest must have the `package.name` field");

  let target: String = if let Some(target) = options.target.clone() {
    target
  } else {
    tauri_utils::platform::target_triple()?
  };
  let binary_extension: String = if target.contains("windows") {
    "exe"
  } else {
    ""
  }
  .into();

  let bin_path = out_dir.join(&bin_name).with_extension(&binary_extension);

  let no_default_features = args.contains(&"--no-default-features".into());

  if options.target == Some("universal-apple-darwin".into()) {
    std::fs::create_dir_all(&out_dir).with_context(|| "failed to create project out directory")?;

    let mut lipo_cmd = Command::new("lipo");
    lipo_cmd
      .arg("-create")
      .arg("-output")
      .arg(out_dir.join(&bin_name));
    for triple in ["aarch64-apple-darwin", "x86_64-apple-darwin"] {
      let mut args_ = args.clone();
      args_.push("--target".into());
      args_.push(triple.into());
      crate::interface::rust::build_project(runner.clone(), args_)
        .with_context(|| format!("failed to build {} binary", triple))?;
      let triple_out_dir = app_settings
        .get_out_dir(Some(triple.into()), options.debug)
        .with_context(|| format!("failed to get {} out dir", triple))?;
      lipo_cmd.arg(triple_out_dir.join(&bin_name));
    }

    let lipo_status = lipo_cmd.status()?;
    if !lipo_status.success() {
      return Err(anyhow::anyhow!(format!(
        "Result of `lipo` command was unsuccessful: {}. (Is `lipo` installed?)",
        lipo_status
      )));
    }
  } else {
    if let Some(target) = &options.target {
      args.push("--target".into());
      args.push(target.clone());
    }
    crate::interface::rust::build_project(runner, args).with_context(|| "failed to build app")?;
  }

  if let Some(product_name) = config_.package.product_name.clone() {
    #[cfg(target_os = "linux")]
    let product_name = product_name.to_kebab_case();

    let product_path = out_dir
      .join(&product_name)
      .with_extension(&binary_extension);

    rename(&bin_path, &product_path).with_context(|| {
      format!(
        "failed to rename `{}` to `{}`",
        bin_path.display(),
        product_path.display(),
      )
    })?;
  }

  if config_.tauri.bundle.active {
    let package_types = if let Some(names) = options.bundles {
      let mut types = vec![];
      for name in names
        .into_iter()
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
    } else if let Some(targets) = &config_.tauri.bundle.targets {
      let mut types = vec![];
      let targets = targets.to_vec();
      if !targets.contains(&"all".into()) {
        for name in targets {
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
        None
      }
    } else {
      None
    };

    let mut enabled_features = features.clone();
    if !no_default_features {
      enabled_features.push("default".into());
    }
    let settings = crate::interface::get_bundler_settings(
      app_settings,
      target,
      &enabled_features,
      &manifest,
      config_,
      &out_dir,
      package_types,
    )
    .with_context(|| "failed to build bundler settings")?;

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
