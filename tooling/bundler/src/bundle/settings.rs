// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::category::AppCategory;
use crate::bundle::{common, platform::target_triple};
pub use tauri_utils::config::WebviewInstallMode;
use tauri_utils::{
  config::BundleType,
  resources::{external_binaries, ResourcePaths},
};

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

/// The type of the package we're bundling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PackageType {
  /// The macOS application bundle (.app).
  MacOsBundle,
  /// The iOS app bundle.
  IosBundle,
  /// The Windows bundle (.msi).
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

impl From<BundleType> for PackageType {
  fn from(bundle: BundleType) -> Self {
    match bundle {
      BundleType::Deb => Self::Deb,
      BundleType::AppImage => Self::AppImage,
      BundleType::Msi => Self::WindowsMsi,
      BundleType::App => Self::MacOsBundle,
      BundleType::Dmg => Self::Dmg,
      BundleType::Updater => Self::Updater,
    }
  }
}

impl PackageType {
  /// Maps a short name to a PackageType.
  /// Possible values are "deb", "ios", "msi", "app", "rpm", "appimage", "dmg", "updater".
  pub fn from_short_name(name: &str) -> Option<PackageType> {
    // Other types we may eventually want to support: apk.
    match name {
      "deb" => Some(PackageType::Deb),
      "ios" => Some(PackageType::IosBundle),
      "msi" => Some(PackageType::WindowsMsi),
      "app" => Some(PackageType::MacOsBundle),
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
      PackageType::WindowsMsi => "msi",
      PackageType::MacOsBundle => "app",
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
  #[cfg(target_os = "linux")]
  PackageType::Deb,
  #[cfg(target_os = "macos")]
  PackageType::IosBundle,
  #[cfg(target_os = "windows")]
  PackageType::WindowsMsi,
  #[cfg(target_os = "macos")]
  PackageType::MacOsBundle,
  #[cfg(target_os = "linux")]
  PackageType::Rpm,
  #[cfg(target_os = "macos")]
  PackageType::Dmg,
  #[cfg(target_os = "linux")]
  PackageType::AppImage,
  PackageType::Updater,
];

/// The package settings.
#[derive(Debug, Clone)]
pub struct PackageSettings {
  /// the package's product name.
  pub product_name: String,
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
#[derive(Debug, Default, Clone)]
pub struct UpdaterSettings {
  /// Whether the updater is active or not.
  pub active: bool,
  /// The updater endpoints.
  pub endpoints: Option<Vec<String>>,
  /// Signature public key.
  pub pubkey: String,
  /// Display built-in dialog or use event system if disabled.
  pub dialog: bool,
  /// Args to pass to `msiexec.exe` to run the updater on Windows.
  pub msiexec_args: Option<&'static [&'static str]>,
}

/// The Linux debian bundle settings.
#[derive(Clone, Debug, Default)]
pub struct DebianSettings {
  // OS-specific settings:
  /// the list of debian dependencies.
  pub depends: Option<Vec<String>>,
  /// List of custom files to add to the deb package.
  /// Maps the path on the debian package to the path of the file to include (relative to the current working directory).
  pub files: HashMap<PathBuf, PathBuf>,
}

/// The macOS bundle settings.
#[derive(Clone, Debug, Default)]
pub struct MacOsSettings {
  /// MacOS frameworks that need to be bundled with the app.
  ///
  /// Each string can either be the name of a framework (without the `.framework` extension, e.g. `"SDL2"`),
  /// in which case we will search for that framework in the standard install locations (`~/Library/Frameworks/`, `/Library/Frameworks/`, and `/Network/Library/Frameworks/`),
  /// or a path to a specific framework bundle (e.g. `./data/frameworks/SDL2.framework`).  Note that this setting just makes tauri-bundler copy the specified frameworks into the OS X app bundle
  /// (under `Foobar.app/Contents/Frameworks/`); you are still responsible for:
  ///
  /// - arranging for the compiled binary to link against those frameworks (e.g. by emitting lines like `cargo:rustc-link-lib=framework=SDL2` from your `build.rs` script)
  ///
  /// - embedding the correct rpath in your binary (e.g. by running `install_name_tool -add_rpath "@executable_path/../Frameworks" path/to/binary` after compiling)
  pub frameworks: Option<Vec<String>>,
  /// A version string indicating the minimum MacOS version that the bundled app supports (e.g. `"10.11"`).
  /// If you are using this config field, you may also want have your `build.rs` script emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11`.
  pub minimum_system_version: Option<String>,
  /// The path to the LICENSE file for macOS apps.
  /// Currently only used by the dmg bundle.
  pub license: Option<String>,
  /// The exception domain to use on the macOS .app bundle.
  ///
  /// This allows communication to the outside world e.g. a web server you're shipping.
  pub exception_domain: Option<String>,
  /// Code signing identity.
  pub signing_identity: Option<String>,
  /// Provider short name for notarization.
  pub provider_short_name: Option<String>,
  /// Path to the entitlements.plist file.
  pub entitlements: Option<String>,
  /// Path to the Info.plist file for the bundle.
  pub info_plist_path: Option<PathBuf>,
}

/// Configuration for a target language for the WiX build.
#[derive(Debug, Clone, Default)]
pub struct WixLanguageConfig {
  /// The path to a locale (`.wxl`) file. See <https://wixtoolset.org/documentation/manual/v3/howtos/ui_and_localization/build_a_localized_version.html>.
  pub locale_path: Option<PathBuf>,
}

/// The languages to build using WiX.
#[derive(Debug, Clone)]
pub struct WixLanguage(pub Vec<(String, WixLanguageConfig)>);

impl Default for WixLanguage {
  fn default() -> Self {
    Self(vec![("en-US".into(), Default::default())])
  }
}

/// Settings specific to the WiX implementation.
#[derive(Clone, Debug, Default)]
pub struct WixSettings {
  /// The app languages to build. See <https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables>.
  pub language: WixLanguage,
  /// By default, the bundler uses an internal template.
  /// This option allows you to define your own wix file.
  pub template: Option<PathBuf>,
  /// A list of paths to .wxs files with WiX fragments to use.
  pub fragment_paths: Vec<PathBuf>,
  /// The ComponentGroup element ids you want to reference from the fragments.
  pub component_group_refs: Vec<String>,
  /// The Component element ids you want to reference from the fragments.
  pub component_refs: Vec<String>,
  /// The FeatureGroup element ids you want to reference from the fragments.
  pub feature_group_refs: Vec<String>,
  /// The Feature element ids you want to reference from the fragments.
  pub feature_refs: Vec<String>,
  /// The Merge element ids you want to reference from the fragments.
  pub merge_refs: Vec<String>,
  /// Disables the Webview2 runtime installation after app install. Will be removed in v2, use [`WindowsSettings::webview_install_mode`] instead.
  pub skip_webview_install: bool,
  /// The path to the LICENSE file.
  pub license: Option<PathBuf>,
  /// Create an elevated update task within Windows Task Scheduler.
  pub enable_elevated_update_task: bool,
  /// Path to a bitmap file to use as the installation user interface banner.
  /// This bitmap will appear at the top of all but the first page of the installer.
  ///
  /// The required dimensions are 493px × 58px.
  pub banner_path: Option<PathBuf>,
  /// Path to a bitmap file to use on the installation user interface dialogs.
  /// It is used on the welcome and completion dialogs.

  /// The required dimensions are 493px × 312px.
  pub dialog_image_path: Option<PathBuf>,
}

/// The Windows bundle settings.
#[derive(Clone, Debug)]
pub struct WindowsSettings {
  /// The file digest algorithm to use for creating file signatures. Required for code signing. SHA-256 is recommended.
  pub digest_algorithm: Option<String>,
  /// The SHA1 hash of the signing certificate.
  pub certificate_thumbprint: Option<String>,
  /// Server to use during timestamping.
  pub timestamp_url: Option<String>,
  /// Whether to use Time-Stamp Protocol (TSP, a.k.a. RFC 3161) for the timestamp server. Your code signing provider may
  /// use a TSP timestamp server, like e.g. SSL.com does. If so, enable TSP by setting to true.
  pub tsp: bool,
  /// WiX configuration.
  pub wix: Option<WixSettings>,
  /// The path to the application icon. Defaults to `./icons/icon.ico`.
  pub icon_path: PathBuf,
  /// The installation mode for the Webview2 runtime.
  pub webview_install_mode: WebviewInstallMode,
  /// Path to the webview fixed runtime to use.
  ///
  /// Overwrites [`Self::webview_install_mode`] if set.
  ///
  /// Will be removed in v2, use [`Self::webview_install_mode`] instead.
  pub webview_fixed_runtime_path: Option<PathBuf>,
  /// Validates a second app installation, blocking the user from installing an older version if set to `false`.
  ///
  /// For instance, if `1.2.1` is installed, the user won't be able to install app version `1.2.0` or `1.1.5`.
  ///
  /// /// The default value of this flag is `true`.
  pub allow_downgrades: bool,
}

impl Default for WindowsSettings {
  fn default() -> Self {
    Self {
      digest_algorithm: None,
      certificate_thumbprint: None,
      timestamp_url: None,
      tsp: false,
      wix: None,
      icon_path: PathBuf::from("icons/icon.ico"),
      webview_install_mode: Default::default(),
      webview_fixed_runtime_path: None,
      allow_downgrades: true,
    }
  }
}

/// The bundle settings of the BuildArtifact we're bundling.
#[derive(Clone, Debug, Default)]
pub struct BundleSettings {
  /// the app's identifier.
  pub identifier: Option<String>,
  /// the app's icon list.
  pub icon: Option<Vec<String>>,
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
  // Bundles for other binaries:
  /// Configuration map for the apps to bundle.
  pub bin: Option<HashMap<String, BundleSettings>>,
  /// External binaries to add to the bundle.
  ///
  /// Note that each binary name should have the target platform's target triple appended,
  /// as well as `.exe` for Windows.
  /// For example, if you're bundling a sidecar called `sqlite3`, the bundler expects
  /// a binary named `sqlite3-x86_64-unknown-linux-gnu` on linux,
  /// and `sqlite3-x86_64-pc-windows-gnu.exe` on windows.
  ///
  /// Run `tauri build --help` for more info on targets.
  ///
  /// If you are building a universal binary for MacOS, the bundler expects
  /// your external binary to also be universal, and named after the target triple,
  /// e.g. `sqlite3-universal-apple-darwin`. See
  /// <https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary>
  pub external_bin: Option<Vec<String>>,
  /// Debian-specific settings.
  pub deb: DebianSettings,
  /// MacOS-specific settings.
  pub macos: MacOsSettings,
  /// Updater configuration.
  pub updater: Option<UpdaterSettings>,
  /// Windows-specific settings.
  pub windows: WindowsSettings,
}

/// A binary to bundle.
#[derive(Clone, Debug)]
pub struct BundleBinary {
  name: String,
  src_path: Option<String>,
  main: bool,
}

impl BundleBinary {
  /// Creates a new bundle binary.
  pub fn new(name: String, main: bool) -> Self {
    Self {
      name,
      src_path: None,
      main,
    }
  }

  /// Sets the src path of the binary.
  #[must_use]
  pub fn set_src_path(mut self, src_path: Option<String>) -> Self {
    self.src_path = src_path;
    self
  }

  /// Mark the binary as the main executable.
  pub fn set_main(&mut self, main: bool) {
    self.main = main;
  }

  /// Sets the binary name.
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  /// Returns the binary name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the binary `main` flag.
  pub fn main(&self) -> bool {
    self.main
  }

  /// Returns the binary source path.
  pub fn src_path(&self) -> Option<&String> {
    self.src_path.as_ref()
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
  /// the bundle settings.
  bundle_settings: BundleSettings,
  /// the binaries to bundle.
  binaries: Vec<BundleBinary>,
  /// The target triple.
  target: String,
}

/// A builder for [`Settings`].
#[derive(Default)]
pub struct SettingsBuilder {
  project_out_directory: Option<PathBuf>,
  package_types: Option<Vec<PackageType>>,
  package_settings: Option<PackageSettings>,
  bundle_settings: BundleSettings,
  binaries: Vec<BundleBinary>,
  target: Option<String>,
}

impl SettingsBuilder {
  /// Creates the default settings builder.
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the project output directory. It's used as current working directory.
  #[must_use]
  pub fn project_out_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
    self
      .project_out_directory
      .replace(path.as_ref().to_path_buf());
    self
  }

  /// Sets the package types to create.
  #[must_use]
  pub fn package_types(mut self, package_types: Vec<PackageType>) -> Self {
    self.package_types = Some(package_types);
    self
  }

  /// Sets the package settings.
  #[must_use]
  pub fn package_settings(mut self, settings: PackageSettings) -> Self {
    self.package_settings.replace(settings);
    self
  }

  /// Sets the bundle settings.
  #[must_use]
  pub fn bundle_settings(mut self, settings: BundleSettings) -> Self {
    self.bundle_settings = settings;
    self
  }

  /// Sets the binaries to bundle.
  #[must_use]
  pub fn binaries(mut self, binaries: Vec<BundleBinary>) -> Self {
    self.binaries = binaries;
    self
  }

  /// Sets the target triple.
  #[must_use]
  pub fn target(mut self, target: String) -> Self {
    self.target.replace(target);
    self
  }

  /// Builds a Settings from the CLI args.
  ///
  /// Package settings will be read from Cargo.toml.
  ///
  /// Bundle settings will be read from from $TAURI_DIR/tauri.conf.json if it exists and fallback to Cargo.toml's [package.metadata.bundle].
  pub fn build(self) -> crate::Result<Settings> {
    let target = if let Some(t) = self.target {
      t
    } else {
      target_triple()?
    };

    Ok(Settings {
      package: self.package_settings.expect("package settings is required"),
      package_types: self.package_types,
      project_out_directory: self
        .project_out_directory
        .expect("out directory is required"),
      binaries: self.binaries,
      bundle_settings: BundleSettings {
        external_bin: self
          .bundle_settings
          .external_bin
          .as_ref()
          .map(|bins| external_binaries(bins, &target)),
        ..self.bundle_settings
      },
      target,
    })
  }
}

impl Settings {
  /// Returns the directory where the bundle should be placed.
  pub fn project_out_directory(&self) -> &Path {
    &self.project_out_directory
  }

  /// Returns the target triple.
  pub fn target(&self) -> &str {
    &self.target
  }

  /// Returns the architecture for the binary being bundled (e.g. "arm", "x86" or "x86_64").
  pub fn binary_arch(&self) -> &str {
    if self.target.starts_with("x86_64") {
      "x86_64"
    } else if self.target.starts_with('i') {
      "x86"
    } else if self.target.starts_with("arm") {
      "arm"
    } else if self.target.starts_with("aarch64") {
      "aarch64"
    } else if self.target.starts_with("universal") {
      "universal"
    } else {
      panic!("Unexpected target triple {}", self.target)
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
    path.push(binary.name());
    path
  }

  /// Returns the list of binaries to bundle.
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
      "macos" => vec![PackageType::MacOsBundle, PackageType::Dmg],
      "ios" => vec![PackageType::IosBundle],
      "linux" => vec![PackageType::Deb, PackageType::AppImage],
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

  /// Returns the product name.
  pub fn product_name(&self) -> &str {
    &self.package.product_name
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

  /// Copies external binaries to a path.
  pub fn copy_binaries(&self, path: &Path) -> crate::Result<()> {
    for src in self.external_binaries() {
      let src = src?;
      let dest = path.join(
        src
          .file_name()
          .expect("failed to extract external binary filename")
          .to_string_lossy()
          .replace(&format!("-{}", self.target), ""),
      );
      common::copy_file(&src, &dest)?;
    }
    Ok(())
  }

  /// Copies resources to a path.
  pub fn copy_resources(&self, path: &Path) -> crate::Result<()> {
    for src in self.resource_files() {
      let src = src?;
      let dest = path.join(tauri_utils::resources::resource_relpath(&src));
      common::copy_file(&src, &dest)?;
    }
    Ok(())
  }

  /// Returns the version string of the bundle.
  pub fn version_string(&self) -> &str {
    &self.package.version
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
    self.package.homepage.as_deref().unwrap_or("")
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

  /// Returns the debian settings.
  pub fn deb(&self) -> &DebianSettings {
    &self.bundle_settings.deb
  }

  /// Returns the MacOS settings.
  pub fn macos(&self) -> &MacOsSettings {
    &self.bundle_settings.macos
  }

  /// Returns the Windows settings.
  pub fn windows(&self) -> &WindowsSettings {
    &self.bundle_settings.windows
  }

  /// Returns the Updater settings.
  pub fn updater(&self) -> Option<&UpdaterSettings> {
    self.bundle_settings.updater.as_ref()
  }

  /// Is update enabled
  pub fn is_update_enabled(&self) -> bool {
    match &self.bundle_settings.updater {
      Some(val) => val.active,
      None => false,
    }
  }
}
