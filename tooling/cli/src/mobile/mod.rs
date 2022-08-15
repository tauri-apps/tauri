// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::{app_paths::tauri_dir, config::Config as TauriConfig};
use anyhow::{bail, Result};
#[cfg(target_os = "macos")]
use cargo_mobile::apple::config::{
  Metadata as AppleMetadata, Platform as ApplePlatform, Raw as RawAppleConfig,
};
use cargo_mobile::{
  android::config::{Metadata as AndroidMetadata, Raw as RawAndroidConfig},
  config::{app::Raw as RawAppConfig, metadata::Metadata, Config, Raw},
};
use std::path::PathBuf;

pub mod android;
mod init;
#[cfg(target_os = "macos")]
pub mod ios;

#[derive(PartialEq, Eq)]
pub enum Target {
  Android,
  #[cfg(target_os = "macos")]
  Ios,
}

impl Target {
  fn ide_name(&self) -> &'static str {
    match self {
      Self::Android => "Android Studio",
      #[cfg(target_os = "macos")]
      Self::Ios => "Xcode",
    }
  }

  fn command_name(&self) -> &'static str {
    match self {
      Self::Android => "android",
      #[cfg(target_os = "macos")]
      Self::Ios => "ios",
    }
  }
}

fn get_metadata(_config: &TauriConfig) -> Metadata {
  Metadata {
    #[cfg(target_os = "macos")]
    apple: AppleMetadata {
      supported: true,
      ios: ApplePlatform {
        features: None,
        frameworks: None,
        valid_archs: None,
        vendor_frameworks: None,
        vendor_sdks: None,
        asset_catalogs: None,
        pods: None,
        additional_targets: None,
        pre_build_scripts: None,
        post_compile_scripts: None,
        post_build_scripts: None,
        command_line_arguments: None,
      },
      macos: Default::default(),
    },
    android: AndroidMetadata {
      supported: true,
      features: None,
      app_sources: None,
      app_plugins: None,
      project_dependencies: None,
      app_dependencies: None,
      app_dependencies_platform: None,
      asset_packs: None,
    },
  }
}

fn get_config(config: &TauriConfig) -> Config {
  let mut s = config.tauri.bundle.identifier.rsplit('.');
  let app_name = s.next().unwrap_or("app").to_string();
  let mut domain = String::new();
  for w in s {
    domain.push_str(w);
    domain.push('.');
  }
  domain.pop();
  let raw = Raw {
    app: RawAppConfig {
      name: app_name,
      stylized_name: config.package.product_name.clone(),
      domain,
      asset_dir: None,
      template_pack: None,
    },
    #[cfg(target_os = "macos")]
    apple: Some(RawAppleConfig {
      development_team: std::env::var("APPLE_DEVELOPMENT_TEAM")
        .ok()
        .or_else(|| config.tauri.ios.development_team)
        .expect("you must set `tauri > iOS > developmentTeam` config value or the `APPLE_DEVELOPMENT_TEAM` environment variable"),
      project_dir: None,
      ios_no_default_features: None,
      ios_features: None,
      macos_no_default_features: None,
      macos_features: None,
      bundle_version: None,
      bundle_version_short: None,
      ios_version: None,
      macos_version: None,
      use_legacy_build_system: None,
      plist_pairs: None,
    }),
    android: Some(RawAndroidConfig {
      min_sdk_version: None,
      vulkan_validation: None,
      project_dir: None,
      no_default_features: None,
      features: None,
    }),
  };
  Config::from_raw(tauri_dir(), raw).unwrap()
}

fn ensure_init(project_dir: PathBuf, target: Target) -> Result<()> {
  if !project_dir.exists() {
    bail!(
      "{} project directory {} doesn't exist. Please run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  } else {
    Ok(())
  }
}
