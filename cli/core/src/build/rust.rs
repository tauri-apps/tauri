use std::{fs::File, io::Read, path::PathBuf, process::Command, str::FromStr};

use serde::Deserialize;

use crate::helpers::{app_paths::tauri_dir, config::Config};
use tauri_bundler::{AppCategory, BundleBinary, BundleSettings, PackageSettings};

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

/// The Cargo settings (Cargo.toml root descriptor).
#[derive(Clone, Debug, Deserialize)]
struct CargoSettings {
  /// the package settings.
  ///
  /// it's optional because ancestor workspace Cargo.toml files may not have package info.
  package: Option<PackageSettings>,
  /// the workspace settings.
  ///
  /// it's present if the read Cargo.toml belongs to a workspace root.
  workspace: Option<WorkspaceSettings>,
  /// the binary targets configuration.
  bin: Option<Vec<BinarySettings>>,
}

impl CargoSettings {
  /// Try to load a set of CargoSettings from a "Cargo.toml" file in the specified directory.
  fn load(dir: &PathBuf) -> crate::Result<Self> {
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
  let mut args = vec!["build", "--features=embedded-server"];

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

pub struct BundlerSettings {
  pub package_settings: PackageSettings,
  pub bundle_settings: BundleSettings,
  pub binaries: Vec<BundleBinary>,
  pub out_dir: PathBuf,
}

pub fn get_bundler_settings(config: &Config, debug: bool) -> crate::Result<BundlerSettings> {
  let tauri_dir = tauri_dir();
  let cargo_settings = CargoSettings::load(&tauri_dir)?;

  let package = match cargo_settings.package {
    Some(package_info) => package_info,
    None => {
      return Err(anyhow::anyhow!(
        "No package info in the config file".to_owned(),
      ))
    }
  };
  let workspace_dir = get_workspace_dir(&tauri_dir);
  let target_dir = get_target_dir(&workspace_dir, None, !debug)?;
  let bundle_settings = tauri_config_to_bundle_settings(config.tauri.bundle.clone())?;

  let mut binaries: Vec<BundleBinary> = vec![];
  if let Some(bin) = cargo_settings.bin {
    let default_run = package
      .default_run
      .clone()
      .unwrap_or_else(|| "".to_string());
    for binary in bin {
      binaries.push(
        BundleBinary::new(
          binary.name.clone(),
          binary.name.as_str() == package.name || binary.name.as_str() == default_run,
        )
        .set_src_path(binary.path),
      )
    }
  }

  let mut bins_path = tauri_dir;
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

  if let Some(default_run) = package.default_run.as_ref() {
    if !binaries.iter().any(|bin| bin.name() == default_run) {
      binaries.push(BundleBinary::new(default_run.to_string(), true));
    }
  }

  if binaries.len() == 1 {
    binaries.get_mut(0).unwrap().set_main(true);
  }

  Ok(BundlerSettings {
    package_settings: package,
    bundle_settings,
    binaries,
    out_dir: target_dir,
  })
}

/// This function determines where 'target' dir is and suffixes it with 'release' or 'debug'
/// to determine where the compiled binary will be located.
fn get_target_dir(
  project_root_dir: &PathBuf,
  target: Option<String>,
  is_release: bool,
) -> crate::Result<PathBuf> {
  let mut path: PathBuf = match std::env::var_os("CARGO_TARGET_DIR") {
    Some(target_dir) => target_dir.into(),
    None => {
      let mut root_dir = project_root_dir.clone();
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
pub fn get_workspace_dir(current_dir: &PathBuf) -> PathBuf {
  let mut dir = current_dir.clone();
  let project_path = current_dir.clone();

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
  current_dir.clone()
}

fn tauri_config_to_bundle_settings(
  config: crate::helpers::config::BundleConfig,
) -> crate::Result<BundleSettings> {
  Ok(BundleSettings {
    name: config.name,
    identifier: config.identifier,
    icon: config.icon,
    version: config.version,
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
    script: config.script,
    deb_depends: config.deb.depends,
    deb_use_bootstrapper: Some(config.deb.use_bootstrapper),
    osx_frameworks: config.osx.frameworks,
    osx_minimum_system_version: config.osx.minimum_system_version,
    osx_license: config.osx.license,
    osx_use_bootstrapper: Some(config.osx.use_bootstrapper),
    external_bin: config.external_bin,
    exception_domain: config.osx.exception_domain,
    ..Default::default()
  })
}
