use super::{
  device_prompt, ensure_init, env, init_dot_cargo, open_and_wait, setup_dev_config, with_config,
  MobileTarget, APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME,
};
use crate::{
  dev::Options as DevOptions,
  helpers::flock,
  interface::{AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  Result,
};
use clap::{ArgAction, Parser};

use dialoguer::{theme::ColorfulTheme, Select};
use tauri_mobile::{
  apple::{config::Config as AppleConfig, device::Device, teams::find_development_teams},
  config::app::App,
  env::Env,
  opts::{NoiseLevel, Profile},
};

use std::env::{set_var, var_os};

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
  /// Disable the dev server for static files.
  #[clap(long)]
  pub no_dev_server: bool,
  /// Open Xcode instead of trying to run on a connected device
  #[clap(short, long)]
  pub open: bool,
  /// Runs on the given device name
  pub device: Option<String>,
}

impl From<Options> for DevOptions {
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
      no_dev_server: options.no_dev_server,
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  if var_os(APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME).is_none() {
    if let Ok(teams) = find_development_teams() {
      let index = match teams.len() {
        0 => None,
        1 => Some(0),
        _ => {
          let index = Select::with_theme(&ColorfulTheme::default())
            .items(
              &teams
                .iter()
                .map(|t| format!("{} (ID: {})", t.name, t.id))
                .collect::<Vec<String>>(),
            )
            .default(0)
            .interact()?;
          Some(index)
        }
      };
      if let Some(index) = index {
        let team = teams.get(index).unwrap();
        log::info!(
            "Using development team `{}`. To make this permanent, set the `{}` environment variable to `{}`",
            team.name,
            APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME,
            team.id
          );
        set_var(APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME, &team.id);
      }
    }
  }
  with_config(
    Some(Default::default()),
    |app, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Ios)?;
      run_dev(options, app, config, noise_level).map_err(Into::into)
    },
  )
}

fn run_dev(
  mut options: Options,
  app: &App,
  config: &AppleConfig,
  noise_level: NoiseLevel,
) -> Result<()> {
  setup_dev_config(&mut options.config)?;
  let env = env()?;
  let device = if options.open {
    None
  } else {
    match device_prompt(&env, options.device.as_deref()) {
      Ok(d) => Some(d),
      Err(e) => {
        log::error!("{e}");
        None
      }
    }
  };

  let mut dev_options: DevOptions = options.clone().into();
  dev_options.target = Some(
    device
      .as_ref()
      .map(|d| d.target().triple.to_string())
      .unwrap_or_else(|| "aarch64-apple-ios".into()),
  );
  let mut interface = crate::dev::setup(&mut dev_options, true)?;

  let app_settings = interface.app_settings();
  let bin_path = app_settings.app_binary_path(&InterfaceOptions {
    debug: !dev_options.release_mode,
    ..Default::default()
  })?;
  let out_dir = bin_path.parent().unwrap();
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("ios"), "iOS")?;

  init_dot_cargo(app, None)?;

  let open = options.open;
  let exit_on_panic = options.exit_on_panic;
  let no_watch = options.no_watch;
  interface.mobile_dev(
    MobileOptions {
      debug: true,
      features: options.features,
      args: Vec::new(),
      config: options.config,
      no_watch: options.no_watch,
    },
    |options| {
      let mut env = env.clone();
      let cli_options = CliOptions {
        features: options.features.clone(),
        args: options.args.clone(),
        noise_level,
        vars: Default::default(),
      };
      let _handle = write_options(cli_options, &mut env)?;

      if open {
        open_and_wait(config, &env)
      } else if let Some(device) = &device {
        match run(device, options, config, &env) {
          Ok(c) => {
            crate::dev::wait_dev_process(c.clone(), move |status, reason| {
              crate::dev::on_app_exit(status, reason, exit_on_panic, no_watch)
            });
            Ok(Box::new(c) as Box<dyn DevProcess>)
          }
          Err(e) => {
            crate::dev::kill_before_dev_process();
            Err(e.into())
          }
        }
      } else {
        open_and_wait(config, &env)
      }
    },
  )
}

#[derive(Debug, thiserror::Error)]
enum RunError {
  #[error("{0}")]
  RunFailed(String),
}
fn run(
  device: &Device<'_>,
  options: MobileOptions,
  config: &AppleConfig,
  env: &Env,
) -> Result<DevChild, RunError> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  device
    .run(
      config,
      env,
      NoiseLevel::FranklyQuitePedantic,
      false, // do not quit on app exit
      profile,
    )
    .map(DevChild::new)
    .map_err(|e| RunError::RunFailed(e.to_string()))
}
