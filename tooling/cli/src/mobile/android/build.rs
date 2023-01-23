use super::{
  delete_codegen_vars, ensure_init, env, init_dot_cargo, log_finished, open_and_wait, with_config,
  MobileTarget,
};
use crate::{
  build::Options as BuildOptions,
  helpers::flock,
  interface::{AppSettings, Interface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  Result,
};
use clap::{ArgAction, Parser};

use tauri_mobile::{
  android::{aab, apk, config::Config as AndroidConfig, env::Env, target::Target},
  opts::{NoiseLevel, Profile},
  target::TargetTrait,
};

use std::env::set_var;

#[derive(Debug, Clone, Parser)]
#[clap(about = "Android build")]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build (all by default).
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Whether to split the APKs and AABs per ABIs.
  #[clap(long)]
  pub split_per_abi: bool,
  /// Build APKs.
  #[clap(long)]
  pub apk: bool,
  /// Build AABs.
  #[clap(long)]
  pub aab: bool,
  /// Open Android Studio
  #[clap(short, long)]
  pub open: bool,
}

impl From<Options> for BuildOptions {
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
  delete_codegen_vars();
  with_config(
    Some(Default::default()),
    |app, config, _metadata, _cli_options| {
      set_var("WRY_RUSTWEBVIEWCLIENT_CLASS_EXTENSION", "");
      set_var("WRY_RUSTWEBVIEW_CLASS_INIT", "");

      ensure_init(config.project_dir(), MobileTarget::Android)?;

      let mut env = env()?;
      init_dot_cargo(app, Some((&env, config)))?;

      let open = options.open;
      run_build(options, config, &mut env, noise_level)?;

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
  config: &AndroidConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
) -> Result<()> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  if !(options.apk || options.aab) {
    // if the user didn't specify the format to build, we'll do both
    options.apk = true;
    options.aab = true;
  }

  let mut build_options: BuildOptions = options.clone().into();
  build_options.target = Some(
    Target::all()
      .get(Target::DEFAULT_KEY)
      .unwrap()
      .triple
      .into(),
  );
  let interface = crate::build::setup(&mut build_options, true)?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: build_options.debug,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("android"), "Android")?;

  let cli_options = CliOptions {
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
  };
  let _handle = write_options(cli_options, &mut env.base)?;

  options
    .features
    .get_or_insert(Vec::new())
    .push("custom-protocol".into());

  let apk_outputs = if options.apk {
    apk::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(Vec::new())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  let aab_outputs = if options.aab {
    aab::build(
      config,
      env,
      noise_level,
      profile,
      get_targets_or_all(Vec::new())?,
      options.split_per_abi,
    )?
  } else {
    Vec::new()
  };

  log_finished(apk_outputs, "APK");
  log_finished(aab_outputs, "AAB");

  Ok(())
}

fn get_targets_or_all<'a>(targets: Vec<String>) -> Result<Vec<&'a Target<'a>>> {
  if targets.is_empty() {
    Ok(Target::all().iter().map(|t| t.1).collect())
  } else {
    let mut outs = Vec::new();

    let possible_targets = Target::all()
      .keys()
      .map(|key| key.to_string())
      .collect::<Vec<String>>()
      .join(",");

    for t in targets {
      let target = Target::for_name(&t).ok_or_else(|| {
        anyhow::anyhow!(
          "Target {} is invalid; the possible targets are {}",
          t,
          possible_targets
        )
      })?;
      outs.push(target);
    }
    Ok(outs)
  }
}
