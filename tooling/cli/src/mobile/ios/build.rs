use super::{
  detect_target_ok, ensure_init, env, init_dot_cargo, log_finished, open_and_wait, with_config,
  MobileTarget,
};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, Interface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  Result,
};
use clap::{ArgAction, Parser};

use cargo_mobile::{
  apple::{config::Config as AppleConfig, target::Target},
  env::Env,
  opts::{NoiseLevel, Profile},
  target::{call_for_targets_with_fallback, TargetInvalid, TargetTrait},
};

use std::fs;

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
    action = ArgAction::Append,
    num_args(0..),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Vec<String>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Build number to append to the app version.
  #[clap(long)]
  pub build_number: Option<u32>,
  /// Open Xcode
  #[clap(short, long)]
  pub open: bool,
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

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  with_config(
    Some(Default::default()),
    |app, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Ios)?;

      let env = env()?;
      init_dot_cargo(app, None)?;

      let open = options.open;
      run_build(options, config, &env, noise_level)?;

      if open {
        open_and_wait(config, &env);
      }

      Ok(())
    },
  )
  .map_err(Into::into)
}

fn run_build(
  mut options: Options,
  config: &AppleConfig,
  env: &Env,
  noise_level: NoiseLevel,
) -> Result<()> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

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
    noise_level,
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
    env,
    |target: &Target| {
      let mut app_version = config.bundle_version().clone();
      if let Some(build_number) = options.build_number {
        app_version.push_extra(build_number);
      }

      target.build(config, env, noise_level, profile)?;
      target.archive(config, env, noise_level, profile, Some(app_version))?;
      target.export(config, env, noise_level)?;

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
  .map_err(|e: TargetInvalid| anyhow::anyhow!(e.to_string()))?
  .map_err(|e: anyhow::Error| e)?;

  log_finished(out_files, "IPA");

  Ok(())
}
