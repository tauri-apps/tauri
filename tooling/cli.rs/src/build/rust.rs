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

use serde::Deserialize;

use crate::helpers::{app_paths::tauri_dir, config::Config};
#[cfg(windows)]
use tauri_bundler::WindowsSettings;
use tauri_bundler::{
  AppCategory, BundleBinary, BundleSettings, DebianSettings, MacOsSettings, PackageSettings,
  UpdaterSettings,
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
  pub name: String,
  /// the package's version.
  pub version: String,
  /// the package's description.
  pub description: String,
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
    let mut toml_file = File::open(toml_path)?;
    toml_file.read_to_string(&mut toml_str)?;
    toml::from_str(&toml_str).map_err(Into::into)
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

pub fn build_project(debug: bool) -> crate::Result<()> {
  let mut args = vec!["build", "--features=custom-protocol"];

  if !debug {
    args.push("--release");
  }

  let status = Command::new("cargo").args(args).status()?;
  if !status.success() {
    return Err(anyhow::anyhow!(format!(
      "Result of `cargo build` operation was unsuccessful: {}",
      status
    )));
  }

  Ok(())
}

pub struct AppSettings {
  cargo_settings: CargoSettings,
  cargo_package_settings: CargoPackageSettings,
  package_settings: PackageSettings,
}

impl AppSettings {
  pub fn new(config: &Config) -> crate::Result<Self> {
    let cargo_settings = CargoSettings::load(&tauri_dir())?;
    let cargo_package_settings = match &cargo_settings.package {
      Some(package_info) => package_info.clone(),
      None => {
        return Err(anyhow::anyhow!(
          "No package info in the config file".to_owned(),
        ))
      }
    };

    let package_settings = PackageSettings {
      product_name: config
        .package
        .product_name
        .clone()
        .unwrap_or_else(|| cargo_package_settings.name.clone()),
      version: config
        .package
        .version
        .clone()
        .unwrap_or_else(|| cargo_package_settings.version.clone()),
      description: cargo_package_settings.description.clone(),
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

  pub fn get_bundle_settings(&self, config: &Config) -> crate::Result<BundleSettings> {
    tauri_config_to_bundle_settings(config.tauri.bundle.clone(), config.tauri.updater.clone())
  }

  pub fn get_out_dir(&self, debug: bool) -> crate::Result<PathBuf> {
    let tauri_dir = tauri_dir();
    let workspace_dir = get_workspace_dir(&tauri_dir);
    get_target_dir(&workspace_dir, None, !debug)
  }

  pub fn get_package_settings(&self) -> PackageSettings {
    self.package_settings.clone()
  }

  pub fn get_binaries(&self, config: &Config) -> crate::Result<Vec<BundleBinary>> {
    let mut binaries: Vec<BundleBinary> = vec![];
    if let Some(bin) = &self.cargo_settings.bin {
      let default_run = self
        .package_settings
        .default_run
        .clone()
        .unwrap_or_else(|| "".to_string());
      for binary in bin {
        binaries.push(
          if binary.name.as_str() == self.cargo_package_settings.name
            || binary.name.as_str() == default_run
          {
            BundleBinary::new(
              config
                .package
                .product_name
                .clone()
                .unwrap_or_else(|| binary.name.clone()),
              true,
            )
          } else {
            BundleBinary::new(binary.name.clone(), false)
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
            bin.name() == name || path.ends_with(bin.src_path().as_ref().unwrap_or(&"".to_string()))
          });
          if !bin_exists {
            binaries.push(BundleBinary::new(name.to_string_lossy().to_string(), false))
          }
        }
      }
    }

    if let Some(default_run) = self.package_settings.default_run.as_ref() {
      match binaries.iter_mut().find(|bin| bin.name() == default_run) {
        Some(bin) => {
          if let Some(product_name) = config.package.product_name.clone() {
            bin.set_name(product_name);
          }
        }
        None => {
          binaries.push(BundleBinary::new(
            config
              .package
              .product_name
              .clone()
              .unwrap_or_else(|| default_run.to_string()),
            true,
          ));
        }
      }
    }

    match binaries.len() {
      0 => binaries.push(BundleBinary::new(
        self.package_settings.product_name.clone(),
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
          let mut config_file = File::open(cargo_config_path)?;
          config_file.read_to_string(&mut config_str)?;
          let config: CargoConfig = toml::from_str(&config_str)?;
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
    if let Ok(cargo_settings) = CargoSettings::load(&dir) {
      if let Some(workspace_settings) = cargo_settings.workspace {
        if let Some(members) = workspace_settings.members {
          if members
            .iter()
            .any(|member| dir.join(member) == project_path)
          {
            return dir;
          }
        }
      }
    }
  }

  // Nothing found walking up the file system, return the starting directory
  current_dir.to_path_buf()
}

fn tauri_config_to_bundle_settings(
  config: crate::helpers::config::BundleConfig,
  updater_config: crate::helpers::config::UpdaterConfig,
) -> crate::Result<BundleSettings> {
  Ok(BundleSettings {
    identifier: config.identifier,
    icon: config.icon,
    resources: config.resources,
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
      depends: config.deb.depends,
      use_bootstrapper: Some(config.deb.use_bootstrapper),
    },
    macos: MacOsSettings {
      frameworks: config.macos.frameworks,
      minimum_system_version: config.macos.minimum_system_version,
      license: config.macos.license,
      use_bootstrapper: Some(config.macos.use_bootstrapper),
      exception_domain: config.macos.exception_domain,
      signing_identity: config.macos.signing_identity,
      entitlements: config.macos.entitlements,
    },
    #[cfg(windows)]
    windows: WindowsSettings {
      timestamp_url: config.windows.timestamp_url,
      digest_algorithm: config.windows.digest_algorithm,
      certificate_thumbprint: config.windows.certificate_thumbprint,
    },
    updater: Some(UpdaterSettings {
      active: updater_config.active,
      // we set it to true by default we shouldn't have to use
      // unwrap_or as we have a default value but used to prevent any failing
      dialog: updater_config.dialog.unwrap_or(true),
      pubkey: updater_config.pubkey,
      endpoints: updater_config.endpoints,
    }),
    ..Default::default()
  })
}
