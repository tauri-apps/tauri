// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, AppUrl, HookCommand, WebviewUrl, MERGE_CONFIG_EXTENSION_NAME},
    resolve_merge_config,
    updater_signature::{read_key_from_file, secret_key as updater_secret_key, sign_file},
  },
  interface::{AppInterface, AppSettings, Interface},
  CommandExt, Result,
};
use anyhow::{bail, Context};
use base64::Engine;
use clap::{ArgAction, Parser};
use log::{debug, error, info, warn};
use std::{
  env::{set_current_dir, var_os},
  path::{Path, PathBuf},
  process::Command,
};
use tauri_bundler::bundle::{bundle_project, Bundle, PackageType};
use tauri_utils::platform::Target;

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode and generate bundles and installers",
  long_about = "Build your app in release mode and generate bundles and installers. It makes use of the `build.distDir` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.distDir`. This will also run `build.beforeBundleCommand` before generating the bundles and installers of your app."
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
  /// Each bundle must be one of `deb`, `rpm`, `appimage`, `msi`, `app` or `dmg` on MacOS and `updater` on all platforms.
  /// If `none` is specified, the bundler will be skipped.
  ///
  /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub bundles: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Command line arguments passed to the runner. Use `--` to explicitly mark the start of the arguments.
  pub args: Vec<String>,
  /// Skip prompting for values
  #[clap(long)]
  pub ci: bool,
}

pub fn command(mut options: Options, verbosity: u8) -> Result<()> {
  options.ci = options.ci || std::env::var("CI").is_ok();
  let ci = options.ci;

  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let mut interface = setup(target, &mut options, false)?;

  let config = get_config(target, options.config.as_deref())?;
  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

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
              "Unsupported bundle format: {name}"
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
      if config_.tauri.bundle.updater.active && !types.contains(&PackageType::Updater) {
        warn!("The updater is enabled but the bundle target list does not contain `updater`, so the updater artifacts won't be generated.");
      }
    }

    // if we have a package to bundle, let's run the `before_bundle_command`.
    if package_types.as_ref().map_or(true, |p| !p.is_empty()) {
      if let Some(before_bundle) = config_.build.before_bundle_command.clone() {
        run_hook(
          "beforeBundleCommand",
          before_bundle,
          &interface,
          options.debug,
        )?;
      }
    }

    let mut settings = app_settings
      .get_bundler_settings(&options.into(), config_, out_dir, package_types)
      .with_context(|| "failed to build bundler settings")?;

    settings.set_log_level(match verbosity {
      0 => log::Level::Error,
      1 => log::Level::Info,
      _ => log::Level::Trace,
    });

    // set env vars used by the bundler
    #[cfg(target_os = "linux")]
    {
      if config_.tauri.bundle.appimage.bundle_media_framework {
        std::env::set_var("APPIMAGE_BUNDLE_GSTREAMER", "1");
      }
    }

    let bundles = bundle_project(settings)
      .map_err(|e| anyhow::anyhow!("{:#}", e))
      .with_context(|| "failed to bundle project")?;

    let updater_bundles: Vec<&Bundle> = bundles
      .iter()
      .filter(|bundle| bundle.package_type == PackageType::Updater)
      .collect();
    // If updater is active and we bundled it
    if config_.tauri.bundle.updater.active && !updater_bundles.is_empty() {
      // if no password provided we use an empty string
      let password = var_os("TAURI_SIGNING_PRIVATE_KEY_PASSWORD")
        .map(|v| v.to_str().unwrap().to_string())
        .or_else(|| if ci { Some("".into()) } else { None });
      // get the private key
      let secret_key = if let Some(mut private_key) =
        var_os("TAURI_SIGNING_PRIVATE_KEY").map(|v| v.to_str().unwrap().to_string())
      {
        // check if env var points to a file..
        let pk_dir = Path::new(&private_key);
        // Check if user provided a path or a key
        // We validate if the path exist or not.
        if pk_dir.exists() {
          // read file content and use it as private key
          private_key = read_key_from_file(pk_dir)?;
        }
        updater_secret_key(private_key, password)
      } else {
        Err(anyhow::anyhow!("A public key has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable."))
      }?;

      let pubkey =
        base64::engine::general_purpose::STANDARD.decode(&config_.tauri.bundle.updater.pubkey)?;
      let pub_key_decoded = String::from_utf8_lossy(&pubkey);
      let public_key = minisign::PublicKeyBox::from_string(&pub_key_decoded)?.into_public_key()?;

      // make sure we have our package built
      let mut signed_paths = Vec::new();
      for elem in updater_bundles {
        // we expect to have only one path in the vec but we iter if we add
        // another type of updater package who require multiple file signature
        for path in elem.bundle_paths.iter() {
          // sign our path from environment variables
          let (signature_path, signature) = sign_file(&secret_key, path)?;
          if signature.keynum() != public_key.keynum() {
            log::warn!(
              "The updater secret key from `TAURI_PRIVATE_KEY` does not match the public key defined in `tauri.conf.json > tauri > updater > pubkey`. If you are not rotating keys, this means your configuration is wrong and won't be accepted at runtime."
            );
          }
          signed_paths.push(signature_path);
        }
      }

      print_signed_updater_archive(&signed_paths)?;
    }
  }

  Ok(())
}

