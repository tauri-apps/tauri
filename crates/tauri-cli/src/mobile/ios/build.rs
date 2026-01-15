// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  detect_target_ok, ensure_init, env, get_app, get_config, inject_resources, load_pbxproj,
  log_finished, open_and_wait, project_config, synchronize_project_config, MobileTarget,
  OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  error::{Context, ErrorExt},
  helpers::{
    app_paths::tauri_dir,
    config::{get as get_tauri_config, ConfigHandle},
    flock,
    plist::merge_plist,
  },
  interface::{AppInterface, Interface, Options as InterfaceOptions},
  mobile::{ios::ensure_ios_runtime_installed, write_options, CliOptions, TargetDevice},
  ConfigValue, Error, Result,
};
use clap::{ArgAction, Parser, ValueEnum};

use cargo_mobile2::{
  apple::{
    config::Config as AppleConfig,
    target::{ArchiveConfig, BuildConfig, ExportConfig, Target},
  },
  env::Env,
  opts::{NoiseLevel, Profile},
  target::{call_for_targets_with_fallback, TargetInvalid, TargetTrait},
};
use rand::distr::{Alphanumeric, SampleString};

use std::{
  env::{set_current_dir, var, var_os},
  fs,
  path::PathBuf,
};

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for iOS and generate IPAs",
  long_about = "Build your app in release mode for iOS and generate IPAs. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
)]
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
  pub targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Vec<String>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Build number to append to the app version.
  #[clap(long)]
  pub build_number: Option<u32>,
  /// Open Xcode
  #[clap(short, long)]
  pub open: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Describes how Xcode should export the archive.
  ///
  /// Use this to create a package ready for the App Store (app-store-connect option) or TestFlight (release-testing option).
  #[clap(long, value_enum)]
  pub export_method: Option<ExportMethod>,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri ios build -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
  /// Target device of this build
  #[clap(skip)]
  pub target_device: Option<TargetDevice>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportMethod {
  AppStoreConnect,
  ReleaseTesting,
  Debugging,
}

impl ExportMethod {
  /// Xcode 15.4 deprecated these names (in this case we should use the Display impl).
  pub fn pre_xcode_15_4_name(&self) -> String {
    match self {
      Self::AppStoreConnect => "app-store".to_string(),
      Self::ReleaseTesting => "ad-hoc".to_string(),
      Self::Debugging => "development".to_string(),
    }
  }
}

impl std::fmt::Display for ExportMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::AppStoreConnect => write!(f, "app-store-connect"),
      Self::ReleaseTesting => write!(f, "release-testing"),
      Self::Debugging => write!(f, "debugging"),
    }
  }
}

impl std::str::FromStr for ExportMethod {
  type Err = &'static str;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "app-store-connect" => Ok(Self::AppStoreConnect),
      "release-testing" => Ok(Self::ReleaseTesting),
      "debugging" => Ok(Self::Debugging),
      _ => Err("unknown ios target"),
    }
  }
}

impl From<Options> for BuildOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features,
      bundles: None,
      no_bundle: false,
      config: options.config,
      args: options.args,
      ci: options.ci,
      skip_stapling: false,
      ignore_version_mismatches: options.ignore_version_mismatches,
      no_sign: false,
    }
  }
}

pub struct BuiltApplication {
  pub config: AppleConfig,
  pub interface: AppInterface,
  // prevent drop
  #[allow(dead_code)]
  options_handle: OptionsHandle,
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<BuiltApplication> {
  crate::helpers::app_paths::resolve();

  let mut build_options: BuildOptions = options.clone().into();
  build_options.target = Some(
    Target::all()
      .get(
        options
          .targets
          .as_ref()
          .and_then(|t| t.first())
          .map(|t| t.as_str())
          .unwrap_or(Target::DEFAULT_KEY),
      )
      .unwrap()
      .triple
      .into(),
  );

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::Ios,
    &options.config.iter().map(|c| &c.0).collect::<Vec<_>>(),
  )?;
  let (interface, mut config) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();

    let interface = AppInterface::new(tauri_config_, build_options.target.clone())?;
    interface.build_options(&mut Vec::new(), &mut build_options.features, true);

