use super::{
  delete_codegen_vars, device_prompt, ensure_init, env, init_dot_cargo, open_and_wait,
  verbosity_to_noise_level, with_config, Error, MobileTarget,
};
use crate::{
  helpers::{config::get as get_tauri_config, flock},
  interface::{AppSettings, Interface, MobileOptions, Options as InterfaceOptions},
  mobile::{write_options, CliOptions, DevChild, DevProcess},
  Result,
};
use clap::Parser;

use cargo_mobile::{
  android::config::{Config as AndroidConfig, Metadata as AndroidMetadata},
  config::Config,
  opts::{NoiseLevel, Profile},
};

use std::env::set_var;

const WEBVIEW_CLIENT_CLASS_EXTENSION: &str = "
    @android.annotation.SuppressLint(\"WebViewClientOnReceivedSslError\")
    override fun onReceivedSslError(view: WebView?, handler: SslErrorHandler, error: android.net.http.SslError) {
        handler.proceed()
    }
";
const WEBVIEW_CLASS_INIT: &str =
  "this.settings.mixedContentMode = android.webkit.WebSettings.MIXED_CONTENT_ALWAYS_ALLOW";

#[derive(Debug, Clone, Parser)]
#[clap(about = "Android dev")]
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
  /// Disable the file watcher
  #[clap(long)]
  pub no_watch: bool,
  /// Open Android Studio instead of trying to run on a connected device
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
      release_mode: false,
      args: Vec::new(),
      no_watch: options.no_watch,
    }
  }
}

pub fn command(options: Options, verbosity: usize) -> Result<()> {
  delete_codegen_vars();
  with_config(Some(Default::default()), |root_conf, config, metadata| {
    set_var(
      "WRY_RUSTWEBVIEWCLIENT_CLASS_EXTENSION",
      WEBVIEW_CLIENT_CLASS_EXTENSION,
    );
    set_var("WRY_RUSTWEBVIEW_CLASS_INIT", WEBVIEW_CLASS_INIT);
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    run_dev(
      options,
      root_conf,
      config,
      metadata,
      verbosity_to_noise_level(verbosity),
    )
    .map_err(|e| Error::DevFailed(format!("{:#}", e)))
  })
  .map_err(Into::into)
}

fn run_dev(
  options: Options,
  root_conf: &Config,
  config: &AndroidConfig,
  metadata: &AndroidMetadata,
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
  let _lock = flock::open_rw(&out_dir.join("lock").with_extension("android"), "Android")?;

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
      write_options(cli_options, &bundle_identifier, MobileTarget::Android)?;

      if open {
        open_and_wait(config)
      } else {
        match run(options, root_conf, config, metadata, noise_level) {
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
  config: &AndroidConfig,
  metadata: &AndroidMetadata,
  noise_level: NoiseLevel,
) -> Result<DevChild, Error> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let build_app_bundle = metadata.asset_packs().is_some();

  let env = env()?;
  init_dot_cargo(root_conf, Some(&env)).map_err(Error::InitDotCargo)?;

  device_prompt(&env)
    .map_err(Error::FailedToPromptForDevice)?
    .run(
      config,
      &env,
      noise_level,
      profile,
      None,
      build_app_bundle,
      false,
      ".MainActivity".into(),
    )
    .map(|c| DevChild(Some(c)))
    .map_err(Error::RunFailed)
}
