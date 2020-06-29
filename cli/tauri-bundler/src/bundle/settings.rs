use super::category::AppCategory;
use crate::bundle::common;
use crate::bundle::platform::target_triple;

use clap::ArgMatches;
use glob;
use serde::Deserialize;
use target_build_utils::TargetInfo;
use toml;
use walkdir;

use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The type of the package we're bundling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageType {
  /// The macOS bundle (.app).
  OsxBundle,
  /// The iOS app bundle.
  IosBundle,
  /// The Windows bundle (.msi).
  #[cfg(target_os = "windows")]
  WindowsMsi,
  /// The Linux Debian package bundle (.deb).
  Deb,
  /// The Linux RPM bundle (.rpm).
  Rpm,
  /// The Linux AppImage bundle (.AppImage).
  AppImage,
  /// The macOS DMG bundle (.dmg).
  Dmg,
}

impl PackageType {
  /// Maps a short name to a PackageType.
  /// Possible values are "deb", "ios", "msi", "osx", "rpm", "appimage", "dmg".
  pub fn from_short_name(name: &str) -> Option<PackageType> {
    // Other types we may eventually want to support: apk.
    match name {
      "deb" => Some(PackageType::Deb),
      "ios" => Some(PackageType::IosBundle),
      #[cfg(target_os = "windows")]
      "msi" => Some(PackageType::WindowsMsi),
      "osx" => Some(PackageType::OsxBundle),
      "rpm" => Some(PackageType::Rpm),
      "appimage" => Some(PackageType::AppImage),
      "dmg" => Some(PackageType::Dmg),
      _ => None,
    }
  }

  /// Gets the short name of this PackageType.
  pub fn short_name(&self) -> &'static str {
    match *self {
      PackageType::Deb => "deb",
      PackageType::IosBundle => "ios",
      #[cfg(target_os = "windows")]
      PackageType::WindowsMsi => "msi",
      PackageType::OsxBundle => "osx",
      PackageType::Rpm => "rpm",
      PackageType::AppImage => "appimage",
      PackageType::Dmg => "dmg",
    }
  }

  /// Gets the list of the possible package types.
  pub fn all() -> &'static [PackageType] {
    ALL_PACKAGE_TYPES
  }
}

const ALL_PACKAGE_TYPES: &[PackageType] = &[
  PackageType::Deb,
  PackageType::IosBundle,
  #[cfg(target_os = "windows")]
  PackageType::WindowsMsi,
  PackageType::OsxBundle,
  PackageType::Rpm,
  PackageType::Dmg,
  PackageType::AppImage,
];

