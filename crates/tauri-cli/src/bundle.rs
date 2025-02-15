// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::{Path, PathBuf},
  str::FromStr,
  sync::OnceLock,
};

use anyhow::Context;
use clap::{builder::PossibleValue, ArgAction, Parser, ValueEnum};
use tauri_bundler::PackageType;
use tauri_utils::platform::Target;

use crate::{
  helpers::{
    self,
    app_paths::tauri_dir,
    config::{get as get_config, ConfigMetadata},
    updater_signature,
  },
  interface::{AppInterface, AppSettings, Interface},
  ConfigValue,
};

#[derive(Debug, Clone)]
pub struct BundleFormat(PackageType);

impl FromStr for BundleFormat {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> crate::Result<Self> {
    PackageType::from_short_name(s)
      .map(Self)
      .ok_or_else(|| anyhow::anyhow!("unknown bundle format {s}"))
  }
}

impl ValueEnum for BundleFormat {
  fn value_variants<'a>() -> &'a [Self] {
    static VARIANTS: OnceLock<Vec<BundleFormat>> = OnceLock::new();
    VARIANTS.get_or_init(|| PackageType::all().iter().map(|t| Self(*t)).collect())
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(PossibleValue::new(self.0.short_name()))
  }
}

#[derive(Debug, Parser, Clone)]
#[clap(
  about = "Generate bundles and installers for your app (already built by `tauri build`)",
  long_about = "Generate bundles and installers for your app (already built by `tauri build`). This run `build.beforeBundleCommand` before generating the bundles and installers of your app."
)]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Space or comma separated list of bundles to package.
  ///
  /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
  #[clap(short, long, action = ArgAction::Append, num_args(0..), value_delimiter = ',')]
  pub bundles: Option<Vec<BundleFormat>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<ConfigValue>,
  /// Space or comma separated list of features, should be the same features passed to `tauri build` if any.
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// Target triple to build against.
  ///
  /// It must be one of the values outputted by `$rustc --print target-list` or `universal-apple-darwin` for an universal macOS application.
  ///
  /// Note that compiling an universal macOS application requires both `aarch64-apple-darwin` and `x86_64-apple-darwin` targets to be installed.
  #[clap(short, long)]
  pub target: Option<String>,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
}

impl From<crate::build::Options> for Options {
  fn from(value: crate::build::Options) -> Self {
    Self {
      bundles: value.bundles,
      target: value.target,
      features: value.features,
      debug: value.debug,
      ci: value.ci,
      config: value.config,
    }
  }
}

pub fn command(options: Options, verbosity: u8) -> crate::Result<()> {
  crate::helpers::app_paths::resolve();

  let ci = options.ci;

  let target = options
    .target
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(Target::current);

  let config = get_config(target, options.config.as_ref().map(|c| &c.0))?;

  let interface = AppInterface::new(
    config.lock().unwrap().as_ref().unwrap(),
    options.target.clone(),
  )?;

  let tauri_path = tauri_dir();
  std::env::set_current_dir(tauri_path)
    .with_context(|| "failed to change current working directory")?;

  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();

  let app_settings = interface.app_settings();
  let interface_options = options.clone().into();

  let out_dir = app_settings.out_dir(&interface_options)?;

  bundle(
    &options,
    verbosity,
    ci,
    &interface,
    &app_settings,
    config_,
    &out_dir,
  )
}