    let app = get_app(MobileTarget::Ios, tauri_config_, &interface);
    let (config, _metadata) = get_config(
      &app,
      tauri_config_,
      &build_options.features,
      &Default::default(),
    )?;
    (interface, config)
  };

  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).context("failed to set current directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::Ios,
    options.ci,
  )?;
  inject_resources(&config, tauri_config.lock().unwrap().as_ref().unwrap())?;

  let mut plist = plist::Dictionary::new();
  plist.insert(
    "CFBundleShortVersionString".into(),
    config.bundle_version_short().into(),
  );

  let info_plist_path = config
    .project_dir()
    .join(config.scheme())
    .join("Info.plist");
  let mut src_plists = vec![info_plist_path.clone().into()];
  src_plists.push(plist::Value::Dictionary(plist).into());
  if tauri_path.join("Info.plist").exists() {
    src_plists.push(tauri_path.join("Info.plist").into());
  }
  if tauri_path.join("Info.ios.plist").exists() {
    src_plists.push(tauri_path.join("Info.ios.plist").into());
  }
  if let Some(info_plist) = &tauri_config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .bundle
    .ios
    .info_plist
  {
    src_plists.push(info_plist.clone().into());
  }
  let merged_info_plist = merge_plist(src_plists)?;
  merged_info_plist
    .to_file_xml(&info_plist_path)
    .map_err(std::io::Error::other)
    .fs_context("failed to save merged Info.plist file", info_plist_path)?;

  let mut env = env().context("failed to load iOS environment")?;

  if !options.open {
    ensure_ios_runtime_installed()?;
  }

  let mut export_options_plist = plist::Dictionary::new();
  if let Some(method) = options.export_method {
    let xcode_version =
      crate::info::env_system::xcode_version().context("failed to determine Xcode version")?;
    let mut iter = xcode_version.split('.');
    let major = iter.next().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver"
    ))?;
    let minor = iter.next().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver"
    ))?;
    let major = major.parse::<u64>().ok().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver: major is not a number"
    ))?;
    let minor = minor.parse::<u64>().ok().context(format!(
      "failed to parse Xcode version `{xcode_version}` as semver: minor is not a number"
    ))?;

    if major < 15 || (major == 15 && minor < 4) {
      export_options_plist.insert("method".to_string(), method.pre_xcode_15_4_name().into());
    } else {
      export_options_plist.insert("method".to_string(), method.to_string().into());
    }
  }

  let (keychain, provisioning_profile) = super::signing_from_env()?;
  let project_config = project_config(keychain.as_ref(), provisioning_profile.as_ref())?;
  let mut pbxproj = load_pbxproj(&config)?;

  // synchronize pbxproj and exportoptions
  synchronize_project_config(
    &config,
    &tauri_config,
    &mut pbxproj,
    &mut export_options_plist,
    &project_config,
    options.debug,
  )?;
  if pbxproj.has_changes() {
    pbxproj
      .save()
      .fs_context("failed to save pbxproj file", pbxproj.path)?;
  }

  // merge export options and write to temp file
  let _export_options_tmp = if !export_options_plist.is_empty() {
    let export_options_plist_path = config.project_dir().join("ExportOptions.plist");
    let export_options =
      tempfile::NamedTempFile::new().context("failed to create temporary file")?;

    let merged_plist = merge_plist(vec![
      export_options_plist_path.into(),
      plist::Value::from(export_options_plist).into(),
    ])?;
    merged_plist
      .to_file_xml(export_options.path())
      .map_err(std::io::Error::other)
      .fs_context(
        "failed to save export options plist file",
        export_options.path().to_path_buf(),
      )?;

    config.set_export_options_plist_path(export_options.path());

    Some(export_options)
  } else {
    None
  };

  let open = options.open;
  let options_handle = run_build(
    &interface,
    options,
    build_options,
    tauri_config,
    &mut config,
    &mut env,
    noise_level,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(BuiltApplication {
    config,
    interface,
    options_handle,
  })
}