/// The bundle settings of the BuildArtifact we're bundling.
#[derive(Clone, Debug, Deserialize, Default)]
struct BundleSettings {
  // General settings:
  /// the name of the bundle.
  name: Option<String>,
  /// the app's identifier.
  identifier: Option<String>,
  /// the app's icon list.
  icon: Option<Vec<String>>,
  /// the app's version.
  version: Option<String>,
  /// the app's resources to bundle.
  ///
  /// each item can be a path to a file or a path to a folder.
  ///
  /// supports glob patterns.
  resources: Option<Vec<String>>,
  /// the app's copyright.
  copyright: Option<String>,
  /// the app's category.
  category: Option<AppCategory>,
  /// the app's short description.
  short_description: Option<String>,
  /// the app's long description.
  long_description: Option<String>,
  /// the app's script to run when unpackaging the bundle.
  script: Option<PathBuf>,
  // OS-specific settings:
  /// the list of debian dependencies.
  deb_depends: Option<Vec<String>>,
  /// whether we should use the bootstrap script on debian or not.
  ///
  /// this script goal is to allow your app to access environment variables e.g $PATH.
  ///
  /// without it, you can't run some applications installed by the user.
  deb_use_bootstrapper: Option<bool>,
  /// Mac OS X frameworks that need to be bundled with the app.
  ///
  /// Each string can either be the name of a framework (without the `.framework` extension, e.g. `"SDL2"`),
  /// in which case we will search for that framework in the standard install locations (`~/Library/Frameworks/`, `/Library/Frameworks/`, and `/Network/Library/Frameworks/`),
  /// or a path to a specific framework bundle (e.g. `./data/frameworks/SDL2.framework`).  Note that this setting just makes tauri-bundler copy the specified frameworks into the OS X app bundle
  /// (under `Foobar.app/Contents/Frameworks/`); you are still responsible for:
  ///
  /// - arranging for the compiled binary to link against those frameworks (e.g. by emitting lines like `cargo:rustc-link-lib=framework=SDL2` from your `build.rs` script)
  ///
  /// - embedding the correct rpath in your binary (e.g. by running `install_name_tool -add_rpath "@executable_path/../Frameworks" path/to/binary` after compiling)
  osx_frameworks: Option<Vec<String>>,
  /// A version string indicating the minimum Mac OS X version that the bundled app supports (e.g. `"10.11"`).
  /// If you are using this config field, you may also want have your `build.rs` script emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11`.
  osx_minimum_system_version: Option<String>,
  /// The path to the LICENSE file for macOS apps.
  /// Currently only used by the dmg bundle.
  osx_license: Option<String>,
  /// whether we should use the bootstrap script on macOS .app or not.
  ///
  /// this script goal is to allow your app to access environment variables e.g $PATH.
  ///
  /// without it, you can't run some applications installed by the user.
  osx_use_bootstrapper: Option<bool>,
  // Bundles for other binaries/examples:
  /// Configuration map for the possible [bin] apps to bundle.
  bin: Option<HashMap<String, BundleSettings>>,
  /// Configuration map for the possible example apps to bundle.
  example: Option<HashMap<String, BundleSettings>>,
  /// External binaries to add to the bundle.
  ///
  /// Note that each binary name will have the target platform's target triple appended,
  /// so if you're bundling the `sqlite3` app, the bundler will look for e.g.
  /// `sqlite3-x86_64-unknown-linux-gnu` on linux,
  /// and `sqlite3-x86_64-pc-windows-gnu.exe` on windows.
  ///
  /// The possible target triples can be seen by running `$ rustup target list`.
  external_bin: Option<Vec<String>>,
  /// The exception domain to use on the macOS .app bundle.
  ///
  /// This allows communication to the outside world e.g. a web server you're shipping.
  exception_domain: Option<String>,
}

/// The `metadata` section of the package configuration.
///
/// # Example Cargo.toml
/// ```
/// [package]
/// name = "..."
///
/// [package.metadata.bundle]
/// identifier = "..."
/// ...other properties from BundleSettings
/// ```
#[derive(Clone, Debug, Deserialize)]
struct MetadataSettings {
  /// the bundle settings of the package.
  bundle: Option<BundleSettings>,
}

/// The `package` section of the app configuration (read from Cargo.toml).
#[derive(Clone, Debug, Deserialize)]
struct PackageSettings {
  /// the package's name.
  name: String,
  /// the package's version.
  version: String,
  /// the package's description.
  description: String,
  /// the package's homepage.
  homepage: Option<String>,
  /// the package's authors.
  authors: Option<Vec<String>>,
  /// the package's metadata.
  metadata: Option<MetadataSettings>,
  /// the default binary to run.
  default_run: Option<String>,
}

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

#[derive(Clone, Debug)]
pub struct BundleBinary {
  name: String,
  path: Option<String>,
  main: bool,
}

impl BundleBinary {
  pub fn name(&self) -> &String {
    &self.name
  }
}

/// The Settings exposed by the module.
#[derive(Clone, Debug)]
pub struct Settings {
  /// the package settings.
  package: PackageSettings,
  /// the package types we're bundling.
  ///
  /// if not present, we'll use the PackageType list for the target OS.
  package_types: Option<Vec<PackageType>>,
  /// the target platform of the build
  target: Option<(String, TargetInfo)>,
  /// the features to use to build the app with `cargo build --features foo bar`.
  features: Option<Vec<String>>,
  /// the directory where the bundles will be placed.
  project_out_directory: PathBuf,
  /// whether we should build the app with release mode or not.
  is_release: bool,
  /// the bundle settings.
  bundle_settings: BundleSettings,
  /// the binaries to bundle.
  binaries: Vec<BundleBinary>,
}