pub fn setup(target: Target, options: &mut Options, mobile: bool) -> Result<AppInterface> {
  let (merge_config, merge_config_path) = resolve_merge_config(&options.config)?;
  options.config = merge_config;

  let config = get_config(target, options.config.as_deref())?;

  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  let interface = AppInterface::new(config_, options.target.clone())?;

  let bundle_identifier_source = match config_.find_bundle_identifier_overwriter() {
    Some(source) if source == MERGE_CONFIG_EXTENSION_NAME => merge_config_path.unwrap_or(source),
    Some(source) => source,
    None => "tauri.conf.json".into(),
  };

  if config_.tauri.bundle.identifier == "com.tauri.dev" {
    error!(
      "You must change the bundle identifier in `{} > tauri > bundle > identifier`. The default value `com.tauri.dev` is not allowed as it must be unique across applications.",
      bundle_identifier_source
    );
    std::process::exit(1);
  }

  if config_
    .tauri
    .bundle
    .identifier
    .chars()
    .any(|ch| !(ch.is_alphanumeric() || ch == '-' || ch == '.'))
  {
    error!(
      "The bundle identifier \"{}\" set in `{} > tauri > bundle > identifier`. The bundle identifier string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-), and periods (.).",
      config_.tauri.bundle.identifier,
      bundle_identifier_source
    );
    std::process::exit(1);
  }

  if let Some(before_build) = config_.build.before_build_command.clone() {
    run_hook(
      "beforeBuildCommand",
      before_build,
      &interface,
      options.debug,
    )?;
  }

  if let AppUrl::Url(WebviewUrl::App(web_asset_path)) = &config_.build.dist_dir {
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

  options
    .features
    .get_or_insert(Vec::new())
    .extend(config_.build.features.clone().unwrap_or_default());
  interface.build_options(&mut options.args, &mut options.features, mobile);

  Ok(interface)
}

fn run_hook(name: &str, hook: HookCommand, interface: &AppInterface, debug: bool) -> Result<()> {
  let (script, script_cwd) = match hook {
    HookCommand::Script(s) if s.is_empty() => (None, None),
    HookCommand::Script(s) => (Some(s), None),
    HookCommand::ScriptWithOptions { script, cwd } => (Some(script), cwd.map(Into::into)),
  };
  let cwd = script_cwd.unwrap_or_else(|| app_dir().clone());
  if let Some(script) = script {
    info!(action = "Running"; "{} `{}`", name, script);

    let mut env = command_env(debug);
    env.extend(interface.env());

    debug!("Setting environment for hook {:?}", env);

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
      .arg("/S")
      .arg("/C")
      .arg(&script)
      .current_dir(cwd)
      .envs(env)
      .piped()
      .with_context(|| format!("failed to run `{}` with `cmd /C`", script))?;
    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
      .arg("-c")
      .arg(&script)
      .current_dir(cwd)
      .envs(env)
      .piped()
      .with_context(|| format!("failed to run `{script}` with `sh -c`"))?;

    if !status.success() {
      bail!(
        "{} `{}` failed with exit code {}",
        name,
        script,
        status.code().unwrap_or_default()
      );
    }
  }

  Ok(())
}

fn print_signed_updater_archive(output_paths: &[PathBuf]) -> crate::Result<()> {
  use std::fmt::Write;
  if !output_paths.is_empty() {
    let pluralised = if output_paths.len() == 1 {
      "updater signature"
    } else {
      "updater signatures"
    };
    let mut printable_paths = String::new();
    for path in output_paths {
      writeln!(
        printable_paths,
        "        {}",
        tauri_utils::display_path(path)
      )?;
    }
    info!( action = "Finished"; "{} {} at:\n{}", output_paths.len(), pluralised, printable_paths);
  }
  Ok(())
}
