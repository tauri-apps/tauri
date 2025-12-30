// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile2::{
  apple::{
    config::{
      Config as AppleConfig, Metadata as AppleMetadata, Platform as ApplePlatform,
      Raw as RawAppleConfig,
    },
    device::{self, Device},
    target::Target,
    teams::find_development_teams,
  },
  config::app::{App, DEFAULT_ASSET_DIR},
  env::Env,
  opts::NoiseLevel,
  os,
  util::{prompt, relativize_path},
};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use sublime_fuzzy::best_match;
use tauri_utils::resources::ResourcePaths;

use super::{
  ensure_init, env, get_app, init::command as init_command, log_finished, read_options, CliOptions,
  OptionsHandle, Target as MobileTarget, MIN_DEVICE_MATCH_SCORE,
};
use crate::{
  error::{Context, ErrorExt},
  helpers::{
    app_paths::tauri_dir,
    config::{BundleResources, Config as TauriConfig, ConfigHandle},
    pbxproj, strip_semver_prerelease_tag,
  },
  ConfigValue, Error, Result,
};

use std::{
  env::{set_var, var_os},
  fs::create_dir_all,
  path::Path,
  str::FromStr,
  thread::sleep,
  time::Duration,
};

mod build;
mod dev;
pub(crate) mod project;
mod run;
mod xcode_script;

pub const APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME: &str = "APPLE_DEVELOPMENT_TEAM";
pub const LIB_OUTPUT_FILE_NAME: &str = "libapp.a";

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "iOS commands",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Debug, Parser)]
#[clap(about = "Initialize iOS target in the project")]
pub struct InitOptions {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
  /// Reinstall dependencies
  #[clap(short, long)]
  reinstall_deps: bool,
  /// Skips installing rust toolchains via rustup
  #[clap(long)]
  skip_targets_install: bool,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json and tauri.ios.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Dev(dev::Options),
  Build(build::Options),
  Run(run::Options),
  #[clap(hide(true))]
  XcodeScript(xcode_script::Options),
}

