// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    command_env,
    config::{get as get_config, ConfigHandle, FrontendDist, HookCommand},
    updater_signature::{secret_key as updater_secret_key, sign_file},
  },
  interface::{AppInterface, AppSettings, Interface},
  CommandExt, ConfigValue, Result,
};
use anyhow::{bail, Context};
use base64::Engine;
use clap::{ArgAction, Parser};
use log::{debug, error, info, warn};
use std::{
  env::{set_current_dir, var},
  path::{Path, PathBuf},
  process::Command,
};
use tauri_bundler::bundle::{bundle_project, Bundle, PackageType};
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
  /// Each bundle must be one of `deb`, `rpm`, `appimage`, `msi`, `app` or `dmg` on MacOS and `updater` on all platforms.
  /// If `none` is specified, the bundler will be skipped.
  ///
  /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub bundles: Option<Vec<String>>,
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

  let bin_path = app_settings.app_binary_path(&interface_options)?;
  let out_dir = bin_path.parent().unwrap();

  interface.build(interface_options)?;

  let app_settings = interface.app_settings();

  if config_.bundle.active {
    let package_types = if let Some(names) = &options.bundles {
      let mut types = vec![];
      let mut skip = false;
      for name in names {
        if name == "none" {
          skip = true;
          break;
        }

        match PackageType::from_short_name(name) {
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

      if skip {
        None
      } else {
        Some(types)
      }
    } else {
      let targets = config_.bundle.targets.to_vec();
      if targets.is_empty() {
        None
      } else {
        Some(targets.into_iter().map(Into::into).collect())
      }
    };

    let updater_pub_key = config_
      .plugins
      .0
      .get("updater")
      .and_then(|k| k.get("pubkey"))
      .and_then(|v| v.as_str())
      .map(|v| v.to_string());
    if let Some(types) = &package_types {
      if updater_pub_key
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false)
        && !types.contains(&PackageType::Updater)
      {
        warn!("`plugins > updater > pubkey` is set, but the bundle target list does not contain `updater`, so the updater artifacts won't be generated.");
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
      if config_.bundle.linux.appimage.bundle_media_framework {
        std::env::set_var("APPIMAGE_BUNDLE_GSTREAMER", "1");
      }

      if let Some(open) = config_.plugins.0.get("shell").and_then(|v| v.get("open")) {
        if open.as_bool().is_some_and(|x| x) || open.is_string() {
          std::env::set_var("APPIMAGE_BUNDLE_XDG_OPEN", "1");
        }
      }

      if settings.deep_link_protocols().is_some() {
        std::env::set_var("APPIMAGE_BUNDLE_XDG_MIME", "1");
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
    if !updater_bundles.is_empty() {
      if let Some(pubkey) = updater_pub_key {
        // get the public key
        // check if pubkey points to a file...
        let maybe_path = Path::new(&pubkey);
        let pubkey = if maybe_path.exists() {
          std::fs::read_to_string(maybe_path)?
        } else {
          pubkey
        };

        // if no password provided we use an empty string
        let password = var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD").ok().or_else(|| {
          if ci {
            Some("".into())
          } else {
            None
          }
        });

        // get the private key
        let secret_key = match var("TAURI_SIGNING_PRIVATE_KEY") {
        Ok(private_key) => {
          // check if private_key points to a file...
          let maybe_path = Path::new(&private_key);
          let private_key = if maybe_path.exists() {
            std::fs::read_to_string(maybe_path)?
          } else {
            private_key
          };
          updater_secret_key(private_key, password)
        }
        _ => Err(anyhow::anyhow!("A public key has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable.")),
      }?;

        let pubkey = base64::engine::general_purpose::STANDARD.decode(pubkey)?;
        let pub_key_decoded = String::from_utf8_lossy(&pubkey);
        let public_key =
          minisign::PublicKeyBox::from_string(&pub_key_decoded)?.into_public_key()?;

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
              "The updater secret key from `TAURI_PRIVATE_KEY` does not match the public key from `plugins > updater > pubkey`. If you are not rotating keys, this means your configuration is wrong and won't be accepted at runtime when performing update."
            );
            }
            signed_paths.push(signature_path);
          }
        }

        print_signed_updater_archive(&signed_paths)?;
      }
    }
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
    error!(
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
    error!(
      "The bundle identifier \"{}\" set in `{} identifier`. The bundle identifier string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-), and periods (.).",
      config_.identifier,
      bundle_identifier_source
    );
    std::process::exit(1);
  }

  if let Some(before_build) = config_.build.before_build_command.clone() {
    run_hook("beforeBuildCommand", before_build, interface, options.debug)?;
  }

  if let Some(FrontendDist::Directory(web_asset_path)) = &config_.build.frontend_dist {
    if !web_asset_path.exists() {
      return Err(anyhow::anyhow!(
          "Unable to find your web assets, did you forget to build your web app? Your frontendDist is set to \"{:?}\".",
          web_asset_path
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
    options.runner = config_.build.runner.clone();
  }

  options
    .features
    .get_or_insert(Vec::new())
    .extend(config_.build.features.clone().unwrap_or_default());
  interface.build_options(&mut options.args, &mut options.features, mobile);

  Ok(())
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