impl CargoSettings {
  /// Try to load a set of CargoSettings from a "Cargo.toml" file in the specified directory.
  fn load(dir: &PathBuf) -> crate::Result<Self> {
    let toml_path = dir.join("Cargo.toml");
    let mut toml_str = String::new();
    let mut toml_file = File::open(toml_path)?;
    toml_file.read_to_string(&mut toml_str)?;
    toml::from_str(&toml_str).map_err(|e| e.into())
  }
}

impl Settings {
  /// Builds a Settings from the CLI args.
  ///
  /// Package settings will be read from Cargo.toml.
  ///
  /// Bundle settings will be read from from $TAURI_DIR/tauri.conf.json if it exists and fallback to Cargo.toml's [package.metadata.bundle].
  pub fn new(current_dir: PathBuf, matches: &ArgMatches<'_>) -> crate::Result<Self> {
    let package_types = match matches.values_of("format") {
      Some(names) => {
        let mut types = vec![];
        for name in names {
          match PackageType::from_short_name(name) {
            Some(package_type) => {
              types.push(package_type);
            }
            None => {
              return Err(crate::Error::GenericError(format!(
                "Unsupported bundle format: {}",
                name
              )));
            }
          }
        }
        Some(types)
      }
      None => None,
    };
    let is_release = matches.is_present("release");
    let target = match matches.value_of("target") {
      Some(triple) => Some((triple.to_string(), TargetInfo::from_str(triple)?)),
      None => None,
    };
    let features = if matches.is_present("features") {
      Some(
        matches
          .values_of("features")
          .expect("Couldn't get the features")
          .map(|s| s.to_string())
          .collect(),
      )
    } else {
      None
    };
    let cargo_settings = CargoSettings::load(&current_dir)?;
    let tauri_config = super::tauri_config::get();

    let package = match cargo_settings.package {
      Some(package_info) => package_info,
      None => {
        return Err(crate::Error::GenericError(
          "No package info in the config file".to_owned(),
        ))
      }
    };
    let workspace_dir = Settings::get_workspace_dir(&current_dir);
    let target_dir = Settings::get_target_dir(&workspace_dir, &target, is_release);
    let bundle_settings = match tauri_config {
      Ok(config) => merge_settings(BundleSettings::default(), config.tauri.bundle),
      Err(e) => {
        let error_message = e.to_string();
        if !error_message.contains("No such file or directory") {
          return Err(crate::Error::GenericError(format!(
            "Failed to read tauri config: {}",
            error_message
          )));
        }
        if let Some(bundle_settings) = package
          .metadata
          .as_ref()
          .and_then(|metadata| metadata.bundle.as_ref())
        {
          bundle_settings.clone()
        } else {
          return Err(crate::Error::GenericError(
            "No 'bundle' key in the tauri.conf.json".to_owned(),
          ));
        }
      }
    };

    let mut binaries: Vec<BundleBinary> = vec![];
    if let Some(bin) = cargo_settings.bin {
      let default_run = package.default_run.clone().unwrap_or("".to_string());
      for binary in bin {
        binaries.push(BundleBinary {
          name: binary.name.clone(),
          path: binary.path,
          main: binary.name.as_str() == package.name || binary.name.as_str() == default_run,
        })
      }
    }

    let mut bins_path = PathBuf::from(current_dir);
    bins_path.push("src/bin");
    if let Ok(fs_bins) = std::fs::read_dir(bins_path) {
      for entry in fs_bins {
        let path = entry?.path();
        if let Some(name) = path.file_stem() {
          if !binaries.iter().any(|bin| {
            bin.name.as_str() == name
              || path.ends_with(bin.path.as_ref().unwrap_or(&"".to_string()))
          }) {
            binaries.push(BundleBinary {
              name: name.to_string_lossy().to_string(),
              path: None,
              main: false,
            })
          }
        }
      }
    }

    if let Some(default_run) = package.default_run.as_ref() {
      if !binaries.iter().any(|bin| bin.name.as_str() == default_run) {
        binaries.push(BundleBinary {
          name: default_run.to_string(),
          path: None,
          main: true,
        })
      }
    }

    if binaries.len() == 1 {
      binaries.get_mut(0).and_then(|bin| {
        bin.main = true;
        Some(bin)
      });
    }

    let bundle_settings = parse_external_bin(bundle_settings)?;

    Ok(Settings {
      package,
      package_types,
      target,
      features,
      is_release,
      project_out_directory: target_dir,
      binaries,
      bundle_settings,
    })
  }