#[allow(clippy::too_many_arguments)]
fn run_build(
  interface: &AppInterface,
  options: Options,
  mut build_options: BuildOptions,
  tauri_config: ConfigHandle,
  config: &mut AppleConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
) -> Result<OptionsHandle> {
  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  crate::build::setup(
    interface,
    &mut build_options,
    tauri_config.lock().unwrap().as_ref().unwrap(),
    true,
  )?;

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(&InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    args: build_options.args.clone(),
    ..Default::default()
  })?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ios"), "iOS")?;

  let cli_options = CliOptions {
    dev: false,
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars: Default::default(),
    config: build_options.config.clone(),
    target_device: options.target_device.clone(),
  };
  let handle = write_options(tauri_config.lock().unwrap().as_ref().unwrap(), cli_options)?;

  if options.open {
    return Ok(handle);
  }

  let mut out_files = Vec::new();

  let force_skip_target_fallback = options.targets.as_ref().is_some_and(|t| t.is_empty());

  call_for_targets_with_fallback(
    options.targets.unwrap_or_default().iter(),
    if force_skip_target_fallback {
      &|_| None
    } else {
      &detect_target_ok
    },
    env,
    |target: &Target| -> Result<()> {
      let mut app_version = config.bundle_version().to_string();
      if let Some(build_number) = options.build_number {
        app_version.push('.');
        app_version.push_str(&build_number.to_string());
      }

      let credentials = auth_credentials_from_env()?;
      let skip_signing = credentials.is_some();

      let mut build_config = BuildConfig::new().allow_provisioning_updates();
      if let Some(credentials) = &credentials {
        build_config = build_config
          .authentication_credentials(credentials.clone())
          .skip_codesign();
      }

      target
        .build(None, config, env, noise_level, profile, build_config)
        .context("failed to build iOS app")?;

      let mut archive_config = ArchiveConfig::new();
      if skip_signing {
        archive_config = archive_config.skip_codesign();
      }

      target
        .archive(
          config,
          env,
          noise_level,
          profile,
          Some(app_version),
          archive_config,
        )
        .context("failed to archive iOS app")?;

      let out_dir = config.export_dir().join(target.arch);

      if target.sdk == "iphonesimulator" {
        fs::create_dir_all(&out_dir)
          .fs_context("failed to create Xcode output directory", out_dir.clone())?;

        let app_path = config
          .archive_dir()
          .join(format!("{}.xcarchive", config.scheme()))
          .join("Products")
          .join("Applications")
          .join(config.app().stylized_name())
          .with_extension("app");

        let path = out_dir.join(app_path.file_name().unwrap());
        fs::rename(&app_path, &path).fs_context("failed to rename app", app_path)?;
        out_files.push(path);
      } else {
        // if we skipped code signing, we do not have the entitlements applied to our exported IPA
        // we must force sign the app binary with a dummy certificate just to preserve the entitlements
        // target.export() will sign it with an actual certificate for us
        if skip_signing {
          let password = Alphanumeric.sample_string(&mut rand::rng(), 16);
          let certificate = tauri_macos_sign::certificate::generate_self_signed(
            tauri_macos_sign::certificate::SelfSignedCertificateRequest {
              algorithm: "rsa".to_string(),
              profile: tauri_macos_sign::certificate::CertificateProfile::AppleDistribution,
              team_id: "unset".to_string(),
              person_name: "Tauri".to_string(),
              country_name: "NL".to_string(),
              validity_days: 365,
              password: password.clone(),
            },
          )
          .map_err(Box::new)?;
          let tmp_dir = tempfile::tempdir().context("failed to create temporary directory")?;
          let cert_path = tmp_dir.path().join("cert.p12");
          std::fs::write(&cert_path, certificate)
            .fs_context("failed to write certificate", cert_path.clone())?;
          let self_signed_cert_keychain =
            tauri_macos_sign::Keychain::with_certificate_file(&cert_path, &password.into())
              .map_err(Box::new)?;

          let app_dir = config
            .export_dir()
            .join(format!("{}.xcarchive", config.scheme()))
            .join("Products/Applications")
            .join(format!("{}.app", config.app().stylized_name()));

          self_signed_cert_keychain
            .sign(
              &app_dir.join(config.app().stylized_name()),
              Some(
                &config
                  .project_dir()
                  .join(config.scheme())
                  .join(format!("{}.entitlements", config.scheme())),
              ),
              false,
            )
            .map_err(Box::new)?;
        }

        let mut export_config = ExportConfig::new().allow_provisioning_updates();
        if let Some(credentials) = &credentials {
          export_config = export_config.authentication_credentials(credentials.clone());
        }

        target
          .export(config, env, noise_level, export_config)
          .context("failed to export iOS app")?;

        if let Ok(ipa_path) = config.ipa_path() {
          fs::create_dir_all(&out_dir)
            .fs_context("failed to create Xcode output directory", out_dir.clone())?;
          let path = out_dir.join(ipa_path.file_name().unwrap());
          fs::rename(&ipa_path, &path).fs_context("failed to rename IPA", ipa_path)?;
          out_files.push(path);
        }
      }

      Ok(())
    },
  )
  .map_err(|e: TargetInvalid| Error::GenericError(e.to_string()))??;

  if !out_files.is_empty() {
    log_finished(out_files, "iOS Bundle");
  }

  Ok(handle)
}

fn auth_credentials_from_env() -> Result<Option<cargo_mobile2::apple::AuthCredentials>> {
  match (
    var("APPLE_API_KEY"),
    var("APPLE_API_ISSUER"),
    var_os("APPLE_API_KEY_PATH").map(PathBuf::from),
  ) {
    (Ok(key_id), Ok(key_issuer_id), Some(key_path)) => {
      Ok(Some(cargo_mobile2::apple::AuthCredentials {
        key_path,
        key_id,
        key_issuer_id,
      }))
    }
    (Err(_), Err(_), None) => Ok(None),
    _ => crate::error::bail!(
      "APPLE_API_KEY, APPLE_API_ISSUER and APPLE_API_KEY_PATH must be provided for code signing"
    ),
  }
}
