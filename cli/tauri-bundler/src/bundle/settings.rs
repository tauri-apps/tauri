use super::category::AppCategory;
use crate::bundle::{common, platform::target_triple};

use serde::Deserialize;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

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
  /// The Updater bundle.
  Updater,
}

impl PackageType {
  /// Maps a short name to a PackageType.
  /// Possible values are "deb", "ios", "msi", "osx", "rpm", "appimage", "dmg", "updater".
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
      "updater" => Some(PackageType::Updater),
      _ => None,
    }
  }

  /// Gets the short name of this PackageType.
  #[allow(clippy::trivially_copy_pass_by_ref)]
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
      PackageType::Updater => "updater",
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
  PackageType::Updater,
];

/// The package settings.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageSettings {
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

/// The updater settings.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdaterSettings {
  /// Whether the updater is active or not.
  pub active: bool,
  /// The updater endpoints.
  pub endpoints: Option<Vec<String>>,
  /// Optional pubkey.
  pub pubkey: Option<String>,
  /// Display built-in dialog or use event system if disabled.
  pub dialog: bool,
}

/// The bundle settings of the BuildArtifact we're bundling.
#[derive(Clone, Debug, Deserialize, Default)]
pub struct BundleSettings {
  // General settings:
  /// the name of the bundle.
  pub name: Option<String>,
  /// the app's identifier.
  pub identifier: Option<String>,
  /// the app's icon list.
  pub icon: Option<Vec<String>>,
  /// the app's version.
  pub version: Option<String>,
  /// the app's resources to bundle.
  ///
  /// each item can be a path to a file or a path to a folder.
  ///
  /// supports glob patterns.
  pub resources: Option<Vec<String>>,
  /// the app's copyright.
  pub copyright: Option<String>,
  /// the app's category.
  pub category: Option<AppCategory>,
  /// the app's short description.
  pub short_description: Option<String>,
  /// the app's long description.
  pub long_description: Option<String>,
  /// the app's script to run when unpackaging the bundle.
  pub script: Option<PathBuf>,
  // OS-specific settings:
  /// the list of debian dependencies.
  pub deb_depends: Option<Vec<String>>,
  /// whether we should use the bootstrap script on debian or not.
  ///
  /// this script goal is to allow your app to access environment variables e.g $PATH.
  ///
  /// without it, you can't run some applications installed by the user.
  pub deb_use_bootstrapper: Option<bool>,
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
  pub osx_frameworks: Option<Vec<String>>,
  /// A version string indicating the minimum Mac OS X version that the bundled app supports (e.g. `"10.11"`).
  /// If you are using this config field, you may also want have your `build.rs` script emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11`.
  pub osx_minimum_system_version: Option<String>,
  /// The path to the LICENSE file for macOS apps.
  /// Currently only used by the dmg bundle.
  pub osx_license: Option<String>,
  /// whether we should use the bootstrap script on macOS .app or not.
  ///
  /// this script goal is to allow your app to access environment variables e.g $PATH.
  ///
  /// without it, you can't run some applications installed by the user.
  pub osx_use_bootstrapper: Option<bool>,
  // Bundles for other binaries/examples:
  /// Configuration map for the possible [bin] apps to bundle.
  pub bin: Option<HashMap<String, BundleSettings>>,
  /// Configuration map for the possible example apps to bundle.
  pub example: Option<HashMap<String, BundleSettings>>,
  /// External binaries to add to the bundle.
  ///
  /// Note that each binary name will have the target platform's target triple appended,
  /// so if you're bundling the `sqlite3` app, the bundler will look for e.g.
  /// `sqlite3-x86_64-unknown-linux-gnu` on linux,
  /// and `sqlite3-x86_64-pc-windows-gnu.exe` on windows.
  ///
  /// The possible target triples can be seen by running `$ rustup target list`.
  pub external_bin: Option<Vec<String>>,
  /// The exception domain to use on the macOS .app bundle.
  ///
  /// This allows communication to the outside world e.g. a web server you're shipping.
  pub exception_domain: Option<String>,
  // Updater configuration
  pub updater: Option<UpdaterSettings>,
}

#[derive(Clone, Debug)]
pub struct BundleBinary {
  name: String,
  src_path: Option<String>,
  main: bool,
}

impl BundleBinary {
  pub fn new(name: String, main: bool) -> Self {
    Self {
      name: if cfg!(windows) {
        format!("{}.exe", name)
      } else {
        name
      },
      src_path: None,
      main,
    }
  }

  pub fn set_src_path(mut self, src_path: Option<String>) -> Self {
    self.src_path = src_path;
    self
  }

