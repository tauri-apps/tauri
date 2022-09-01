use super::{env, with_config};
use crate::Result;
use clap::Parser;

use cargo_mobile::{apple::target::Target, opts::Profile, util};

use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

#[derive(Debug, Parser)]
pub struct Options {
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

pub fn command(options: Options) -> Result<()> {
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

  with_config(None, |_root_conf, config, metadata, cli_options| {
    let env = env()?;
    // The `PATH` env var Xcode gives us is missing any additions
    // made by the user's profile, so we'll manually add cargo's
    // `PATH`.
    let env = env.prepend_to_path(util::home_dir()?.join(".cargo/bin"));

    if !options.sdk_root.is_dir() {
      return Err(anyhow::anyhow!(
        "SDK root provided by Xcode was invalid. {} doesn't exist or isn't a directory",
        options.sdk_root.display(),
      ));
    }
    let include_dir = options.sdk_root.join("usr/include");
    if !include_dir.is_dir() {
      return Err(anyhow::anyhow!(
        "Include dir was invalid. {} doesn't exist or isn't a directory",
        include_dir.display()
      ));
    }

    let mut host_env = HashMap::<&str, &OsStr>::new();

    host_env.insert("RUST_BACKTRACE", "1".as_ref());

    let macos_target = Target::macos();

    let isysroot = format!("-isysroot {}", options.sdk_root.display());

    for arch in options.arches {
      // Set target-specific flags
      let triple = match arch.as_str() {
        "arm64" => "aarch64_apple_ios",
        "arm64-sim" => "aarch64_apple_ios_sim",
        "x86_64" => "x86_64_apple_ios",
        _ => {
          return Err(anyhow::anyhow!(
            "Arch specified by Xcode was invalid. {} isn't a known arch",
            arch
          ))
        }
      };
      let cflags = format!("CFLAGS_{}", triple);
      let cxxflags = format!("CFLAGS_{}", triple);
      let objc_include_path = format!("OBJC_INCLUDE_PATH_{}", triple);
      let mut target_env = host_env.clone();
      target_env.insert(cflags.as_ref(), isysroot.as_ref());
      target_env.insert(cxxflags.as_ref(), isysroot.as_ref());
      target_env.insert(objc_include_path.as_ref(), include_dir.as_ref());

      let target = if macos {
        &macos_target
      } else {
        Target::for_arch(&arch).ok_or_else(|| {
          anyhow::anyhow!(
            "Arch specified by Xcode was invalid. {} isn't a known arch",
            arch
          )
        })?
      };
      target.compile_lib(
        config,
        metadata,
        cli_options.noise_level,
        true,
        profile,
        &env,
        target_env,
      )?;
    }
    Ok(())
  })
  .map_err(Into::into)
}