pub fn command(cli: Cli, verbosity: u8) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => {
      crate::helpers::app_paths::resolve();
      init_command(
        MobileTarget::Ios,
        options.ci,
        options.reinstall_deps,
        options.skip_targets_install,
        options.config,
      )?
    }
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level).map(|_| ())?,
    Commands::Run(options) => run::command(options, noise_level)?,
    Commands::XcodeScript(options) => xcode_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: &App,
  tauri_config: &TauriConfig,
  features: &[String],
  cli_options: &CliOptions,
) -> Result<(AppleConfig, AppleMetadata)> {
  let mut ios_options = cli_options.clone();
  ios_options.features.extend_from_slice(features);

  let bundle_version = if let Some(bundle_version) = tauri_config
    .bundle
    .ios
    .bundle_version
    .clone()
    .or_else(|| tauri_config.version.clone())
  {
    // if it's a semver string, we must strip the prerelease tag
    if let Ok(mut version) = semver::Version::from_str(&bundle_version) {
      if !version.pre.is_empty() {
        log::warn!("CFBundleVersion cannot have prerelease tag; stripping from {bundle_version}");
        strip_semver_prerelease_tag(&mut version)?;
      }
      // correctly serialize version - cannot contain `+` as build metadata separator
      Some(format!(
        "{}.{}.{}{}",
        version.major,
        version.minor,
        version.patch,
        if version.build.is_empty() {
          "".to_string()
        } else {
          format!(".{}", version.build.as_str())
        }
      ))
    } else {
      // let it go as is - cargo-mobile2 will validate it
      Some(bundle_version)
    }
  } else {
    None
  };
  let full_bundle_version_short = if let Some(app_version) = &tauri_config.version {
    if let Ok(mut version) = semver::Version::from_str(app_version) {
      if !version.pre.is_empty() {
        log::warn!(
          "CFBundleShortVersionString cannot have prerelease tag; stripping from {app_version}"
        );
        strip_semver_prerelease_tag(&mut version)?;
      }
      // correctly serialize version - cannot contain `+` as build metadata separator
      Some(format!(
        "{}.{}.{}{}",
        version.major,
        version.minor,
        version.patch,
        if version.build.is_empty() {
          "".to_string()
        } else {
          format!(".{}", version.build.as_str())
        }
      ))
    } else {
      // let it go as is - cargo-mobile2 will validate it
      Some(app_version.clone())
    }
  } else {
    bundle_version.clone()
  };
  let bundle_version_short = if let Some(full_version) = full_bundle_version_short.as_deref() {
    let mut s = full_version.split('.');
    let short_version = format!(
      "{}.{}.{}",
      s.next().unwrap_or("0"),
      s.next().unwrap_or("0"),
      s.next().unwrap_or("0")
    );

    if short_version != full_version {
      log::warn!("{full_version:?} is not a valid CFBundleShortVersionString since it must contain exactly three dot separated integers; setting it to {short_version} instead");
    }

    Some(short_version)
  } else {
    None
  };

  let raw = RawAppleConfig {
    development_team: std::env::var(APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME)
        .ok()
        .or_else(|| tauri_config.bundle.ios.development_team.clone())
        .or_else(|| {
          let teams = find_development_teams().unwrap_or_default();
          match teams.len() {
            0 => {
              log::warn!("No code signing certificates found. You must add one and set the certificate development team ID on the `bundle > iOS > developmentTeam` config value or the `{APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME}` environment variable. To list the available certificates, run `tauri info`.");
              None
            }
            1 => None,
            _ => {
              log::warn!("You must set the code signing certificate development team ID on the `bundle > iOS > developmentTeam` config value or the `{APPLE_DEVELOPMENT_TEAM_ENV_VAR_NAME}` environment variable. Available certificates: {}", teams.iter().map(|t| format!("{} (ID: {})", t.name, t.id)).collect::<Vec<String>>().join(", "));
              None
            }
          }
        }),
    ios_features: Some(ios_options.features.clone()),
    bundle_version,
    bundle_version_short,
    ios_version: Some(tauri_config.bundle.ios.minimum_system_version.clone()),
    ..Default::default()
  };
  let config = AppleConfig::from_raw(app.clone(), Some(raw))
    .context("failed to create Apple configuration")?;

  let tauri_dir = tauri_dir();

  let mut vendor_frameworks = Vec::new();
  let mut frameworks = Vec::new();
  for framework in tauri_config
    .bundle
    .ios
    .frameworks
    .clone()
    .unwrap_or_default()
  {
    let framework_path = Path::new(&framework);
    let ext = framework_path.extension().unwrap_or_default();
    if ext.is_empty() {
      frameworks.push(framework);
    } else if ext == "framework" {
      frameworks.push(
        framework_path
          .file_stem()
          .unwrap()
          .to_string_lossy()
          .to_string(),
      );
    } else {
      vendor_frameworks.push(
        relativize_path(tauri_dir.join(framework_path), config.project_dir())
          .to_string_lossy()
          .to_string(),
      );
    }
  }

  let metadata = AppleMetadata {
    supported: true,
    ios: ApplePlatform {
      cargo_args: Some(ios_options.args),
      features: if ios_options.features.is_empty() {
        None
      } else {
        Some(ios_options.features)
      },
      frameworks: Some(frameworks),
      vendor_frameworks: Some(vendor_frameworks),
      ..Default::default()
    },
    macos: Default::default(),
  };

  set_var("TAURI_IOS_PROJECT_PATH", config.project_dir());
  set_var("TAURI_IOS_APP_NAME", config.app().name());

  Ok((config, metadata))
}

fn connected_device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  let device_list = device::list_devices(env).map_err(|cause| {
    Error::GenericError(format!("Failed to detect connected iOS devices: {cause}"))
  })?;
  if !device_list.is_empty() {
    let device = if let Some(t) = target {
      let (device, score) = device_list
        .into_iter()
        .rev()
        .map(|d| {
          let score = best_match(t, d.name()).map_or(0, |m| m.score());
          (d, score)
        })
        .max_by_key(|(_, score)| *score)
        // we already checked the list is not empty
        .unwrap();
      if score > MIN_DEVICE_MATCH_SCORE {
        device
      } else {
        crate::error::bail!("Could not find an iOS device matching {t}")
      }
    } else {
      let index = if device_list.len() > 1 {
        prompt::list(
          concat!("Detected ", "iOS", " devices"),
          device_list.iter(),
          "device",
          None,
          "Device",
        )
        .context("failed to prompt for device")?
      } else {
        0
      };
      device_list.into_iter().nth(index).unwrap()
    };
    println!(
      "Detected connected device: {} with target {:?}",
      device,
      device.target().triple,
    );

    Ok(device)
  } else {
    crate::error::bail!("No connected iOS devices detected")
  }
}