  /// This function determines where 'target' dir is and suffixes it with 'release' or 'debug'
  /// to determine where the compiled binary will be located.
  fn get_target_dir(
    project_root_dir: &PathBuf,
    target: &Option<(String, TargetInfo)>,
    is_release: bool,
  ) -> PathBuf {
    let mut path = project_root_dir.join("target");
    if let &Some((ref triple, _)) = target {
      path.push(triple);
    }
    path.push(if is_release { "release" } else { "debug" });
    path
  }

  /// Walks up the file system, looking for a Cargo.toml file
  /// If one is found before reaching the root, then the current_dir's package belongs to that parent workspace if it's listed on [workspace.members].
  ///
  /// If this package is part of a workspace, returns the path to the workspace directory
  /// Otherwise returns the current directory.
  pub fn get_workspace_dir(current_dir: &PathBuf) -> PathBuf {
    let mut dir = current_dir.clone();
    let project_name = CargoSettings::load(&dir).unwrap().package.unwrap().name;

    while dir.pop() {
      match CargoSettings::load(&dir) {
        Ok(cargo_settings) => match cargo_settings.workspace {
          Some(workspace_settings) => {
            if workspace_settings.members.is_some()
              && workspace_settings
                .members
                .expect("Couldn't get members")
                .iter()
                .any(|member| member.as_str() == project_name)
            {
              return dir;
            }
          }
          None => {}
        },
        Err(_) => {}
      }
    }

    // Nothing found walking up the file system, return the starting directory
    current_dir.clone()
  }

  /// Returns the directory where the bundle should be placed.
  pub fn project_out_directory(&self) -> &Path {
    &self.project_out_directory
  }

  /// Returns the architecture for the binary being bundled (e.g. "arm", "x86" or "x86_64").
  pub fn binary_arch(&self) -> &str {
    if let Some((_, ref info)) = self.target {
      info.target_arch()
    } else {
      std::env::consts::ARCH
    }
  }

  /// Returns the file name of the binary being bundled.
  pub fn main_binary_name(&self) -> &str {
    self
      .binaries
      .iter()
      .find(|bin| bin.main)
      .expect("failed to find main binary")
      .name
      .as_str()
  }

  /// Returns the path to the specified binary.
  pub fn binary_path(&self, binary: &BundleBinary) -> PathBuf {
    let mut path = self.project_out_directory.clone();
    path.push(binary.name.clone());
    path
  }

  pub fn binaries(&self) -> &Vec<BundleBinary> {
    &self.binaries
  }

  /// If a list of package types was specified by the command-line, returns
  /// that list filtered by the current target OS available targets.
  ///
  /// If a target triple was specified by the
  /// command-line, returns the native package type(s) for that target.
  ///
  /// Otherwise returns the native package type(s) for the host platform.
  ///
  /// Fails if the host/target's native package type is not supported.
  pub fn package_types(&self) -> crate::Result<Vec<PackageType>> {
    let target_os = if let Some((_, ref info)) = self.target {
      info.target_os()
    } else {
      std::env::consts::OS
    };
    let platform_types = match target_os {
      "macos" => vec![PackageType::OsxBundle, PackageType::Dmg],
      "ios" => vec![PackageType::IosBundle],
      "linux" => vec![PackageType::Deb, PackageType::AppImage],
      #[cfg(target_os = "windows")]
      "windows" => vec![PackageType::WindowsMsi],
      os => {
        return Err(crate::Error::GenericError(format!(
          "Native {} bundles not yet supported.",
          os
        )))
      }
    };
    if let Some(package_types) = &self.package_types {
      let mut types = vec![];
      for package_type in package_types {
        let package_type = *package_type;
        if platform_types
          .clone()
          .into_iter()
          .any(|t| t == package_type)
        {
          types.push(package_type);
        }
      }
      Ok(types)
    } else {
      Ok(platform_types)
    }
  }

