// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile2::{
  android::{
    adb,
    config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
    device::Device,
    emulator,
    env::Env,
    target::Target,
  },
  config::app::{App, DEFAULT_ASSET_DIR},
  opts::{FilterLevel, NoiseLevel},
  os,
  target::TargetTrait,
  util::prompt,
};
use clap::{Parser, Subcommand};
use semver::Version;
use std::{
  env::set_var,
  fs::{create_dir, create_dir_all, read_dir, write},
  io::Cursor,
  path::{Path, PathBuf},
  process::{exit, Command},
  thread::sleep,
  time::Duration,
};
use sublime_fuzzy::best_match;
use tauri_utils::resources::ResourcePaths;

use super::{
  ensure_init, get_app, init::command as init_command, log_finished, read_options, CliOptions,
  OptionsHandle, Target as MobileTarget, MIN_DEVICE_MATCH_SCORE,
};
use crate::{
  error::Context,
  helpers::config::{BundleResources, Config as TauriConfig},
  ConfigValue, Error, ErrorExt, Result,
};

mod android_studio_script;
mod build;
mod dev;
pub(crate) mod project;
mod run;

const NDK_VERSION: &str = "29.0.13846066";
const SDK_VERSION: u8 = 36;

#[cfg(target_os = "macos")]
const CMDLINE_TOOLS_URL: &str =
  "https://dl.google.com/android/repository/commandlinetools-mac-13114758_latest.zip";
#[cfg(target_os = "linux")]
const CMDLINE_TOOLS_URL: &str =
  "https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip";
#[cfg(windows)]
const CMDLINE_TOOLS_URL: &str =
  "https://dl.google.com/android/repository/commandlinetools-win-13114758_latest.zip";

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Android commands",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Debug, Parser)]
#[clap(about = "Initialize Android target in the project")]
pub struct InitOptions {
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
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
  AndroidStudioScript(android_studio_script::Options),
}

pub fn command(cli: Cli, verbosity: u8) -> Result<()> {
  let noise_level = NoiseLevel::from_occurrences(verbosity as u64);
  match cli.command {
    Commands::Init(options) => {
      crate::helpers::app_paths::resolve();
      init_command(
        MobileTarget::Android,
        options.ci,
        false,
        options.skip_targets_install,
        options.config,
      )?
    }
    Commands::Dev(options) => dev::command(options, noise_level)?,
    Commands::Build(options) => build::command(options, noise_level).map(|_| ())?,
    Commands::Run(options) => run::command(options, noise_level)?,
    Commands::AndroidStudioScript(options) => android_studio_script::command(options)?,
  }

  Ok(())
}

pub fn get_config(
  app: &App,
  config: &TauriConfig,
  features: &[String],
  cli_options: &CliOptions,
) -> (AndroidConfig, AndroidMetadata) {
  let mut android_options = cli_options.clone();
  android_options.features.extend_from_slice(features);

  let raw = RawAndroidConfig {
    features: Some(android_options.features.clone()),
    logcat_filter_specs: vec![
      "RustStdoutStderr".into(),
      format!(
        "*:{}",
        match cli_options.noise_level {
          NoiseLevel::Polite => FilterLevel::Info,
          NoiseLevel::LoudAndProud => FilterLevel::Debug,
          NoiseLevel::FranklyQuitePedantic => FilterLevel::Verbose,
        }
        .logcat()
      ),
    ],
    min_sdk_version: Some(config.bundle.android.min_sdk_version),
    ..Default::default()
  };
  let config = AndroidConfig::from_raw(app.clone(), Some(raw)).unwrap();

  let metadata = AndroidMetadata {
    supported: true,
    cargo_args: Some(android_options.args),
    features: Some(android_options.features),
    ..Default::default()
  };

  set_var(
    "WRY_ANDROID_PACKAGE",
    app.android_identifier_escape_kotlin_keyword(),
  );
  set_var("TAURI_ANDROID_PACKAGE_UNESCAPED", app.identifier());
  set_var("WRY_ANDROID_LIBRARY", app.lib_name());
  set_var("TAURI_ANDROID_PROJECT_PATH", config.project_dir());

  let src_main_dir = config
    .project_dir()
    .join("app/src/main")
    .join(format!("java/{}", app.identifier().replace('.', "/"),));
  if config.project_dir().exists() {
    if src_main_dir.exists() {
      let _ = create_dir(src_main_dir.join("generated"));
    } else {
      log::error!(
      "Project directory {} does not exist. Did you update the package name in `Cargo.toml` or the bundle identifier in `tauri.conf.json > identifier`? Save your changes, delete the `gen/android` folder and run `tauri android init` to recreate the Android project.",
      src_main_dir.display()
    );
      exit(1);
    }
  }
  set_var(
    "WRY_ANDROID_KOTLIN_FILES_OUT_DIR",
    src_main_dir.join("generated"),
  );

  (config, metadata)
}