#[allow(clippy::too_many_arguments)]
pub fn bundle<A: AppSettings>(
  options: &Options,
  verbosity: u8,
  ci: bool,
  interface: &AppInterface,
  app_settings: &std::sync::Arc<A>,
  config: &ConfigMetadata,
  out_dir: &Path,
) -> crate::Result<()> {
  let package_types: Vec<PackageType> = if let Some(bundles) = &options.bundles {
    bundles.iter().map(|bundle| bundle.0).collect::<Vec<_>>()
  } else {
    config
      .bundle
      .targets
      .to_vec()
      .into_iter()
      .map(Into::into)
      .collect()
  };

  if package_types.is_empty() {
    return Ok(());
  }

  // if we have a package to bundle, let's run the `before_bundle_command`.
  if !package_types.is_empty() {
    if let Some(before_bundle) = config.build.before_bundle_command.clone() {
      helpers::run_hook(
        "beforeBundleCommand",
        before_bundle,
        interface,
        options.debug,
      )?;
    }
  }

  let mut settings = app_settings
    .get_bundler_settings(options.clone().into(), config, out_dir, package_types)
    .with_context(|| "failed to build bundler settings")?;

  settings.set_log_level(match verbosity {
    0 => log::Level::Error,
    1 => log::Level::Info,
    _ => log::Level::Trace,
  });

  let bundles = tauri_bundler::bundle_project(&settings)
    .map_err(|e| match e {
      tauri_bundler::Error::BundlerError(e) => e,
      e => anyhow::anyhow!("{e:#}"),
    })
    .with_context(|| "failed to bundle project")?;

  sign_updaters(settings, bundles, ci)?;

  Ok(())
}

fn sign_updaters(
  settings: tauri_bundler::Settings,
  bundles: Vec<tauri_bundler::Bundle>,
  ci: bool,
) -> crate::Result<()> {
  let Some(update_settings) = settings.updater() else {
    // Updater not enabled
    return Ok(());
  };

  let update_enabled_bundles: Vec<&tauri_bundler::Bundle> = bundles
    .iter()
    .filter(|bundle| {
      matches!(
        bundle.package_type,
        PackageType::Updater
          | PackageType::Nsis
          | PackageType::WindowsMsi
          | PackageType::AppImage
          | PackageType::Deb
      )
    })
    .collect();

  if update_enabled_bundles.is_empty() {
    return Ok(());
  }

  // get the public key
  let pubkey = &update_settings.pubkey;
  // check if pubkey points to a file...
  let maybe_path = Path::new(pubkey);
  let pubkey = if maybe_path.exists() {
    std::fs::read_to_string(maybe_path)?
  } else {
    pubkey.to_string()
  };

  // if no password provided we use an empty string
  let password = std::env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD")
    .ok()
    .or_else(|| if ci { Some("".into()) } else { None });

  // get the private key
  let private_key = std::env::var("TAURI_SIGNING_PRIVATE_KEY")
    .map_err(|_| anyhow::anyhow!("A public key has been found, but no private key. Make sure to set `TAURI_SIGNING_PRIVATE_KEY` environment variable."))?;
  // check if private_key points to a file...
  let maybe_path = Path::new(&private_key);
  let private_key = if maybe_path.exists() {
    std::fs::read_to_string(maybe_path)
      .with_context(|| format!("faild to read {}", maybe_path.display()))?
  } else {
    private_key
  };
  let secret_key =
    updater_signature::secret_key(private_key, password).context("failed to decode secret key")?;
  let public_key = updater_signature::pub_key(pubkey).context("failed to decode pubkey")?;

  let mut signed_paths = Vec::new();
  for bundle in update_enabled_bundles {
    // we expect to have only one path in the vec but we iter if we add
    // another type of updater package who require multiple file signature
    for path in &bundle.bundle_paths {
      // sign our path from environment variables
      let (signature_path, signature) = updater_signature::sign_file(&secret_key, path)?;
      if signature.keynum() != public_key.keynum() {
        log::warn!("The updater secret key from `TAURI_SIGNING_PRIVATE_KEY` does not match the public key from `plugins > updater > pubkey`. If you are not rotating keys, this means your configuration is wrong and won't be accepted at runtime when performing update.");
      }
      signed_paths.push(signature_path);
    }
  }

  print_signed_updater_archive(&signed_paths)?;

  Ok(())
}

fn print_signed_updater_archive(output_paths: &[PathBuf]) -> crate::Result<()> {
  use std::fmt::Write;
  if !output_paths.is_empty() {
    let finished_bundles = output_paths.len();
    let pluralised = if finished_bundles == 1 {
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
    log::info!( action = "Finished"; "{finished_bundles} {pluralised} at:\n{printable_paths}");
  }
  Ok(())
}
