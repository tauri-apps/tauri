use super::{
  device_prompt, ensure_init, env, init_dot_cargo, open_and_wait, with_config, Error, MobileTarget,
};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  Result,
};
use clap::Parser;

use cargo_mobile::{
  apple::config::Config as AppleConfig,
  config::Config,
  opts::{NoiseLevel, Profile},
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "iOS dev")]
pub struct Options {
  /// List of cargo features to activate
  #[clap(short, long, multiple_occurrences(true), multiple_values(true))]
  pub features: Option<Vec<String>>,
  /// Exit on panic
  #[clap(short, long)]
  exit_on_panic: bool,
  /// JSON string or path to JSON file to merge with tauri.conf.json
  #[clap(short, long)]
  pub config: Option<String>,
  /// Run the code in release mode
  #[clap(long = "release")]
  pub release_mode: bool,
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
  /// Open Xcode instead of trying to run on a connected device
  #[clap(short, long)]
  pub open: bool,
}

impl From<Options> for crate::dev::Options {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      target: None,
      features: options.features,
      exit_on_panic: options.exit_on_panic,
      config: options.config,
      release_mode: options.release_mode,
      args: Vec::new(),
      no_watch: options.no_watch,
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  with_config(Some(Default::default()), |root_conf, config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Ios)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    run_dev(options, root_conf, config, noise_level)
      .map_err(|e| Error::DevFailed(format!("{:#}", e)))
  })
  .map_err(Into::into)
}

fn run_dev(
  options: Options,
  root_conf: &Config,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<()> {
  let mut dev_options = options.clone().into();
  let mut interface = crate::dev::setup(&mut dev_options)?;

  let bundle_identifier = {
    let tauri_config =
      get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    tauri_config_.tauri.bundle.identifier.clone()
  };

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: !dev_options.release_mode,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("ios"), "iOS")?;

  let open = options.open;
  interface.mobile_dev(
    MobileOptions {
      debug: true,
      features: options.features,
      args: Vec::new(),
      config: options.config,
      no_watch: options.no_watch,
    },
    |options| {
      let cli_options = CliOptions {
        features: options.features.clone(),
        args: options.args.clone(),
        vars: Default::default(),
      };
      write_options(cli_options, &bundle_identifier, MobileTarget::Ios)?;
      if open {
        open_and_wait(config)
      } else {
        match run(options, root_conf, config, noise_level) {
          Ok(c) => Ok(Box::new(c) as Box<dyn DevProcess>),
          Err(Error::FailedToPromptForDevice(e)) => {
            log::error!("{}", e);
            open_and_wait(config)
          }
          Err(e) => Err(e.into()),
        }
      }
    },
  )
}

fn run(
  options: MobileOptions,
  root_conf: &Config,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<DevChild, Error> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let env = env()?;
  init_dot_cargo(root_conf, None).map_err(Error::InitDotCargo)?;

  device_prompt(&env)
    .map_err(Error::FailedToPromptForDevice)?
    .run(config, &env, noise_level, false, profile)
    .map(|c| DevChild(Some(c)))
    .map_err(Error::RunFailed)
}