pub fn env(non_interactive: bool) -> Result<Env> {
  let env = super::env().context("failed to setup Android environment")?;
  ensure_env(non_interactive).context("failed to ensure Android environment")?;
  cargo_mobile2::android::env::Env::from_env(env).context("failed to load Android environment")
}

fn download_cmdline_tools(extract_path: &Path) -> Result<()> {
  log::info!("Downloading Android command line tools...");

  let mut response = crate::helpers::http::get(CMDLINE_TOOLS_URL)
    .context("failed to download Android command line tools")?;
  let body = response
    .body_mut()
    .with_config()
    .limit(200 * 1024 * 1024 /* 200MB */)
    .read_to_vec()
    .context("failed to read Android command line tools download response")?;

  let mut zip = zip::ZipArchive::new(Cursor::new(body))
    .context("failed to create zip archive from Android command line tools download response")?;

  log::info!(
    "Extracting Android command line tools to {}",
    extract_path.display()
  );
  zip
    .extract(extract_path)
    .context("failed to extract Android command line tools")?;

  Ok(())
}

fn ensure_env(non_interactive: bool) -> Result<()> {
  ensure_java()?;
  ensure_sdk(non_interactive)?;
  ensure_ndk(non_interactive)?;
  Ok(())
}

fn ensure_java() -> Result<()> {
  if std::env::var_os("JAVA_HOME").is_none() {
    #[cfg(windows)]
    let default_java_home = "C:\\Program Files\\Android\\Android Studio\\jbr";
    #[cfg(target_os = "macos")]
    let default_java_home = "/Applications/Android Studio.app/Contents/jbr/Contents/Home";
    #[cfg(target_os = "linux")]
    let default_java_home = "/opt/android-studio/jbr";

    if Path::new(default_java_home).exists() {
      log::info!("Using Android Studio's default Java installation: {default_java_home}");
      std::env::set_var("JAVA_HOME", default_java_home);
    } else if which::which("java").is_err() {
      crate::error::bail!("Java not found in PATH, default Android Studio Java installation not found at {default_java_home} and JAVA_HOME environment variable not set. Please install Java before proceeding");
    }
  }

  Ok(())
}