  /// If the bundle is being cross-compiled, returns the target triple string
  /// (e.g. `"x86_64-apple-darwin"`). If the bundle is targeting the host
  /// environment, returns `None`.
  pub fn target_triple(&self) -> Option<&str> {
    match self.target {
      Some((ref triple, _)) => Some(triple.as_str()),
      None => None,
    }
  }

  /// Returns the features that is being built.
  pub fn build_features(&self) -> Option<Vec<String>> {
    self.features.to_owned()
  }

  /// Returns true if the bundle is being compiled in release mode, false if
  /// it's being compiled in debug mode.
  pub fn is_release_build(&self) -> bool {
    self.is_release
  }

  /// Returns the bundle name, which is either package.metadata.bundle.name or package.name
  pub fn bundle_name(&self) -> &str {
    self
      .bundle_settings
      .name
      .as_ref()
      .unwrap_or(&self.package.name)
  }

  /// Returns the bundle's identifier
  pub fn bundle_identifier(&self) -> &str {
    self
      .bundle_settings
      .identifier
      .as_ref()
      .map(String::as_str)
      .unwrap_or("")
  }

  /// Returns an iterator over the icon files to be used for this bundle.
  pub fn icon_files(&self) -> ResourcePaths<'_> {
    match self.bundle_settings.icon {
      Some(ref paths) => ResourcePaths::new(paths.as_slice(), false),
      None => ResourcePaths::new(&[], false),
    }
  }

  /// Returns an iterator over the resource files to be included in this
  /// bundle.
  pub fn resource_files(&self) -> ResourcePaths<'_> {
    match self.bundle_settings.resources {
      Some(ref paths) => ResourcePaths::new(paths.as_slice(), true),
      None => ResourcePaths::new(&[], true),
    }
  }

  /// Returns an iterator over the external binaries to be included in this
  /// bundle.
  pub fn external_binaries(&self) -> ResourcePaths<'_> {
    match self.bundle_settings.external_bin {
      Some(ref paths) => ResourcePaths::new(paths.as_slice(), true),
      None => ResourcePaths::new(&[], true),
    }
  }

  /// Returns the OSX exception domain.
  pub fn exception_domain(&self) -> Option<&String> {
    self.bundle_settings.exception_domain.as_ref()
  }

  /// Copies external binaries to a path.
  pub fn copy_binaries(&self, path: &Path) -> crate::Result<()> {
    for src in self.external_binaries() {
      let src = src?;
      let dest = path.join(
        src
          .file_name()
          .expect("failed to extract external binary filename"),
      );
      common::copy_file(&src, &dest)?;
    }
    Ok(())
  }

  /// Copies resources to a path.
  pub fn copy_resources(&self, path: &Path) -> crate::Result<()> {
    for src in self.resource_files() {
      let src = src?;
      let dest = path.join(common::resource_relpath(&src));
      common::copy_file(&src, &dest)?;
    }
    Ok(())
  }

  /// Returns the version string of the bundle, which is either package.metadata.version or package.version.
  pub fn version_string(&self) -> &str {
    self
      .bundle_settings
      .version
      .as_ref()
      .unwrap_or(&self.package.version)
  }

  /// Returns the copyright text.
  pub fn copyright_string(&self) -> Option<&str> {
    self.bundle_settings.copyright.as_ref().map(String::as_str)
  }

  /// Returns the list of authors name.
  pub fn author_names(&self) -> &[String] {
    match self.package.authors {
      Some(ref names) => names.as_slice(),
      None => &[],
    }
  }

  /// Returns the authors as a comma-separated string.
  pub fn authors_comma_separated(&self) -> Option<String> {
    let names = self.author_names();
    if names.is_empty() {
      None
    } else {
      Some(names.join(", "))
    }
  }

  /// Returns the package's homepage URL, defaulting to "" if not defined.
  pub fn homepage_url(&self) -> &str {
    &self
      .package
      .homepage
      .as_ref()
      .map(String::as_str)
      .unwrap_or("")
  }

  /// Returns the app's category.
  pub fn app_category(&self) -> Option<AppCategory> {
    self.bundle_settings.category
  }

  /// Returns the app's short description.
  pub fn short_description(&self) -> &str {
    self
      .bundle_settings
      .short_description
      .as_ref()
      .unwrap_or(&self.package.description)
  }

  /// Returns the app's long description.
  pub fn long_description(&self) -> Option<&str> {
    self
      .bundle_settings
      .long_description
      .as_ref()
      .map(String::as_str)
  }

  /// Returns the dependencies of the debian bundle.
  pub fn debian_dependencies(&self) -> &[String] {
    match self.bundle_settings.deb_depends {
      Some(ref dependencies) => dependencies.as_slice(),
      None => &[],
    }
  }

  /// Returns whether the debian bundle should use the bootstrap script or not.
  pub fn debian_use_bootstrapper(&self) -> bool {
    self.bundle_settings.deb_use_bootstrapper.unwrap_or(false)
  }

  /// Returns the frameworks to bundle with the macOS .app
  pub fn osx_frameworks(&self) -> &[String] {
    match self.bundle_settings.osx_frameworks {
      Some(ref frameworks) => frameworks.as_slice(),
      None => &[],
    }
  }

  /// Returns the minimum system version of the macOS bundle.
  pub fn osx_minimum_system_version(&self) -> Option<&str> {
    self
      .bundle_settings
      .osx_minimum_system_version
      .as_ref()
      .map(String::as_str)
  }

  /// Returns the path to the DMG bundle license.
  pub fn osx_license(&self) -> Option<&str> {
    self
      .bundle_settings
      .osx_license
      .as_ref()
      .map(String::as_str)
  }

  /// Returns whether the macOS .app bundle should use the bootstrap script or not.
  pub fn osx_use_bootstrapper(&self) -> bool {
    self.bundle_settings.osx_use_bootstrapper.unwrap_or(false)
  }
}

