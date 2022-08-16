// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::config::get as get_tauri_config;
use cargo_mobile::{
  apple::{
    config::{Config as AppleConfig, Metadata as AppleMetadata},
    target::{CompileLibError, Target},
  },
  env::{Env, Error as EnvError},
  opts::{NoiseLevel, Profile},
  os, util,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init, get_config, get_metadata,
  init::{command as init_command, Options as InitOptions},
  Target as MobileTarget,
};
use crate::Result;
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

pub(crate) mod project;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  EnvInitFailed(EnvError),
  #[error("invalid tauri configuration: {0}")]
  InvalidTauriConfig(String),
  #[error("{0}")]
  ProjectNotInitialized(String),
  #[error(transparent)]
  OpenFailed(os::OpenFileError),
  #[error(transparent)]
  NoHomeDir(util::NoHomeDir),
  #[error("SDK root provided by Xcode was invalid. {sdk_root} doesn't exist or isn't a directory")]
  SdkRootInvalid { sdk_root: PathBuf },
  #[error("Include dir was invalid. {include_dir} doesn't exist or isn't a directory")]
  IncludeDirInvalid { include_dir: PathBuf },
  #[error("macOS SDK root was invalid. {macos_sdk_root} doesn't exist or isn't a directory")]
  MacosSdkRootInvalid { macos_sdk_root: PathBuf },
  #[error("Arch specified by Xcode was invalid. {arch} isn't a known arch")]
  ArchInvalid { arch: String },
  #[error(transparent)]
  CompileLibFailed(CompileLibError),
}

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
pub struct XcodeScriptOptions {
  /// Value of `PLATFORM_DISPLAY_NAME` env var
  #[clap(long)]
  platform: String,
  /// Value of `SDKROOT` env var
  #[clap(long)]
  sdk_root: PathBuf,
  /// Value of `CONFIGURATION` env var
  #[clap(long)]
  configuration: String,
  /// Value of `FORCE_COLOR` env var
  #[clap(long)]
  force_color: bool,
  /// Value of `ARCHS` env var
  #[clap(index = 1, required = true)]
  arches: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Open,
  #[clap(hide(true))]
  XcodeScript(XcodeScriptOptions),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, MobileTarget::Ios)?,
    Commands::Open => open()?,
    Commands::XcodeScript(options) => xcode_script(options)?,
  }

  Ok(())
}

fn with_config(
  f: impl FnOnce(&AppleConfig, &AppleMetadata) -> Result<(), Error>,
) -> Result<(), Error> {
  let tauri_config =
    get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
  let tauri_config_guard = tauri_config.lock().unwrap();
  let tauri_config_ = tauri_config_guard.as_ref().unwrap();
  let config = get_config(tauri_config_);
  let metadata = get_metadata(tauri_config_);
  f(config.apple(), metadata.apple())
}

fn open() -> Result<()> {
  with_config(|config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Ios)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Xcode", config.project_dir()).map_err(Error::OpenFailed)
  })?;
  Ok(())
}

fn xcode_script(options: XcodeScriptOptions) -> Result<()> {
  fn macos_from_platform(platform: &str) -> bool {
    platform == "macOS"
  }

  fn profile_from_configuration(configuration: &str) -> Profile {
    if configuration == "release" {
      Profile::Release
    } else {
      Profile::Debug
    }
  }

  let profile = profile_from_configuration(&options.configuration);
  let macos = macos_from_platform(&options.platform);

  with_config(|config, metadata| {
    let env = Env::new().map_err(Error::EnvInitFailed)?;
    // The `PATH` env var Xcode gives us is missing any additions
    // made by the user's profile, so we'll manually add cargo's
    // `PATH`.
    let env = env.prepend_to_path(
      util::home_dir()
        .map_err(Error::NoHomeDir)?
        .join(".cargo/bin"),
    );

    if !options.sdk_root.is_dir() {
      return Err(Error::SdkRootInvalid {
        sdk_root: options.sdk_root,
      });
    }
    let include_dir = options.sdk_root.join("usr/include");
    if !include_dir.is_dir() {
      return Err(Error::IncludeDirInvalid { include_dir });
    }

    let mut host_env = HashMap::<&str, &OsStr>::new();

    // Host flags that are used by build scripts
    let (macos_isysroot, library_path) = {
      let macos_sdk_root = options
        .sdk_root
        .join("../../../../MacOSX.platform/Developer/SDKs/MacOSX.sdk");
      if !macos_sdk_root.is_dir() {
        return Err(Error::MacosSdkRootInvalid { macos_sdk_root });
      }
      (
        format!("-isysroot {}", macos_sdk_root.display()),
        format!("{}/usr/lib", macos_sdk_root.display()),
      )
    };
    host_env.insert("MAC_FLAGS", macos_isysroot.as_ref());
    host_env.insert("CFLAGS_x86_64_apple_darwin", macos_isysroot.as_ref());
    host_env.insert("CXXFLAGS_x86_64_apple_darwin", macos_isysroot.as_ref());

    host_env.insert(
      "OBJC_INCLUDE_PATH_x86_64_apple_darwin",
      include_dir.as_os_str(),
    );

    host_env.insert("RUST_BACKTRACE", "1".as_ref());

    let macos_target = Target::macos();

    let isysroot = format!("-isysroot {}", options.sdk_root.display());

    for arch in options.arches {
      // Set target-specific flags
      let triple = match arch.as_str() {
        "arm64" => "aarch64_apple_ios",
        "x86_64" => "x86_64_apple_ios",
        _ => return Err(Error::ArchInvalid { arch }),
      };
      let cflags = format!("CFLAGS_{}", triple);
      let cxxflags = format!("CFLAGS_{}", triple);
      let objc_include_path = format!("OBJC_INCLUDE_PATH_{}", triple);
      let mut target_env = host_env.clone();
      target_env.insert(cflags.as_ref(), isysroot.as_ref());
      target_env.insert(cxxflags.as_ref(), isysroot.as_ref());
      target_env.insert(objc_include_path.as_ref(), include_dir.as_ref());
      // Prevents linker errors in build scripts and proc macros:
      // https://github.com/signalapp/libsignal-client/commit/02899cac643a14b2ced7c058cc15a836a2165b6d
      target_env.insert("LIBRARY_PATH", library_path.as_ref());

      let target = if macos {
        &macos_target
      } else {
        Target::for_arch(&arch).ok_or_else(|| Error::ArchInvalid {
          arch: arch.to_owned(),
        })?
      };
      target
        .compile_lib(
          config,
          metadata,
          NoiseLevel::Polite,
          true.into(),
          profile,
          &env,
          target_env,
        )
        .map_err(Error::CompileLibFailed)?;
    }
    Ok(())
  })?;

  Ok(())
}