fn ensure_sdk(non_interactive: bool) -> Result<()> {
  let android_home = std::env::var_os("ANDROID_HOME")
    .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
    .map(PathBuf::from);
  if !android_home.as_ref().is_some_and(|v| v.exists()) {
    log::info!(
      "ANDROID_HOME {}, trying to locate Android SDK...",
      if let Some(v) = &android_home {
        format!("not found at {}", v.display())
      } else {
        "not set".into()
      }
    );

    #[cfg(target_os = "macos")]
    let default_android_home = dirs::home_dir().unwrap().join("Library/Android/sdk");
    #[cfg(target_os = "linux")]
    let default_android_home = dirs::home_dir().unwrap().join("Android/Sdk");
    #[cfg(windows)]
    let default_android_home = dirs::data_local_dir().unwrap().join("Android/Sdk");

    if default_android_home.exists() {
      log::info!(
        "Using installed Android SDK: {}",
        default_android_home.display()
      );
    } else if non_interactive {
      crate::error::bail!("Android SDK not found. Make sure the SDK and NDK are installed and the ANDROID_HOME and NDK_HOME environment variables are set.");
    } else {
      log::error!(
        "Android SDK not found at {}",
        default_android_home.display()
      );

      let extract_path = if create_dir_all(&default_android_home).is_ok() {
        default_android_home.clone()
      } else {
        std::env::current_dir().context("failed to get current directory")?
      };

      let sdk_manager_path = extract_path
        .join("cmdline-tools/bin/sdkmanager")
        .with_extension(if cfg!(windows) { "bat" } else { "" });

      let mut granted_permission_to_install = false;

      if !sdk_manager_path.exists() {
        granted_permission_to_install = crate::helpers::prompts::confirm(
          "Do you want to install the Android Studio command line tools to setup the Android SDK?",
          Some(false),
        )
        .unwrap_or_default();

        if !granted_permission_to_install {
          crate::error::bail!("Skipping Android Studio command line tools installation. Please go through the manual setup process described in the documentation: https://tauri.app/start/prerequisites/#android");
        }

        download_cmdline_tools(&extract_path)?;
      }

      if !granted_permission_to_install {
        granted_permission_to_install = crate::helpers::prompts::confirm(
          "Do you want to install the Android SDK using the command line tools?",
          Some(false),
        )
        .unwrap_or_default();

        if !granted_permission_to_install {
          crate::error::bail!("Skipping Android Studio SDK installation. Please go through the manual setup process described in the documentation: https://tauri.app/start/prerequisites/#android");
        }
      }

      log::info!("Running sdkmanager to install platform-tools, android-{SDK_VERSION} and ndk-{NDK_VERSION} on {}...", default_android_home.display());
      let status = Command::new(&sdk_manager_path)
        .arg(format!("--sdk_root={}", default_android_home.display()))
        .arg("--install")
        .arg("platform-tools")
        .arg(format!("platforms;android-{SDK_VERSION}"))
        .arg(format!("ndk;{NDK_VERSION}"))
        .status()
        .map_err(|error| Error::CommandFailed {
          command: format!("{} --sdk_root={} --install platform-tools platforms;android-{SDK_VERSION} ndk;{NDK_VERSION}", sdk_manager_path.display(), default_android_home.display()),
          error,
        })?;

      if !status.success() {
        crate::error::bail!("Failed to install Android SDK");
      }
    }

    std::env::set_var("ANDROID_HOME", default_android_home);
  }

  Ok(())
}

fn ensure_ndk(non_interactive: bool) -> Result<()> {
  // re-evaluate ANDROID_HOME
  let android_home = std::env::var_os("ANDROID_HOME")
    .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
    .map(PathBuf::from)
    .context("Failed to locate Android SDK")?;
  let mut installed_ndks = read_dir(android_home.join("ndk"))
    .map(|dir| {
      dir
        .into_iter()
        .flat_map(|e| e.ok().map(|e| e.path()))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  installed_ndks.sort();

  if let Some(ndk) = installed_ndks.last() {
    log::info!("Using installed NDK: {}", ndk.display());
    std::env::set_var("NDK_HOME", ndk);
  } else if non_interactive {
    crate::error::bail!("Android NDK not found. Make sure the NDK is installed and the NDK_HOME environment variable is set.");
  } else {
    let sdk_manager_path = android_home
      .join("cmdline-tools/bin/sdkmanager")
      .with_extension(if cfg!(windows) { "bat" } else { "" });

    let mut granted_permission_to_install = false;

    if !sdk_manager_path.exists() {
      granted_permission_to_install = crate::helpers::prompts::confirm(
        "Do you want to install the Android Studio command line tools to setup the Android NDK?",
        Some(false),
      )
      .unwrap_or_default();

      if !granted_permission_to_install {
        crate::error::bail!("Skipping Android Studio command line tools installation. Please go through the manual setup process described in the documentation: https://tauri.app/start/prerequisites/#android");
      }

      download_cmdline_tools(&android_home)?;
    }

    if !granted_permission_to_install {
      granted_permission_to_install = crate::helpers::prompts::confirm(
        "Do you want to install the Android NDK using the command line tools?",
        Some(false),
      )
      .unwrap_or_default();

      if !granted_permission_to_install {
        crate::error::bail!("Skipping Android Studio NDK installation. Please go through the manual setup process described in the documentation: https://tauri.app/start/prerequisites/#android");
      }
    }

    log::info!(
      "Running sdkmanager to install ndk-{NDK_VERSION} on {}...",
      android_home.display()
    );
    let status = Command::new(&sdk_manager_path)
      .arg(format!("--sdk_root={}", android_home.display()))
      .arg("--install")
      .arg(format!("ndk;{NDK_VERSION}"))
      .status()
      .map_err(|error| Error::CommandFailed {
        command: format!(
          "{} --sdk_root={} --install ndk;{NDK_VERSION}",
          sdk_manager_path.display(),
          android_home.display()
        ),
        error,
      })?;

    if !status.success() {
      crate::error::bail!("Failed to install Android NDK");
    }

    let ndk_path = android_home.join("ndk").join(NDK_VERSION);
    log::info!("Installed NDK: {}", ndk_path.display());
    std::env::set_var("NDK_HOME", ndk_path);
  }

  Ok(())
}

fn delete_codegen_vars() {
  for (k, _) in std::env::vars() {
    if k.starts_with("WRY_") && (k.ends_with("CLASS_EXTENSION") || k.ends_with("CLASS_INIT")) {
      std::env::remove_var(k);
    }
  }
}

fn adb_device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  let device_list = adb::device_list(env).context("failed to detect connected Android devices")?;
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
        crate::error::bail!("Could not find an Android device matching {t}")
      }
    } else if device_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "Android", " devices"),
        device_list.iter(),
        "device",
        None,
        "Device",
      )
      .context("failed to prompt for device")?;
      device_list.into_iter().nth(index).unwrap()
    } else {
      device_list.into_iter().next().unwrap()
    };

    log::info!(
      "Detected connected device: {} with target {:?}",
      device,
      device.target().triple,
    );
    Ok(device)
  } else {
    Err(Error::GenericError(
      "No connected Android devices detected".to_string(),
    ))
  }
}