#[derive(Default, Deserialize)]
struct InstalledRuntimesList {
  runtimes: Vec<InstalledRuntime>,
}

#[derive(Deserialize)]
struct InstalledRuntime {
  name: String,
}

fn simulator_prompt(env: &'_ Env, target: Option<&str>) -> Result<device::Simulator> {
  let simulator_list = device::list_simulators(env).map_err(|cause| {
    Error::GenericError(format!(
      "Failed to detect connected iOS Simulator devices: {cause}"
    ))
  })?;
  if !simulator_list.is_empty() {
    let device = if let Some(t) = target {
      let (device, score) = simulator_list
        .into_iter()
        .rev()
        .map(|d| {
          let score = best_match(t, d.name()).map_or(0, |m| m.score());
          (d, score)
        })
        .max_by_key(|(_, score)| *score)
        // we already checked the list is not empty
        .unwrap();
      if score > MIN_DEVICE_MATCH_SCORE {
        device
      } else {
        crate::error::bail!("Could not find an iOS Simulator matching {t}")
      }
    } else if simulator_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "iOS", " simulators"),
        simulator_list.iter(),
        "simulator",
        None,
        "Simulator",
      )
      .context("failed to prompt for simulator")?;
      simulator_list.into_iter().nth(index).unwrap()
    } else {
      simulator_list.into_iter().next().unwrap()
    };
    Ok(device)
  } else {
    log::warn!("No available iOS Simulator detected");
    let install_ios = crate::helpers::prompts::confirm(
      "Would you like to install the latest iOS runtime?",
      Some(false),
    )
    .unwrap_or_default();
    if install_ios {
      duct::cmd("xcodebuild", ["-downloadPlatform", "iOS"])
        .stdout_file(os_pipe::dup_stdout().unwrap())
        .stderr_file(os_pipe::dup_stderr().unwrap())
        .run()
        .map_err(|e| Error::CommandFailed {
          command: "xcodebuild -downloadPlatform iOS".to_string(),
          error: e,
        })?;
      return simulator_prompt(env, target);
    }
    crate::error::bail!("No available iOS Simulator detected")
  }
}

fn device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  if let Ok(device) = connected_device_prompt(env, target) {
    Ok(device)
  } else {
    let simulator = simulator_prompt(env, target)?;
    log::info!("Starting simulator {}", simulator.name());
    simulator
      .start_detached(env)
      .context("failed to start simulator")?;
    Ok(simulator.into())
  }
}

fn ensure_ios_runtime_installed() -> Result<()> {
  let installed_platforms_json = duct::cmd("xcrun", ["simctl", "list", "runtimes", "--json"])
    .read()
    .map_err(|e| Error::CommandFailed {
      command: "xcrun simctl list runtimes --json".to_string(),
      error: e,
    })?;
  let installed_platforms: InstalledRuntimesList =
    serde_json::from_str(&installed_platforms_json).unwrap_or_default();
  if !installed_platforms
    .runtimes
    .iter()
    .any(|r| r.name.starts_with("iOS"))
  {
    log::warn!("iOS platform not installed");
    let install_ios = crate::helpers::prompts::confirm(
      "Would you like to install the latest iOS runtime?",
      Some(false),
    )
    .unwrap_or_default();
    if install_ios {
      duct::cmd("xcodebuild", ["-downloadPlatform", "iOS"])
        .stdout_file(os_pipe::dup_stdout().unwrap())
        .stderr_file(os_pipe::dup_stderr().unwrap())
        .run()
        .map_err(|e| Error::CommandFailed {
          command: "xcodebuild -downloadPlatform iOS".to_string(),
          error: e,
        })?;
    } else {
      crate::error::bail!("iOS platform not installed");
    }
  }
  Ok(())
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env, None).map(|device| device.target()).ok()
}