/// Parses the external binaries to bundle, adding the target triple suffix to each of them.
fn parse_external_bin(bundle_settings: BundleSettings) -> crate::Result<BundleSettings> {
  let target_triple = target_triple()?;
  let mut win_paths = Vec::new();
  let external_bin = match bundle_settings.external_bin {
    Some(paths) => {
      for curr_path in paths.iter() {
        win_paths.push(format!(
          "{}-{}{}",
          curr_path,
          target_triple,
          if cfg!(windows) { ".exe" } else { "" }
        ));
      }
      Some(win_paths)
    }
    None => Some(vec![]),
  };

  Ok(BundleSettings {
    external_bin,
    ..bundle_settings
  })
}

/// Returns the first Option with a value, or None if both are None.
fn options_value<T>(first: Option<T>, second: Option<T>) -> Option<T> {
  if let Some(_) = first {
    first
  } else {
    second
  }
}

/// Merges the bundle settings from Cargo.toml and tauri.conf.json
fn merge_settings(
  bundle_settings: BundleSettings,
  config: crate::bundle::tauri_config::BundleConfig,
) -> BundleSettings {
  BundleSettings {
    name: options_value(config.name, bundle_settings.name),
    identifier: options_value(config.identifier, bundle_settings.identifier),
    icon: options_value(config.icon, bundle_settings.icon),
    version: options_value(config.version, bundle_settings.version),
    resources: options_value(config.resources, bundle_settings.resources),
    copyright: options_value(config.copyright, bundle_settings.copyright),
    category: options_value(config.category, bundle_settings.category),
    short_description: options_value(config.short_description, bundle_settings.short_description),
    long_description: options_value(config.long_description, bundle_settings.long_description),
    script: options_value(config.script, bundle_settings.script),
    deb_depends: options_value(config.deb.depends, bundle_settings.deb_depends),
    deb_use_bootstrapper: Some(config.deb.use_bootstrapper),
    osx_frameworks: options_value(config.osx.frameworks, bundle_settings.osx_frameworks),
    osx_minimum_system_version: options_value(
      config.osx.minimum_system_version,
      bundle_settings.osx_minimum_system_version,
    ),
    osx_license: options_value(config.osx.license, bundle_settings.osx_license),
    osx_use_bootstrapper: Some(config.osx.use_bootstrapper),
    external_bin: options_value(config.external_bin, bundle_settings.external_bin),
    exception_domain: options_value(
      config.osx.exception_domain,
      bundle_settings.exception_domain,
    ),
    ..bundle_settings
  }
}

