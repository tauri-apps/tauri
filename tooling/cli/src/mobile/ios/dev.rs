use super::{
  device_prompt, ensure_init, env, init_dot_cargo, open_and_wait, with_config, MobileTarget,
};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  Result,
};
use clap::{ArgAction, Parser};

use cargo_mobile::{
  apple::config::Config as AppleConfig,
  config::app::App,
  env::Env,
  opts::{NoiseLevel, Profile},
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "iOS dev")]
pub struct Options {
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
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
  /// Runs on the given device name
  pub device: Option<String>,
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
  with_config(
    Some(Default::default()),
    |app, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Ios)?;
      run_dev(options, app, config, noise_level).map_err(Into::into)
    },
  )
}

fn run_dev(
  options: Options,
  app: &App,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<()> {
  let mut dev_options = options.clone().into();
  let mut interface = crate::dev::setup(&mut dev_options)?;

  let bundle_identifier = {
    let tauri_config = get_tauri_config(None)?;
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

  let env = env()?;
  init_dot_cargo(app, None)?;

  let open = options.open;
  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
  let device = options.device;
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
        noise_level,
        vars: Default::default(),
      };
      write_options(cli_options, &bundle_identifier, MobileTarget::Ios)?;

      if open {
        open_and_wait(config, &env)
      } else {
        match run(device.as_deref(), options, config, &env, noise_level) {
          Ok(c) => {
            crate::dev::wait_dev_process(c.clone(), move |status, reason| {
              crate::dev::on_app_exit(status, reason, exit_on_panic, no_watch)
            });
            Ok(Box::new(c) as Box<dyn DevProcess>)
          }
          Err(RunError::FailedToPromptForDevice(e)) => {
            log::error!("{}", e);
            open_and_wait(config, &env)
          }
          Err(e) => Err(e.into()),
        }
      }
    },
  )
}

#[derive(Debug, thiserror::Error)]
enum RunError {
  #[error("{0}")]
  FailedToPromptForDevice(String),
  #[error("{0}")]
  RunFailed(String),
}
fn run(
  device: Option<&str>,
  options: MobileOptions,
  config: &AppleConfig,
  env: &Env,
  noise_level: NoiseLevel,
) -> Result<DevChild, RunError> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let non_interactive = true; // ios-deploy --noninteractive (quit when app crashes or exits)

  device_prompt(env, device)
    .map_err(|e| RunError::FailedToPromptForDevice(e.to_string()))?
    .run(config, env, noise_level, non_interactive, profile)
    .map(DevChild::new)
    .map_err(|e| RunError::RunFailed(e.to_string()))
}
