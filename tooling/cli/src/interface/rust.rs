// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::File,
  io::Read,
  path::{Path, PathBuf},
  process::Command,
  str::FromStr,
};

use anyhow::Context;
#[cfg(target_os = "linux")]
use heck::ToKebabCase;
use log::warn;
use serde::Deserialize;

use crate::{
  helpers::{
    app_paths::tauri_dir,
    config::{wix_settings, Config},
    manifest::Manifest,
  },
  CommandExt,
};
use tauri_bundler::{
  AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings,
  UpdaterSettings, WindowsSettings,
};

/// The `workspace` section of the app configuration (read from Cargo.toml).
#[derive(Clone, Debug, Deserialize)]
struct WorkspaceSettings {
  /// the workspace members.
  members: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct BinarySettings {
  name: String,
  path: Option<String>,
}

/// The package settings.
#[derive(Debug, Clone, Deserialize)]
pub struct CargoPackageSettings {
  /// the package's name.
  pub name: Option<String>,
  /// the package's version.
  pub version: Option<String>,
  /// the package's description.
  pub description: Option<String>,
  /// the package's homepage.
  pub homepage: Option<String>,
  /// the package's authors.
  pub authors: Option<Vec<String>>,
  /// the default binary to run.
  pub default_run: Option<String>,
}

/// The Cargo settings (Cargo.toml root descriptor).
#[derive(Clone, Debug, Deserialize)]
struct CargoSettings {
  /// the package settings.
  ///
  /// it's optional because ancestor workspace Cargo.toml files may not have package info.
  package: Option<CargoPackageSettings>,
  /// the workspace settings.
  ///
  /// it's present if the read Cargo.toml belongs to a workspace root.
  workspace: Option<WorkspaceSettings>,
  /// the binary targets configuration.
  bin: Option<Vec<BinarySettings>>,
}

impl CargoSettings {
  /// Try to load a set of CargoSettings from a "Cargo.toml" file in the specified directory.
  fn load(dir: &Path) -> crate::Result<Self> {
    let toml_path = dir.join("Cargo.toml");
    let mut toml_str = String::new();
    let mut toml_file = File::open(toml_path).with_context(|| "failed to open Cargo.toml")?;
    toml_file
      .read_to_string(&mut toml_str)
      .with_context(|| "failed to read Cargo.toml")?;
    toml::from_str(&toml_str)
      .with_context(|| "failed to parse Cargo.toml")
      .map_err(Into::into)
  }
}

#[derive(Deserialize)]
struct CargoBuildConfig {
  #[serde(rename = "target-dir")]
  target_dir: Option<String>,
}

#[derive(Deserialize)]
struct CargoConfig {
  build: Option<CargoBuildConfig>,
}

pub fn build_project(runner: String, args: Vec<String>) -> crate::Result<()> {
  Command::new(&runner)
    .args(&["build", "--features=custom-protocol"])
    .args(args)
    .pipe()?
    .output_ok()
    .with_context(|| format!("Result of `{} build` operation was unsuccessful", runner))?;

  Ok(())
}

pub struct AppSettings {
  cargo_settings: CargoSettings,
  cargo_package_settings: CargoPackageSettings,
  package_settings: PackageSettings,
}

impl AppSettings {
  pub fn new(config: &Config) -> crate::Result<Self> {
    let cargo_settings =
      CargoSettings::load(&tauri_dir()).with_context(|| "failed to load cargo settings")?;
    let cargo_package_settings = match &cargo_settings.package {
      Some(package_info) => package_info.clone(),
      None => {
        return Err(anyhow::anyhow!(
          "No package info in the config file".to_owned(),
        ))
      }
    };

    let package_settings = PackageSettings {
      product_name: config.package.product_name.clone().unwrap_or_else(|| {
        cargo_package_settings
          .name
          .clone()
          .expect("Cargo manifest must have the `package.name` field")
      }),
      version: config.package.version.clone().unwrap_or_else(|| {
        cargo_package_settings
          .version
          .clone()
          .expect("Cargo manifest must have the `package.version` field")
      }),
      description: cargo_package_settings
        .description
        .clone()
        .unwrap_or_default(),
      homepage: cargo_package_settings.homepage.clone(),
      authors: cargo_package_settings.authors.clone(),
      default_run: cargo_package_settings.default_run.clone(),
    };

    Ok(Self {
      cargo_settings,
      cargo_package_settings,
      package_settings,
    })
  }