fn emulator_prompt(env: &'_ Env, target: Option<&str>) -> Result<emulator::Emulator> {
  let emulator_list = emulator::avd_list(env).unwrap_or_default();
  if !emulator_list.is_empty() {
    let emulator = if let Some(t) = target {
      let (device, score) = emulator_list
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
        crate::error::bail!("Could not find an Android Emulator matching {t}")
      }
    } else if emulator_list.len() > 1 {
      let index = prompt::list(
        concat!("Detected ", "Android", " emulators"),
        emulator_list.iter(),
        "emulator",
        None,
        "Emulator",
      )
      .context("failed to prompt for emulator")?;
      emulator_list.into_iter().nth(index).unwrap()
    } else {
      emulator_list.into_iter().next().unwrap()
    };

    Ok(emulator)
  } else {
    Err(Error::GenericError(
      "No available Android Emulator detected".to_string(),
    ))
  }
}

fn device_prompt<'a>(env: &'_ Env, target: Option<&str>) -> Result<Device<'a>> {
  if let Ok(device) = adb_device_prompt(env, target) {
    Ok(device)
  } else {
    let emulator = emulator_prompt(env, target)?;
    log::info!("Starting emulator {}", emulator.name());
    emulator
      .start_detached(env)
      .context("failed to start emulator")?;
    let mut tries = 0;
    loop {
      sleep(Duration::from_secs(2));
      if let Ok(device) = adb_device_prompt(env, Some(emulator.name())) {
        return Ok(device);
      }
      if tries >= 3 {
        log::info!("Waiting for emulator to start... (maybe the emulator is unauthorized or offline, run `adb devices` to check)");
      } else {
        log::info!("Waiting for emulator to start...");
      }
      tries += 1;
    }
  }
}

fn detect_target_ok<'a>(env: &Env) -> Option<&'a Target<'a>> {
  device_prompt(env, None).map(|device| device.target()).ok()
}

fn open_and_wait(config: &AndroidConfig, env: &Env) -> ! {
  log::info!("Opening Android Studio");
  if let Err(e) = os::open_file_with("Android Studio", config.project_dir(), &env.base) {
    log::error!("{e}");
  }
  loop {
    sleep(Duration::from_secs(24 * 60 * 60));
  }
}