fn open_and_wait(config: &AppleConfig, env: &Env) -> ! {
  log::info!("Opening Xcode");
  if let Err(e) = os::open_file_with("Xcode", config.project_dir(), env) {
    log::error!("{e}");
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}

fn inject_resources(config: &AppleConfig, tauri_config: &TauriConfig) -> Result<()> {
  let asset_dir = config.project_dir().join(DEFAULT_ASSET_DIR);
  create_dir_all(&asset_dir).fs_context("failed to create asset directory", asset_dir.clone())?;

  let resources = match &tauri_config.bundle.resources {
    Some(BundleResources::List(paths)) => Some(ResourcePaths::new(paths.as_slice(), true)),
    Some(BundleResources::Map(map)) => Some(ResourcePaths::from_map(map, true)),
    None => None,
  };
  if let Some(resources) = resources {
    for resource in resources.iter() {
      let resource = resource.context("failed to get resource")?;
      let dest = asset_dir.join(resource.target());
      crate::helpers::fs::copy_file(resource.path(), dest)?;
    }
  }

  Ok(())
}

pub fn signing_from_env() -> Result<(
  Option<tauri_macos_sign::Keychain>,
  Option<tauri_macos_sign::ProvisioningProfile>,
)> {
  let keychain = match (
    var_os("IOS_CERTIFICATE"),
    var_os("IOS_CERTIFICATE_PASSWORD"),
  ) {
    (Some(certificate), Some(certificate_password)) => {
      log::info!("Reading iOS certificates from ");
      tauri_macos_sign::Keychain::with_certificate(&certificate, &certificate_password)
        .map(Some)
        .map_err(Box::new)?
    }
    (Some(_), None) => {
      log::warn!("The IOS_CERTIFICATE environment variable is set but not IOS_CERTIFICATE_PASSWORD. Ignoring the certificate...");
      None
    }
    _ => None,
  };

  let provisioning_profile = if let Some(provisioning_profile) = var_os("IOS_MOBILE_PROVISION") {
    tauri_macos_sign::ProvisioningProfile::from_base64(&provisioning_profile)
      .map(Some)
      .map_err(Box::new)?
  } else {
    if keychain.is_some() {
      log::warn!("You have provided an iOS certificate via environment variables but the IOS_MOBILE_PROVISION environment variable is not set. This will fail when signing unless the profile is set in your Xcode project.");
    }
    None
  };

  Ok((keychain, provisioning_profile))
}

pub struct ProjectConfig {
  pub code_sign_identity: Option<String>,
  pub team_id: Option<String>,
  pub provisioning_profile_uuid: Option<String>,
}

pub fn project_config(
  keychain: Option<&tauri_macos_sign::Keychain>,
  provisioning_profile: Option<&tauri_macos_sign::ProvisioningProfile>,
) -> Result<ProjectConfig> {
  Ok(ProjectConfig {
    code_sign_identity: keychain.map(|k| k.signing_identity()),
    team_id: keychain.and_then(|k| k.team_id().map(ToString::to_string)),
    provisioning_profile_uuid: provisioning_profile.and_then(|p| p.uuid().ok()),
  })
}

pub fn load_pbxproj(config: &AppleConfig) -> Result<pbxproj::Pbxproj> {
  pbxproj::parse(
    config
      .project_dir()
      .join(format!("{}.xcodeproj", config.app().name()))
      .join("project.pbxproj"),
  )
}

pub fn synchronize_project_config(
  config: &AppleConfig,
  tauri_config: &ConfigHandle,
  pbxproj: &mut pbxproj::Pbxproj,
  export_options_plist: &mut plist::Dictionary,
  project_config: &ProjectConfig,
  debug: bool,
) -> Result<()> {
  let identifier = tauri_config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .identifier
    .clone();
  let product_name = tauri_config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .product_name
    .clone();

  let manual_signing = project_config.code_sign_identity.is_some()
    || project_config.provisioning_profile_uuid.is_some();

  if let Some(xc_configuration_list) = pbxproj
    .xc_configuration_list
    .clone()
    .into_values()
    .find(|l| l.comment.contains("_iOS"))
  {
    for build_configuration_ref in xc_configuration_list.build_configurations {
      if manual_signing {
        pbxproj.set_build_settings(&build_configuration_ref.id, "CODE_SIGN_STYLE", "Manual");
      }

      if let Some(team) = config.development_team() {
        let team = format!("\"{team}\"");
        pbxproj.set_build_settings(&build_configuration_ref.id, "DEVELOPMENT_TEAM", &team);
      }

      pbxproj.set_build_settings(
        &build_configuration_ref.id,
        "PRODUCT_BUNDLE_IDENTIFIER",
        &identifier,
      );

      if let Some(product_name) = &product_name {
        pbxproj.set_build_settings(
          &build_configuration_ref.id,
          "PRODUCT_NAME",
          &format!("\"{product_name}\""),
        );
      }

      if let Some(identity) = &project_config.code_sign_identity {
        let identity = format!("\"{identity}\"");
        pbxproj.set_build_settings(&build_configuration_ref.id, "CODE_SIGN_IDENTITY", &identity);
        pbxproj.set_build_settings(
          &build_configuration_ref.id,
          "\"CODE_SIGN_IDENTITY[sdk=iphoneos*]\"",
          &identity,
        );
      }

      if let Some(id) = &project_config.team_id {
        let id = format!("\"{id}\"");
        pbxproj.set_build_settings(&build_configuration_ref.id, "DEVELOPMENT_TEAM", &id);
        pbxproj.set_build_settings(
          &build_configuration_ref.id,
          "\"DEVELOPMENT_TEAM[sdk=iphoneos*]\"",
          &id,
        );
      }

      if let Some(profile_uuid) = &project_config.provisioning_profile_uuid {
        let profile_uuid = format!("\"{profile_uuid}\"");
        pbxproj.set_build_settings(
          &build_configuration_ref.id,
          "PROVISIONING_PROFILE_SPECIFIER",
          &profile_uuid,
        );
        pbxproj.set_build_settings(
          &build_configuration_ref.id,
          "\"PROVISIONING_PROFILE_SPECIFIER[sdk=iphoneos*]\"",
          &profile_uuid,
        );
      }
    }
  }

  let build_configuration = {
    if let Some(xc_configuration_list) = pbxproj
      .xc_configuration_list
      .clone()
      .into_values()
      .find(|l| l.comment.contains("_iOS"))
    {
      let mut configuration = None;
      let target = if debug { "debug" } else { "release" };
      for build_configuration_ref in xc_configuration_list.build_configurations {
        if build_configuration_ref.comments.contains(target) {
          configuration = pbxproj
            .xc_build_configuration
            .get(&build_configuration_ref.id);
          break;
        }
      }

      configuration
    } else {
      None
    }
  };

  if let Some(build_configuration) = build_configuration {
    if let Some(style) = build_configuration.get_build_setting("CODE_SIGN_STYLE") {
      export_options_plist.insert(
        "signingStyle".to_string(),
        style.value.to_lowercase().into(),
      );
    } else {
      export_options_plist.insert("signingStyle".to_string(), "automatic".into());
    }

    if manual_signing {
      if let Some(identity) = build_configuration
        .get_build_setting("\"CODE_SIGN_IDENTITY[sdk=iphoneos*]\"")
        .or_else(|| build_configuration.get_build_setting("CODE_SIGN_IDENTITY"))
      {
        export_options_plist.insert(
          "signingCertificate".to_string(),
          identity.value.trim_matches('"').into(),
        );
      }

      let profile_uuid = project_config
        .provisioning_profile_uuid
        .clone()
        .or_else(|| {
          build_configuration
            .get_build_setting("\"PROVISIONING_PROFILE_SPECIFIER[sdk=iphoneos*]\"")
            .or_else(|| build_configuration.get_build_setting("PROVISIONING_PROFILE_SPECIFIER"))
            .map(|setting| setting.value.trim_matches('"').to_string())
        });
      if let Some(profile_uuid) = profile_uuid {
        let mut provisioning_profiles = plist::Dictionary::new();
        provisioning_profiles.insert(config.app().identifier().to_string(), profile_uuid.into());
        export_options_plist.insert(
          "provisioningProfiles".to_string(),
          provisioning_profiles.into(),
        );
      }
    }

    if let Some(id) = build_configuration
      .get_build_setting("\"DEVELOPMENT_TEAM[sdk=iphoneos*]\"")
      .or_else(|| build_configuration.get_build_setting("DEVELOPMENT_TEAM"))
    {
      export_options_plist.insert("teamID".to_string(), id.value.trim_matches('"').into());
    }
  }

  Ok(())
}
