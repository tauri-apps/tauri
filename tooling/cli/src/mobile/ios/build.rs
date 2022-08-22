use super::{detect_target_ok, ensure_init, env, init_dot_cargo, with_config, Error, MobileTarget};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, Interface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  Result,
};
use clap::Parser;

use cargo_mobile::{
  apple::{config::Config as AppleConfig, target::Target},
  env::Env,
  opts::{NoiseLevel, Profile},
  target::{call_for_targets_with_fallback, TargetInvalid, TargetTrait},
};

use std::{fmt::Write, fs, path::PathBuf};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Android build")]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build.
  #[clap(
    short,
    long = "target",
    multiple_occurrences(true),
    multiple_values(true),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Vec<String>,
  /// List of cargo features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Build number to append to the app version.
  #[clap(long)]
  pub build_number: Option<u32>,
}

impl From<Options> for crate::build::Options {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      config: options.config,
      args: Vec::new(),
    }
  }
}

pub fn command(options: Options) -> Result<()> {
  with_config(|root_conf, config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Ios)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = env()?;
    init_dot_cargo(root_conf, None).map_err(Error::InitDotCargo)?;

    run_build(options, config, env).map_err(|e| Error::BuildFailed(e.to_string()))
  })
  .map_err(Into::into)
}

fn run_build(mut options: Options, config: &AppleConfig, env: Env) -> Result<()> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };
  let noise_level = NoiseLevel::Polite;

  let bundle_identifier = {
    let tauri_config = get_tauri_config(None)?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    tauri_config_.tauri.bundle.identifier.clone()
  };

  let mut build_options = options.clone().into();
  let interface = crate::build::setup(&mut build_options)?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: build_options.debug,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("ios"), "iOS")?;

  let cli_options = CliOptions {
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    vars: Default::default(),
  };
  write_options(cli_options, &bundle_identifier, MobileTarget::Ios)?;

  options
    .features
    .get_or_insert(Vec::new())
    .push("custom-protocol".into());

  let mut out_files = Vec::new();

  call_for_targets_with_fallback(
    options.targets.iter(),
    &detect_target_ok,
    &env,
    |target: &Target| {
      let mut app_version = config.bundle_version().clone();
      if let Some(build_number) = options.build_number {
        app_version.push_extra(build_number);
      }

      target.build(config, &env, noise_level, profile)?;
      target.archive(config, &env, noise_level, profile, Some(app_version))?;
      target.export(config, &env, noise_level)?;

      if let Ok(ipa_path) = config.ipa_path() {
        let out_dir = config.export_dir().join(target.arch);
        fs::create_dir_all(&out_dir)?;
        let path = out_dir.join(ipa_path.file_name().unwrap());
        fs::rename(&ipa_path, &path)?;
        out_files.push(path);
      }

      anyhow::Result::Ok(())
    },
  )
  .map_err(|e: TargetInvalid| Error::TargetInvalid(e.to_string()))?
  .map_err(|e: anyhow::Error| e)?;

  log_finished(out_files, "IPA");

  Ok(())
}

fn log_finished(outputs: Vec<PathBuf>, kind: &str) {
  if !outputs.is_empty() {
    let mut printable_paths = String::new();
    for path in &outputs {
      writeln!(printable_paths, "        {}", path.display()).unwrap();
    }

    log::info!(action = "Finished"; "{} {}{} at:\n{}", outputs.len(), kind, if outputs.len() == 1 { "" } else { "s" }, printable_paths);
  }
}