  pub fn cargo_package_settings(&self) -> &CargoPackageSettings {
    &self.cargo_package_settings
  }

  pub fn get_bundle_settings(
    &self,
    config: &Config,
    manifest: &Manifest,
    features: &[String],
  ) -> crate::Result<BundleSettings> {
    tauri_config_to_bundle_settings(
      manifest,
      features,
      config.tauri.bundle.clone(),
      config.tauri.system_tray.clone(),
      config.tauri.updater.clone(),
    )
  }

  pub fn get_out_dir(&self, target: Option<String>, debug: bool) -> crate::Result<PathBuf> {
    let tauri_dir = tauri_dir();
    let workspace_dir = get_workspace_dir(&tauri_dir);
    get_target_dir(&workspace_dir, target, !debug)
  }

  pub fn get_package_settings(&self) -> PackageSettings {
    self.package_settings.clone()
  }

  pub fn get_binaries(&self, config: &Config, target: &str) -> crate::Result<Vec<BundleBinary>> {
    let mut binaries: Vec<BundleBinary> = vec![];

    let binary_extension: String = if target.contains("windows") {
      ".exe"
    } else {
      ""
    }
    .into();

    if let Some(bin) = &self.cargo_settings.bin {
      let default_run = self
        .package_settings
        .default_run
        .clone()
        .unwrap_or_else(|| "".to_string());
      for binary in bin {
        binaries.push(
          if Some(&binary.name) == self.cargo_package_settings.name.as_ref()
            || binary.name.as_str() == default_run
          {
            BundleBinary::new(
              format!(
                "{}{}",
                config
                  .package
                  .binary_name()
                  .unwrap_or_else(|| binary.name.clone()),
                &binary_extension
              ),
              true,
            )
          } else {
            BundleBinary::new(
              format!("{}{}", binary.name.clone(), &binary_extension),
              false,
            )
          }
          .set_src_path(binary.path.clone()),
        )
      }
    }

    let mut bins_path = tauri_dir();
    bins_path.push("src/bin");
    if let Ok(fs_bins) = std::fs::read_dir(bins_path) {
      for entry in fs_bins {
        let path = entry?.path();
        if let Some(name) = path.file_stem() {
          let bin_exists = binaries.iter().any(|bin| {
            bin.name() == name || path.ends_with(bin.src_path().unwrap_or(&"".to_string()))
          });
          if !bin_exists {
            binaries.push(BundleBinary::new(
              format!("{}{}", name.to_string_lossy(), &binary_extension),
              false,
            ))
          }
        }
      }
    }

    if let Some(default_run) = self.package_settings.default_run.as_ref() {
      match binaries.iter_mut().find(|bin| bin.name() == default_run) {
        Some(bin) => {
          if let Some(bin_name) = config.package.binary_name() {
            bin.set_name(bin_name);
          }
        }
        None => {
          binaries.push(BundleBinary::new(
            format!(
              "{}{}",
              config
                .package
                .binary_name()
                .unwrap_or_else(|| default_run.to_string()),
              &binary_extension
            ),
            true,
          ));
        }
      }
    }

    match binaries.len() {
      0 => binaries.push(BundleBinary::new(
        #[cfg(target_os = "linux")]
        self.package_settings.product_name.to_kebab_case(),
        #[cfg(not(target_os = "linux"))]
        format!(
          "{}{}",
          self.package_settings.product_name.clone(),
          &binary_extension
        ),
        true,
      )),
      1 => binaries.get_mut(0).unwrap().set_main(true),
      _ => {}
    }

    Ok(binaries)
  }
}

/// This function determines where 'target' dir is and suffixes it with 'release' or 'debug'
/// to determine where the compiled binary will be located.
fn get_target_dir(
  project_root_dir: &Path,
  target: Option<String>,
  is_release: bool,
) -> crate::Result<PathBuf> {
  let mut path: PathBuf = match std::env::var_os("CARGO_TARGET_DIR") {
    Some(target_dir) => target_dir.into(),
    None => {
      let mut root_dir = project_root_dir.to_path_buf();
      let target_path: Option<PathBuf> = loop {
        // cargo reads configs under .cargo/config.toml or .cargo/config
        let mut cargo_config_path = root_dir.join(".cargo/config");
        if !cargo_config_path.exists() {
          cargo_config_path = root_dir.join(".cargo/config.toml");
        }
        // if the path exists, parse it
        if cargo_config_path.exists() {
          let mut config_str = String::new();
          let mut config_file = File::open(&cargo_config_path)
            .with_context(|| format!("failed to open {:?}", cargo_config_path))?;
          config_file
            .read_to_string(&mut config_str)
            .with_context(|| "failed to read cargo config file")?;
          let config: CargoConfig =
            toml::from_str(&config_str).with_context(|| "failed to parse cargo config file")?;
          if let Some(build) = config.build {
            if let Some(target_dir) = build.target_dir {
              break Some(target_dir.into());
            }
          }
        }
        if !root_dir.pop() {
          break None;
        }
      };
      target_path.unwrap_or_else(|| project_root_dir.join("target"))
    }
  };

  if let Some(ref triple) = target {
    path.push(triple);
  }
  path.push(if is_release { "release" } else { "debug" });
  Ok(path)
}

/// Walks up the file system, looking for a Cargo.toml file
/// If one is found before reaching the root, then the current_dir's package belongs to that parent workspace if it's listed on [workspace.members].
///
/// If this package is part of a workspace, returns the path to the workspace directory
/// Otherwise returns the current directory.
pub fn get_workspace_dir(current_dir: &Path) -> PathBuf {
  let mut dir = current_dir.to_path_buf();
  let project_path = dir.clone();

  while dir.pop() {
    if dir.join("Cargo.toml").exists() {
      match CargoSettings::load(&dir) {
        Ok(cargo_settings) => {
          if let Some(workspace_settings) = cargo_settings.workspace {
            if let Some(members) = workspace_settings.members {
              if members.iter().any(|member| {
                glob::glob(&dir.join(member).to_string_lossy())
                  .unwrap()
                  .any(|p| p.unwrap() == project_path)
              }) {
                return dir;
              }
            }
          }
        }
        Err(e) => {
          warn!(
              "Found `{}`, which may define a parent workspace, but \
            failed to parse it. If this is indeed a parent workspace, undefined behavior may occur: \
            \n    {:#}",
              dir.display(),
              e
            );
        }
      }
    }
  }

  // Nothing found walking up the file system, return the starting directory
  current_dir.to_path_buf()
}

#[allow(unused_variables)]
fn tauri_config_to_bundle_settings(
  manifest: &Manifest,
  features: &[String],
  config: crate::helpers::config::BundleConfig,
  system_tray_config: Option<crate::helpers::config::SystemTrayConfig>,
  updater_config: crate::helpers::config::UpdaterConfig,
) -> crate::Result<BundleSettings> {
  let enabled_features = manifest.all_enabled_features(features);

  #[cfg(windows)]
  let windows_icon_path = PathBuf::from(
    config
      .icon
      .iter()
      .find(|i| i.ends_with(".ico"))
      .cloned()
      .expect("the bundle config must have a `.ico` icon"),
  );
  #[cfg(not(windows))]
  let windows_icon_path = PathBuf::from("");

  #[allow(unused_mut)]
  let mut resources = config.resources.unwrap_or_default();
  #[allow(unused_mut)]
  let mut depends = config.deb.depends.unwrap_or_default();

  #[cfg(target_os = "linux")]
  {
    if let Some(system_tray_config) = &system_tray_config {
      let mut icon_path = system_tray_config.icon_path.clone();
      icon_path.set_extension("png");
      resources.push(icon_path.display().to_string());
      if enabled_features.contains(&"tauri/gtk-tray".into()) {
        depends.push("libappindicator3-1".into());
      } else {
        depends.push("libayatana-appindicator3-1".into());
      }
    }

    // provides `libwebkit2gtk-4.0.so.37` and all `4.0` versions have the -37 package name
    depends.push("libwebkit2gtk-4.0-37".to_string());
    depends.push("libgtk-3-0".to_string());
  }

  #[cfg(windows)]
  {
    if let Some(webview_fixed_runtime_path) = &config.windows.webview_fixed_runtime_path {
      resources.push(webview_fixed_runtime_path.display().to_string());
    }
  }

  let signing_identity = match std::env::var_os("APPLE_SIGNING_IDENTITY") {
    Some(signing_identity) => Some(
      signing_identity
        .to_str()
        .expect("failed to convert APPLE_SIGNING_IDENTITY to string")
        .to_string(),
    ),
    None => config.macos.signing_identity,
  };

  let provider_short_name = match std::env::var_os("APPLE_PROVIDER_SHORT_NAME") {
    Some(provider_short_name) => Some(
      provider_short_name
        .to_str()
        .expect("failed to convert APPLE_PROVIDER_SHORT_NAME to string")
        .to_string(),
    ),
    None => config.macos.provider_short_name,
  };

  Ok(BundleSettings {
    identifier: Some(config.identifier),
    icon: Some(config.icon),
    resources: if resources.is_empty() {
      None
    } else {
      Some(resources)
    },
    copyright: config.copyright,
    category: match config.category {
      Some(category) => Some(AppCategory::from_str(&category).map_err(|e| match e {
        Some(e) => anyhow::anyhow!("invalid category, did you mean `{}`?", e),
        None => anyhow::anyhow!("invalid category"),
      })?),
      None => None,
    },
    short_description: config.short_description,
    long_description: config.long_description,
    external_bin: config.external_bin,
    deb: DebianSettings {
      depends: if depends.is_empty() {
        None
      } else {
        Some(depends)
      },
      files: config.deb.files,
    },
    macos: MacOsSettings {
      frameworks: config.macos.frameworks,
      minimum_system_version: config.macos.minimum_system_version,
      license: config.macos.license,
      exception_domain: config.macos.exception_domain,
      signing_identity,
      provider_short_name,
      entitlements: config.macos.entitlements,
      info_plist_path: {
        let path = tauri_dir().join("Info.plist");
        if path.exists() {
          Some(path)
        } else {
          None
        }
      },
    },
    windows: WindowsSettings {
      timestamp_url: config.windows.timestamp_url,
      tsp: config.windows.tsp,
      digest_algorithm: config.windows.digest_algorithm,
      certificate_thumbprint: config.windows.certificate_thumbprint,
      wix: config.windows.wix.map(|w| {
        let mut wix = wix_settings(w);
        wix.license = wix.license.map(|l| tauri_dir().join(l));
        wix
      }),
      icon_path: windows_icon_path,
      webview_fixed_runtime_path: config.windows.webview_fixed_runtime_path,
      allow_downgrades: config.windows.allow_downgrades,
    },
    updater: Some(UpdaterSettings {
      active: updater_config.active,
      // we set it to true by default we shouldn't have to use
      // unwrap_or as we have a default value but used to prevent any failing
      dialog: updater_config.dialog,
      pubkey: updater_config.pubkey,
      endpoints: updater_config
        .endpoints
        .map(|endpoints| endpoints.iter().map(|e| e.to_string()).collect()),
      msiexec_args: Some(updater_config.windows.install_mode.msiexec_args()),
    }),
    ..Default::default()
  })
}