fn inject_resources(config: &AndroidConfig, tauri_config: &TauriConfig) -> Result<()> {
  let asset_dir = config
    .project_dir()
    .join("app/src/main")
    .join(DEFAULT_ASSET_DIR);
  create_dir_all(&asset_dir).fs_context("failed to create asset directory", asset_dir.clone())?;

  write(
    asset_dir.join("tauri.conf.json"),
    serde_json::to_string(&tauri_config).with_context(|| "failed to serialize tauri config")?,
  )
  .fs_context(
    "failed to write tauri config",
    asset_dir.join("tauri.conf.json"),
  )?;

  let resources = match &tauri_config.bundle.resources {
    Some(BundleResources::List(paths)) => Some(ResourcePaths::new(paths.as_slice(), true)),
    Some(BundleResources::Map(map)) => Some(ResourcePaths::from_map(map, true)),
    None => None,
  };
  if let Some(resources) = resources {
    for resource in resources.iter() {
      let resource = resource.context("failed to get resource")?;
      let dest = asset_dir.join(resource.target());
      crate::helpers::fs::copy_file(resource.path(), dest).context("failed to copy resource")?;
    }
  }

  Ok(())
}

fn configure_cargo(env: &mut Env, config: &AndroidConfig) -> Result<()> {
  for target in Target::all().values() {
    let config = target
      .generate_cargo_config(config, env)
      .context("failed to find Android tool")?;
    let target_var_name = target.triple.replace('-', "_").to_uppercase();
    if let Some(linker) = config.linker {
      env.base.insert_env_var(
        format!("CARGO_TARGET_{target_var_name}_LINKER"),
        linker.into(),
      );
    }
    env.base.insert_env_var(
      format!("CARGO_TARGET_{target_var_name}_RUSTFLAGS"),
      config.rustflags.join(" ").into(),
    );
  }

  Ok(())
}

fn generate_tauri_properties(
  config: &AndroidConfig,
  tauri_config: &TauriConfig,
  dev: bool,
) -> Result<()> {
  let app_tauri_properties_path = config.project_dir().join("app").join("tauri.properties");

  let mut app_tauri_properties = Vec::new();
  if let Some(version) = tauri_config.version.as_ref() {
    app_tauri_properties.push(format!("tauri.android.versionName={version}"));
    if tauri_config.bundle.android.auto_increment_version_code && !dev {
      let last_version_code = std::fs::read_to_string(&app_tauri_properties_path)
        .ok()
        .and_then(|content| {
          content
            .lines()
            .find(|line| line.starts_with("tauri.android.versionCode="))
            .and_then(|line| line.split('=').nth(1))
            .and_then(|s| s.trim().parse::<u32>().ok())
        });
      let new_version_code = last_version_code.map(|v| v.saturating_add(1)).unwrap_or(1);
      app_tauri_properties.push(format!("tauri.android.versionCode={new_version_code}"));
    } else if let Some(version_code) = tauri_config.bundle.android.version_code.as_ref() {
      app_tauri_properties.push(format!("tauri.android.versionCode={version_code}"));
    } else if let Ok(version) = Version::parse(version) {
      let mut version_code = version.major * 1000000 + version.minor * 1000 + version.patch;

      if dev {
        version_code = version_code.clamp(1, 2100000000);
      }

      if version_code == 0 {
        crate::error::bail!(
          "You must change the `version` in `tauri.conf.json`. The default value `0.0.0` is not allowed for Android package and must be at least `0.0.1`."
        );
      } else if version_code > 2100000000 {
        crate::error::bail!(
          "Invalid version code {}. Version code must be between 1 and 2100000000. You must change the `version` in `tauri.conf.json`.",
          version_code
        );
      }

      app_tauri_properties.push(format!("tauri.android.versionCode={version_code}"));
    }
  }

  if !app_tauri_properties.is_empty() {
    let app_tauri_properties_content = format!(
      "// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.\n{}",
      app_tauri_properties.join("\n")
    );
    if std::fs::read_to_string(&app_tauri_properties_path)
      .map(|o| o != app_tauri_properties_content)
      .unwrap_or(true)
    {
      write(&app_tauri_properties_path, app_tauri_properties_content)
        .context("failed to write tauri.properties")?;
    }
  }

  Ok(())
}