/// A helper to iterate through resources.
pub struct ResourcePaths<'a> {
  /// the patterns to iterate.
  pattern_iter: std::slice::Iter<'a, String>,
  /// the glob iterator if the path from the current iteration is a glob pattern.
  glob_iter: Option<glob::Paths>,
  /// the walkdir iterator if the path from the current iteration is a directory.
  walk_iter: Option<walkdir::IntoIter>,
  /// whether the resource paths allows directories or not.
  allow_walk: bool,
  /// the pattern of the current iteration.
  current_pattern: Option<String>,
  /// whether the current pattern is valid or not.
  current_pattern_is_valid: bool,
}

impl<'a> ResourcePaths<'a> {
  /// Creates a new ResourcePaths from a slice of patterns to iterate
  fn new(patterns: &'a [String], allow_walk: bool) -> ResourcePaths<'a> {
    ResourcePaths {
      pattern_iter: patterns.iter(),
      glob_iter: None,
      walk_iter: None,
      allow_walk,
      current_pattern: None,
      current_pattern_is_valid: false,
    }
  }
}

impl<'a> Iterator for ResourcePaths<'a> {
  type Item = crate::Result<PathBuf>;

  fn next(&mut self) -> Option<crate::Result<PathBuf>> {
    loop {
      if let Some(ref mut walk_entries) = self.walk_iter {
        if let Some(entry) = walk_entries.next() {
          let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(Err(crate::Error::from(error))),
          };
          let path = entry.path();
          if path.is_dir() {
            continue;
          }
          self.current_pattern_is_valid = true;
          return Some(Ok(path.to_path_buf()));
        }
      }
      self.walk_iter = None;
      if let Some(ref mut glob_paths) = self.glob_iter {
        if let Some(glob_result) = glob_paths.next() {
          let path = match glob_result {
            Ok(path) => path,
            Err(error) => return Some(Err(crate::Error::from(error))),
          };
          if path.is_dir() {
            if self.allow_walk {
              let walk = walkdir::WalkDir::new(path);
              self.walk_iter = Some(walk.into_iter());
              continue;
            } else {
              let msg = format!("{:?} is a directory", path);
              return Some(Err(crate::Error::GenericError(msg)));
            }
          }
          self.current_pattern_is_valid = true;
          return Some(Ok(path));
        } else {
          if let Some(current_path) = &self.current_pattern {
            if !self.current_pattern_is_valid {
              return Some(Err(crate::Error::GenericError(format!(
                "Path matching '{}' not found",
                current_path
              ))));
            }
          }
        }
      }
      self.glob_iter = None;
      if let Some(pattern) = self.pattern_iter.next() {
        self.current_pattern = Some(pattern.to_string());
        self.current_pattern_is_valid = false;
        let glob = match glob::glob(pattern) {
          Ok(glob) => glob,
          Err(error) => return Some(Err(crate::Error::from(error))),
        };
        self.glob_iter = Some(glob);
        continue;
      }
      return None;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{AppCategory, BundleSettings, CargoSettings};
  use toml;

  #[test]
  fn parse_cargo_toml() {
    let toml_str = "\
                    [package]\n\
                    name = \"example\"\n\
                    version = \"0.1.0\"\n\
                    authors = [\"Jane Doe\"]\n\
                    license = \"MIT\"\n\
                    description = \"An example application.\"\n\
                    build = \"build.rs\"\n\
                    \n\
                    [package.metadata.bundle]\n\
                    name = \"Example Application\"\n\
                    identifier = \"com.example.app\"\n\
                    resources = [\"data\", \"foo/bar\"]\n\
                    category = \"Puzzle Game\"\n\
                    long_description = \"\"\"\n\
                    This is an example of a\n\
                    simple application.\n\
                    \"\"\"\n\
                    \n\
                    [dependencies]\n\
                    rand = \"0.4\"\n";
    let cargo_settings: CargoSettings = toml::from_str(toml_str).unwrap();
    let package = cargo_settings.package.expect("Couldn't get package");
    assert_eq!(package.name, "example");
    assert_eq!(package.version, "0.1.0");
    assert_eq!(package.description, "An example application.");
    assert_eq!(package.homepage, None);
    assert_eq!(package.authors, Some(vec!["Jane Doe".to_string()]));
    assert!(package.metadata.is_some());
    let metadata = package
      .metadata
      .as_ref()
      .expect("Failed to get metadata ref");
    assert!(metadata.bundle.is_some());
    let bundle = metadata.bundle.as_ref().expect("Failed to get bundle ref");
    assert_eq!(bundle.name, Some("Example Application".to_string()));
    assert_eq!(bundle.identifier, Some("com.example.app".to_string()));
    assert_eq!(bundle.icon, None);
    assert_eq!(bundle.version, None);
    assert_eq!(
      bundle.resources,
      Some(vec!["data".to_string(), "foo/bar".to_string()])
    );
    assert_eq!(bundle.category, Some(AppCategory::PuzzleGame));
    assert_eq!(
      bundle.long_description,
      Some(
        "This is an example of a\n\
         simple application.\n"
          .to_string()
      )
    );
  }

  #[test]
  fn parse_bin_and_example_bundles() {
    let toml_str = "\
            [package]\n\
            name = \"example\"\n\
            version = \"0.1.0\"\n\
            description = \"An example application.\"\n\
            \n\
            [package.metadata.bundle.bin.foo]\n\
            name = \"Foo App\"\n\
            \n\
            [package.metadata.bundle.bin.bar]\n\
            name = \"Bar App\"\n\
            \n\
            [package.metadata.bundle.example.baz]\n\
            name = \"Baz Example\"\n\
            \n\
            [[bin]]\n\
            name = \"foo\"\n
            \n\
            [[bin]]\n\
            name = \"bar\"\n
            \n\
            [[example]]\n\
            name = \"baz\"\n";
    let cargo_settings: CargoSettings = toml::from_str(toml_str).expect("Failed to read from toml");
    assert!(cargo_settings.package.is_some());
    let package = cargo_settings
      .package
      .as_ref()
      .expect("Failed to get package ref");
    assert!(package.metadata.is_some());
    let metadata = package
      .metadata
      .as_ref()
      .expect("Failed to get metadata ref");
    assert!(metadata.bundle.is_some());
    let bundle = metadata.bundle.as_ref().expect("Failed to get bundle ref");
    assert!(bundle.example.is_some());

    let bins = bundle.bin.as_ref().expect("Failed to get bin ref");
    assert!(bins.contains_key("foo"));
    let foo: &BundleSettings = bins.get("foo").expect("Failed to get foo bundle settings");
    assert_eq!(foo.name, Some("Foo App".to_string()));
    assert!(bins.contains_key("bar"));
    let bar: &BundleSettings = bins.get("bar").expect("Failed to get bar bundle settings");
    assert_eq!(bar.name, Some("Bar App".to_string()));

    let examples = bundle.example.as_ref().expect("Failed to get example ref");
    assert!(examples.contains_key("baz"));
    let baz: &BundleSettings = examples
      .get("baz")
      .expect("Failed to get baz bundle settings");
    assert_eq!(baz.name, Some("Baz Example".to_string()));
  }
}