  pub fn set_main(&mut self, main: bool) {
    self.main = main;
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  #[cfg(windows)]
  pub fn main(&self) -> bool {
    self.main
  }

  pub fn src_path(&self) -> &Option<String> {
    &self.src_path
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
  /// the directory where the bundles will be placed.
  project_out_directory: PathBuf,
  /// whether or not to enable verbose logging
  is_verbose: bool,
  /// the bundle settings.
  bundle_settings: BundleSettings,
  /// the binaries to bundle.
  binaries: Vec<BundleBinary>,
}

#[derive(Default)]
pub struct SettingsBuilder {
  project_out_directory: Option<PathBuf>,
  verbose: bool,
  package_types: Option<Vec<PackageType>>,
  package_settings: Option<PackageSettings>,
  updater_settings: Option<UpdaterSettings>,
  bundle_settings: BundleSettings,
  binaries: Vec<BundleBinary>,
}

impl SettingsBuilder {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn project_out_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
    self
      .project_out_directory
      .replace(path.as_ref().to_path_buf());
    self
  }

  pub fn verbose(mut self) -> Self {
    self.verbose = true;
    self
  }

  pub fn package_types(mut self, package_types: Vec<PackageType>) -> Self {
    self.package_types = Some(package_types);
    self
  }

  pub fn package_settings(mut self, settings: PackageSettings) -> Self {
    self.package_settings.replace(settings);
    self
  }

  pub fn bundle_settings(mut self, settings: BundleSettings) -> Self {
    self.bundle_settings = settings;
    self
  }

  pub fn updater_settings(mut self, settings: UpdaterSettings) -> Self {
    self.updater_settings = Some(settings);
    self
  }

  pub fn binaries(mut self, binaries: Vec<BundleBinary>) -> Self {
    self.binaries = binaries;
    self
  }

  /// Builds a Settings from the CLI args.
  ///
  /// Package settings will be read from Cargo.toml.
  ///
  /// Bundle settings will be read from from $TAURI_DIR/tauri.conf.json if it exists and fallback to Cargo.toml's [package.metadata.bundle].
  pub fn build(self) -> crate::Result<Settings> {
    let bundle_settings = parse_external_bin(self.bundle_settings)?;

    Ok(Settings {
      package: self.package_settings.expect("package settings is required"),
      package_types: self.package_types,
      is_verbose: self.verbose,
      project_out_directory: self
        .project_out_directory
        .expect("out directory is required"),
      binaries: self.binaries,
      bundle_settings,
    })
  }
}

impl Settings {
  /// Returns the directory where the bundle should be placed.
  pub fn project_out_directory(&self) -> &Path {
    &self.project_out_directory
  }

  /// Returns the architecture for the binary being bundled (e.g. "arm", "x86" or "x86_64").
  pub fn binary_arch(&self) -> &str {
    std::env::consts::ARCH
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
    path.push(binary.name());
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
    let target_os = std::env::consts::OS;
    let mut platform_types = match target_os {
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

    // add updater if needed
    if self.is_update_enabled() {
      platform_types.push(PackageType::Updater)
    }

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

  /// Returns true if verbose logging is enabled
  pub fn is_verbose(&self) -> bool {
    self.is_verbose
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
    self.bundle_settings.identifier.as_deref().unwrap_or("")
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
    self.bundle_settings.copyright.as_deref()
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
    &self.package.homepage.as_deref().unwrap_or("")
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
    self.bundle_settings.long_description.as_deref()
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
    self.bundle_settings.osx_minimum_system_version.as_deref()
  }

  /// Returns the path to the DMG bundle license.
  pub fn osx_license(&self) -> Option<&str> {
    self.bundle_settings.osx_license.as_deref()
  }

  /// Returns whether the macOS .app bundle should use the bootstrap script or not.
  pub fn osx_use_bootstrapper(&self) -> bool {
    self.bundle_settings.osx_use_bootstrapper.unwrap_or(false)
  }

  /// Is update enabled
  pub fn is_update_enabled(&self) -> bool {
    match &self.bundle_settings.updater {
      Some(val) => val.active,
      None => false,
    }
  }

  /// Is pubkey provided?
  pub fn is_updater_pubkey(&self) -> bool {
    match &self.bundle_settings.updater {
      Some(val) => val.pubkey.is_some(),
      None => false,
    }
  }

  /// Get pubkey (mainly for testing)
  #[cfg(test)]
  pub fn updater_pubkey(&self) -> Option<&str> {
    self
      .bundle_settings
      .updater
      .as_ref()
      .expect("Updater is not defined")
      .pubkey
      .as_deref()
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
        } else if let Some(current_path) = &self.current_pattern {
          if !self.current_pattern_is_valid {
            return Some(Err(crate::Error::GenericError(format!(
              "Path matching '{}' not found",
              current_path
            ))));
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
