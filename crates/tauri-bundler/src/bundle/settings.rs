// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::category::AppCategory;
use crate::{bundle::platform::target_triple, utils::fs_utils};
use anyhow::Context;
pub use tauri_utils::config::WebviewInstallMode;
use tauri_utils::{
  config::{
    BundleType, DeepLinkProtocol, FileAssociation, NSISInstallerMode, NsisCompression,
    RpmCompression,
  },
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
  /// The NSIS bundle (.exe).
  Nsis,
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
      BundleType::Rpm => Self::Rpm,
      BundleType::AppImage => Self::AppImage,
      BundleType::Msi => Self::WindowsMsi,
      BundleType::Nsis => Self::Nsis,
      BundleType::App => Self::MacOsBundle,
      BundleType::Dmg => Self::Dmg,
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
      "nsis" => Some(PackageType::Nsis),
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
      PackageType::Nsis => "nsis",
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

  /// Gets a number representing priority which used to sort package types
  /// in an order that guarantees that if a certain package type
  /// depends on another (like Dmg depending on MacOsBundle), the dependency
  /// will be built first
  ///
  /// The lower the number, the higher the priority
  pub fn priority(&self) -> u32 {
    match self {
      PackageType::MacOsBundle => 0,
      PackageType::IosBundle => 0,
      PackageType::WindowsMsi => 0,
      PackageType::Nsis => 0,
      PackageType::Deb => 0,
      PackageType::Rpm => 0,
      PackageType::AppImage => 0,
      PackageType::Dmg => 1,
      PackageType::Updater => 2,
    }
  }
}

const ALL_PACKAGE_TYPES: &[PackageType] = &[
  #[cfg(target_os = "linux")]
  PackageType::Deb,
  #[cfg(target_os = "macos")]
  PackageType::IosBundle,
  #[cfg(target_os = "windows")]
  PackageType::WindowsMsi,
  #[cfg(target_os = "windows")]
  PackageType::Nsis,
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
  /// Should generate v1 compatible zipped updater
  pub v1_compatible: bool,
  /// Signature public key.
  pub pubkey: String,
  /// Args to pass to `msiexec.exe` to run the updater on Windows.
  pub msiexec_args: &'static [&'static str],
}

/// The Linux debian bundle settings.
#[derive(Clone, Debug, Default)]
pub struct DebianSettings {
  // OS-specific settings:
  /// the list of debian dependencies.
  pub depends: Option<Vec<String>>,
  /// the list of debian dependencies recommendations.
  pub recommends: Option<Vec<String>>,
  /// the list of dependencies the package provides.
  pub provides: Option<Vec<String>>,
  /// the list of package conflicts.
  pub conflicts: Option<Vec<String>>,
  /// the list of package replaces.
  pub replaces: Option<Vec<String>>,
  /// List of custom files to add to the deb package.
  /// Maps the path on the debian package to the path of the file to include (relative to the current working directory).
  pub files: HashMap<PathBuf, PathBuf>,
  /// Path to a custom desktop file Handlebars template.
  ///
  /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
  ///
  /// Default file contents:
  /// ```text
  #[doc = include_str!("./linux/freedesktop/main.desktop")]
  /// ```
  pub desktop_template: Option<PathBuf>,
  /// Define the section in Debian Control file. See : <https://www.debian.org/doc/debian-policy/ch-archive.html#s-subsections>
  pub section: Option<String>,
  /// Change the priority of the Debian Package. By default, it is set to `optional`.
  /// Recognized Priorities as of now are :  `required`, `important`, `standard`, `optional`, `extra`
  pub priority: Option<String>,
  /// Path of the uncompressed Changelog file, to be stored at /usr/share/doc/package-name/changelog.gz. See
  /// <https://www.debian.org/doc/debian-policy/ch-docs.html#changelog-files-and-release-notes>
  pub changelog: Option<PathBuf>,
  /// Path to script that will be executed before the package is unpacked. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  pub pre_install_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is unpacked. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  pub post_install_script: Option<PathBuf>,
  /// Path to script that will be executed before the package is removed. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  pub pre_remove_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is removed. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  pub post_remove_script: Option<PathBuf>,
}

/// The Linux AppImage bundle settings.
#[derive(Clone, Debug, Default)]
pub struct AppImageSettings {
  /// The files to include in the Appimage Binary.
  pub files: HashMap<PathBuf, PathBuf>,
  /// Whether to include gstreamer plugins for audio/media support.
  pub bundle_media_framework: bool,
  /// Whether to include the `xdg-open` binary.
  pub bundle_xdg_open: bool,
}

/// The RPM bundle settings.
#[derive(Clone, Debug, Default)]
pub struct RpmSettings {
  /// The list of RPM dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// the list of of RPM dependencies your application recommends.
  pub recommends: Option<Vec<String>>,
  /// The list of RPM dependencies your application provides.
  pub provides: Option<Vec<String>>,
  /// The list of RPM dependencies your application conflicts with. They must not be present
  /// in order for the package to be installed.
  pub conflicts: Option<Vec<String>>,
  /// The list of RPM dependencies your application supersedes - if this package is installed,
  /// packages listed as "obsoletes" will be automatically removed (if they are present).
  pub obsoletes: Option<Vec<String>>,
  /// The RPM release tag.
  pub release: String,
  /// The RPM epoch.
  pub epoch: u32,
  /// List of custom files to add to the RPM package.
  /// Maps the path on the RPM package to the path of the file to include (relative to the current working directory).
  pub files: HashMap<PathBuf, PathBuf>,
  /// Path to a custom desktop file Handlebars template.
  ///
  /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
  ///
  /// Default file contents:
  /// ```text
  #[doc = include_str!("./linux/freedesktop/main.desktop")]
  /// ```
  pub desktop_template: Option<PathBuf>,
  /// Path to script that will be executed before the package is unpacked. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  pub pre_install_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is unpacked. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  pub post_install_script: Option<PathBuf>,
  /// Path to script that will be executed before the package is removed. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  pub pre_remove_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is removed. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  pub post_remove_script: Option<PathBuf>,
  /// Compression algorithm and level. Defaults to `Gzip` with level 6.
  pub compression: Option<RpmCompression>,
}

/// Position coordinates struct.
#[derive(Clone, Debug, Default)]
pub struct Position {
  /// X coordinate.
  pub x: u32,
  /// Y coordinate.
  pub y: u32,
}

/// Size of the window.
#[derive(Clone, Debug, Default)]
pub struct Size {
  /// Width of the window.
  pub width: u32,
  /// Height of the window.
  pub height: u32,
}

/// The DMG bundle settings.
#[derive(Clone, Debug, Default)]
pub struct DmgSettings {
  /// Image to use as the background in dmg file. Accepted formats: `png`/`jpg`/`gif`.
  pub background: Option<PathBuf>,
  /// Position of volume window on screen.
  pub window_position: Option<Position>,
  /// Size of volume window.
  pub window_size: Size,
  /// Position of app file on window.
  pub app_position: Position,
  /// Position of application folder on window.
  pub application_folder_position: Position,
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
  /// List of custom files to add to the application bundle.
  /// Maps the path in the Contents directory in the app to the path of the file to include (relative to the current working directory).
  pub files: HashMap<PathBuf, PathBuf>,
  /// A version string indicating the minimum MacOS version that the bundled app supports (e.g. `"10.11"`).
  /// If you are using this config field, you may also want have your `build.rs` script emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11`.
  pub minimum_system_version: Option<String>,
  /// The exception domain to use on the macOS .app bundle.
  ///
  /// This allows communication to the outside world e.g. a web server you're shipping.
  pub exception_domain: Option<String>,
  /// Code signing identity.
  pub signing_identity: Option<String>,
  /// Preserve the hardened runtime version flag, see <https://developer.apple.com/documentation/security/hardened_runtime>
  ///
  /// Settings this to `false` is useful when using an ad-hoc signature, making it less strict.
  pub hardened_runtime: bool,
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
  /// MSI installer version in the format `major.minor.patch.build` (build is optional).
  ///
  /// Because a valid version is required for MSI installer, it will be derived from [`PackageSettings::version`] if this field is not set.
  ///
  /// The first field is the major version and has a maximum value of 255. The second field is the minor version and has a maximum value of 255.
  /// The third and fourth fields have a maximum value of 65,535.
  ///
  /// See <https://learn.microsoft.com/en-us/windows/win32/msi/productversion> for more info.
  pub version: Option<String>,
  /// A GUID upgrade code for MSI installer. This code **_must stay the same across all of your updates_**,
  /// otherwise, Windows will treat your update as a different app and your users will have duplicate versions of your app.
  ///
  /// By default, tauri generates this code by generating a Uuid v5 using the string `<productName>.exe.app.x64` in the DNS namespace.
  /// You can use Tauri's CLI to generate and print this code for you by running `tauri inspect wix-upgrade-code`.
  ///
  /// It is recommended that you set this value in your tauri config file to avoid accidental changes in your upgrade code
  /// whenever you want to change your product name.
  pub upgrade_code: Option<uuid::Uuid>,
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
  /// Create an elevated update task within Windows Task Scheduler.
  pub enable_elevated_update_task: bool,
  /// Path to a bitmap file to use as the installation user interface banner.
  /// This bitmap will appear at the top of all but the first page of the installer.
  ///
  /// The required dimensions are 493px × 58px.
  pub banner_path: Option<PathBuf>,
  /// Path to a bitmap file to use on the installation user interface dialogs.
  /// It is used on the welcome and completion dialogs.
  ///
  /// The required dimensions are 493px × 312px.
  pub dialog_image_path: Option<PathBuf>,
  /// Enables FIPS compliant algorithms.
  pub fips_compliant: bool,
}

/// Settings specific to the NSIS implementation.
#[derive(Clone, Debug, Default)]
pub struct NsisSettings {
  /// A custom .nsi template to use.
  pub template: Option<PathBuf>,
  /// The path to a bitmap file to display on the header of installers pages.
  ///
  /// The recommended dimensions are 150px x 57px.
  pub header_image: Option<PathBuf>,
  /// The path to a bitmap file for the Welcome page and the Finish page.
  ///
  /// The recommended dimensions are 164px x 314px.
  pub sidebar_image: Option<PathBuf>,
  /// The path to an icon file used as the installer icon.
  pub installer_icon: Option<PathBuf>,
  /// Whether the installation will be for all users or just the current user.
  pub install_mode: NSISInstallerMode,
  /// A list of installer languages.
  /// By default the OS language is used. If the OS language is not in the list of languages, the first language will be used.
  /// To allow the user to select the language, set `display_language_selector` to `true`.
  ///
  /// See <https://github.com/kichik/nsis/tree/9465c08046f00ccb6eda985abbdbf52c275c6c4d/Contrib/Language%20files> for the complete list of languages.
  pub languages: Option<Vec<String>>,
  /// An key-value pair where the key is the language and the
  /// value is the path to a custom `.nsi` file that holds the translated text for tauri's custom messages.
  ///
  /// See <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/languages/English.nsh> for an example `.nsi` file.
  ///
  /// **Note**: the key must be a valid NSIS language and it must be added to [`NsisConfig`]languages array,
  pub custom_language_files: Option<HashMap<String, PathBuf>>,
  /// Whether to display a language selector dialog before the installer and uninstaller windows are rendered or not.
  /// By default the OS language is selected, with a fallback to the first language in the `languages` array.
  pub display_language_selector: bool,
  /// Set compression algorithm used to compress files in the installer.
  pub compression: NsisCompression,
  /// Set the folder name for the start menu shortcut.
  ///
  /// Use this option if you have multiple apps and wish to group their shortcuts under one folder
  /// or if you generally prefer to set your shortcut inside a folder.
  ///
  /// Examples:
  /// - `AwesomePublisher`, shortcut will be placed in `%AppData%\Microsoft\Windows\Start Menu\Programs\AwesomePublisher\<your-app>.lnk`
  /// - If unset, shortcut will be placed in `%AppData%\Microsoft\Windows\Start Menu\Programs\<your-app>.lnk`
  pub start_menu_folder: Option<String>,
  /// A path to a `.nsh` file that contains special NSIS macros to be hooked into the
  /// main installer.nsi script.
  ///
  /// Supported hooks are:
  /// - `NSIS_HOOK_PREINSTALL`: This hook runs before copying files, setting registry key values and creating shortcuts.
  /// - `NSIS_HOOK_POSTINSTALL`: This hook runs after the installer has finished copying all files, setting the registry keys and created shortcuts.
  /// - `NSIS_HOOK_PREUNINSTALL`: This hook runs before removing any files, registry keys and shortcuts.
  /// - `NSIS_HOOK_POSTUNINSTALL`: This hook runs after files, registry keys and shortcuts have been removed.
  ///
  ///
  /// ### Example
  ///
  /// ```nsh
  /// !macro NSIS_HOOK_PREINSTALL
  ///   MessageBox MB_OK "PreInstall"
  /// !macroend
  ///
  /// !macro NSIS_HOOK_POSTINSTALL
  ///   MessageBox MB_OK "PostInstall"
  /// !macroend
  ///
  /// !macro NSIS_HOOK_PREUNINSTALL
  ///   MessageBox MB_OK "PreUnInstall"
  /// !macroend
  ///
  /// !macro NSIS_HOOK_POSTUNINSTALL
  ///   MessageBox MB_OK "PostUninstall"
  /// !macroend
  /// ```
  pub installer_hooks: Option<PathBuf>,
  /// Try to ensure that the WebView2 version is equal to or newer than this version,
  /// if the user's WebView2 is older than this version,
  /// the installer will try to trigger a WebView2 update.
  pub minimum_webview2_version: Option<String>,
}

/// The Custom Signing Command Settings for Windows exe
#[derive(Clone, Debug)]
pub struct CustomSignCommandSettings {
  /// The command to run to sign the binary.
  pub cmd: String,
  /// The arguments to pass to the command.
  ///
  /// "%1" will be replaced with the path to the binary to be signed.
  pub args: Vec<String>,
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
  /// Nsis configuration.
  pub nsis: Option<NsisSettings>,
  /// The path to the application icon. Defaults to `./icons/icon.ico`.
  #[deprecated = "This is used for the MSI installer and will be removed in 3.0.0, use `BundleSettings::icon` field and make sure a `.ico` icon exists instead."]
  pub icon_path: PathBuf,
  /// The installation mode for the Webview2 runtime.
  pub webview_install_mode: WebviewInstallMode,
  /// Validates a second app installation, blocking the user from installing an older version if set to `false`.
  ///
  /// For instance, if `1.2.1` is installed, the user won't be able to install app version `1.2.0` or `1.1.5`.
  ///
  /// /// The default value of this flag is `true`.
  pub allow_downgrades: bool,

  /// Specify a custom command to sign the binaries.
  /// This command needs to have a `%1` in it which is just a placeholder for the binary path,
  /// which we will detect and replace before calling the command.
  ///
  /// Example:
  /// ```text
  /// sign-cli --arg1 --arg2 %1
  /// ```
  ///
  /// By Default we use `signtool.exe` which can be found only on Windows so
  /// if you are on another platform and want to cross-compile and sign you will
  /// need to use another tool like `osslsigncode`.
  pub sign_command: Option<CustomSignCommandSettings>,
}

#[allow(deprecated)]
mod _default {
  use super::*;

  impl Default for WindowsSettings {
    fn default() -> Self {
      Self {
        digest_algorithm: None,
        certificate_thumbprint: None,
        timestamp_url: None,
        tsp: false,
        wix: None,
        nsis: None,
        icon_path: PathBuf::from("icons/icon.ico"),
        webview_install_mode: Default::default(),
        allow_downgrades: true,
        sign_command: None,
      }
    }
  }
}

/// The bundle settings of the BuildArtifact we're bundling.
#[derive(Clone, Debug, Default)]
pub struct BundleSettings {
  /// the app's identifier.
  pub identifier: Option<String>,
  /// The app's publisher. Defaults to the second element in the identifier string.
  ///
  /// Currently maps to the Manufacturer property of the Windows Installer
  /// and the Maintainer field of debian packages if the Cargo.toml does not have the authors field.
  pub publisher: Option<String>,
  /// A url to the home page of your application. If None, will
  /// fallback to [PackageSettings::homepage].
  ///
  /// Supported bundle targets: `deb`, `rpm`, `nsis` and `msi`
  pub homepage: Option<String>,
  /// the app's icon list.
  pub icon: Option<Vec<String>>,
  /// the app's resources to bundle.
  ///
  /// each item can be a path to a file or a path to a folder.
  ///
  /// supports glob patterns.
  pub resources: Option<Vec<String>>,
  /// The app's resources to bundle. Takes precedence over `Self::resources` when specified.
  ///
  /// Maps each resource path to its target directory in the bundle resources directory.
  ///
  /// Supports glob patterns.
  pub resources_map: Option<HashMap<String, String>>,
  /// the app's copyright.
  pub copyright: Option<String>,
  /// The package's license identifier to be included in the appropriate bundles.
  /// If not set, defaults to the license from the Cargo.toml file.
  pub license: Option<String>,
  /// The path to the license file to be included in the appropriate bundles.
  pub license_file: Option<PathBuf>,
  /// the app's category.
  pub category: Option<AppCategory>,
  /// the file associations
  pub file_associations: Option<Vec<FileAssociation>>,
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
  /// Deep-link protocols.
  pub deep_link_protocols: Option<Vec<DeepLinkProtocol>>,
  /// Debian-specific settings.
  pub deb: DebianSettings,
  /// AppImage-specific settings.
  pub appimage: AppImageSettings,
  /// Rpm-specific settings.
  pub rpm: RpmSettings,
  /// DMG-specific settings.
  pub dmg: DmgSettings,
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
  main: bool,
  src_path: Option<String>,
}

impl BundleBinary {
  /// Creates a new bundle binary.
  pub fn new(name: String, main: bool) -> Self {
    Self {
      name,
      main,
      src_path: None,
    }
  }

  /// Creates a new bundle binary with path.
  pub fn with_path(name: String, main: bool, src_path: Option<String>) -> Self {
    Self {
      name,
      src_path,
      main,
    }
  }

  /// Mark the binary as the main executable.
  pub fn set_main(&mut self, main: bool) {
    self.main = main;
  }

  /// Sets the binary name.
  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  /// Sets the src path of the binary.
  #[must_use]
  pub fn set_src_path(mut self, src_path: Option<String>) -> Self {
    self.src_path = src_path;
    self
  }

  /// Returns the binary `main` flag.
  pub fn main(&self) -> bool {
    self.main
  }

  /// Returns the binary name.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns the binary source path.
  pub fn src_path(&self) -> Option<&String> {
    self.src_path.as_ref()
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arch {
  /// For the x86_64 / x64 / AMD64 instruction sets (64 bits).
  X86_64,
  /// For the x86 / i686 / i686 / 8086 instruction sets (32 bits).
  X86,
  /// For the AArch64 / ARM64 instruction sets (64 bits).
  AArch64,
  /// For the AArch32 / ARM32 instruction sets with hard-float (32 bits).
  Armhf,
  /// For the AArch32 / ARM32 instruction sets with soft-float (32 bits).
  Armel,
  /// For universal macOS applications.
  Universal,
}

/// The Settings exposed by the module.
#[derive(Clone, Debug)]
pub struct Settings {
  /// The log level.
  log_level: log::Level,
  /// the package settings.
  package: PackageSettings,
  /// the package types we're bundling.
  ///
  /// if not present, we'll use the PackageType list for the target OS.
  package_types: Option<Vec<PackageType>>,
  /// the directory where the bundles will be placed.
  project_out_directory: PathBuf,
  /// the directory to place tools used by the bundler,
  /// if `None`, tools are placed in the current user's platform-specific cache directory.
  local_tools_directory: Option<PathBuf>,
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
  log_level: Option<log::Level>,
  project_out_directory: Option<PathBuf>,
  package_types: Option<Vec<PackageType>>,
  package_settings: Option<PackageSettings>,
  bundle_settings: BundleSettings,
  binaries: Vec<BundleBinary>,
  target: Option<String>,
  local_tools_directory: Option<PathBuf>,
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

  /// Sets the directory to place tools used by the bundler
  /// when [`BundleSettings::use_local_tools_dir`] is true.
  #[must_use]
  pub fn local_tools_directory<P: AsRef<Path>>(mut self, path: P) -> Self {
    self
      .local_tools_directory
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

  /// Sets the log level for spawned commands. Defaults to [`log::Level::Error`].
  #[must_use]
  pub fn log_level(mut self, level: log::Level) -> Self {
    self.log_level.replace(level);
    self
  }

  /// Builds a Settings from the CLI args.
  ///
  /// Package settings will be read from Cargo.toml.
  ///
  /// Bundle settings will be read from $TAURI_DIR/tauri.conf.json if it exists and fallback to Cargo.toml's [package.metadata.bundle].
  pub fn build(self) -> crate::Result<Settings> {
    let target = if let Some(t) = self.target {
      t
    } else {
      target_triple()?
    };

    Ok(Settings {
      log_level: self.log_level.unwrap_or(log::Level::Error),
      package: self
        .package_settings
        .ok_or_else(|| crate::Error::GenericError("package settings is required".into()))?,
      package_types: self.package_types,
      project_out_directory: self
        .project_out_directory
        .ok_or_else(|| crate::Error::GenericError("out directory is required".into()))?,
      local_tools_directory: self.local_tools_directory,
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
  /// Sets the log level for spawned commands.
  pub fn set_log_level(&mut self, level: log::Level) {
    self.log_level = level;
  }

  /// Returns the log level for spawned commands.
  pub fn log_level(&self) -> log::Level {
    self.log_level
  }

  /// Returns the directory where the bundle should be placed.
  pub fn project_out_directory(&self) -> &Path {
    &self.project_out_directory
  }

  /// Returns the target triple.
  pub fn target(&self) -> &str {
    &self.target
  }

  /// Returns the architecture for the binary being bundled (e.g. "arm", "x86" or "x86_64").
  pub fn binary_arch(&self) -> Arch {
    if self.target.starts_with("x86_64") {
      Arch::X86_64
    } else if self.target.starts_with('i') {
      Arch::X86
    } else if self.target.starts_with("arm") && self.target.ends_with("hf") {
      Arch::Armhf
    } else if self.target.starts_with("arm") {
      Arch::Armel
    } else if self.target.starts_with("aarch64") {
      Arch::AArch64
    } else if self.target.starts_with("universal") {
      Arch::Universal
    } else {
      panic!("Unexpected target triple {}", self.target)
    }
  }

  /// Returns the file name of the binary being bundled.
  pub fn main_binary(&self) -> crate::Result<&BundleBinary> {
    self
      .binaries
      .iter()
      .find(|bin| bin.main)
      .context("failed to find main binary, make sure you have a `package > default-run` in the Cargo.toml file")
      .map_err(Into::into)
  }

  /// Returns the file name of the binary being bundled.
  pub fn main_binary_mut(&mut self) -> crate::Result<&mut BundleBinary> {
    self
      .binaries
      .iter_mut()
      .find(|bin| bin.main)
      .context("failed to find main binary, make sure you have a `package > default-run` in the Cargo.toml file")
      .map_err(Into::into)
  }

  /// Returns the file name of the binary being bundled.
  pub fn main_binary_name(&self) -> crate::Result<&str> {
    self
      .binaries
      .iter()
      .find(|bin| bin.main)
      .context("failed to find main binary, make sure you have a `package > default-run` in the Cargo.toml file")
      .map(|b| b.name())
      .map_err(Into::into)
  }

  /// Returns the path to the specified binary.
  pub fn binary_path(&self, binary: &BundleBinary) -> PathBuf {
    let target_os = self
      .target()
      .split('-')
      .nth(2)
      .unwrap_or(std::env::consts::OS);

    let path = self.project_out_directory.join(binary.name());

    if target_os == "windows" {
      path.with_extension("exe")
    } else {
      path
    }
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
    let target_os = self
      .target
      .split('-')
      .nth(2)
      .unwrap_or(std::env::consts::OS)
      .replace("darwin", "macos");

    let platform_types = match target_os.as_str() {
      "macos" => vec![PackageType::MacOsBundle, PackageType::Dmg],
      "ios" => vec![PackageType::IosBundle],
      "linux" => vec![PackageType::Deb, PackageType::Rpm, PackageType::AppImage],
      "windows" => vec![PackageType::WindowsMsi, PackageType::Nsis],
      os => {
        return Err(crate::Error::GenericError(format!(
          "Native {os} bundles not yet supported."
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

  /// Returns the product name.
  pub fn product_name(&self) -> &str {
    &self.package.product_name
  }

  /// Returns the bundle's identifier
  pub fn bundle_identifier(&self) -> &str {
    self.bundle_settings.identifier.as_deref().unwrap_or("")
  }

  /// Returns the bundle's publisher
  pub fn publisher(&self) -> Option<&str> {
    self.bundle_settings.publisher.as_deref()
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
    match (
      &self.bundle_settings.resources,
      &self.bundle_settings.resources_map,
    ) {
      (Some(paths), None) => ResourcePaths::new(paths.as_slice(), true),
      (None, Some(map)) => ResourcePaths::from_map(map, true),
      (Some(_), Some(_)) => panic!("cannot use both `resources` and `resources_map`"),
      (None, None) => ResourcePaths::new(&[], true),
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
  ///
  /// Returns the list of destination paths.
  pub fn copy_binaries(&self, path: &Path) -> crate::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for src in self.external_binaries() {
      let src = src?;
      let dest = path.join(
        src
          .file_name()
          .expect("failed to extract external binary filename")
          .to_string_lossy()
          .replace(&format!("-{}", self.target), ""),
      );
      fs_utils::copy_file(&src, &dest)?;
      paths.push(dest);
    }
    Ok(paths)
  }

  /// Copies resources to a path.
  pub fn copy_resources(&self, path: &Path) -> crate::Result<()> {
    for resource in self.resource_files().iter() {
      let resource = resource?;
      let dest = path.join(resource.target());
      fs_utils::copy_file(resource.path(), &dest)?;
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

  /// Returns the bundle license.
  pub fn license(&self) -> Option<String> {
    self.bundle_settings.license.clone()
  }

  /// Returns the bundle license file.
  pub fn license_file(&self) -> Option<PathBuf> {
    self.bundle_settings.license_file.clone()
  }

  /// Returns the package's homepage URL, defaulting to "" if not defined.
  pub fn homepage_url(&self) -> Option<&str> {
    self
      .bundle_settings
      .homepage
      .as_deref()
      .or(self.package.homepage.as_deref())
  }

  /// Returns the app's category.
  pub fn app_category(&self) -> Option<AppCategory> {
    self.bundle_settings.category
  }

  /// Return file associations.
  pub fn file_associations(&self) -> Option<&Vec<FileAssociation>> {
    self.bundle_settings.file_associations.as_ref()
  }

  /// Return the list of deep link protocols to be registered for
  /// this bundle.
  pub fn deep_link_protocols(&self) -> Option<&Vec<DeepLinkProtocol>> {
    self.bundle_settings.deep_link_protocols.as_ref()
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

  /// Returns the directory for local tools path.
  pub fn local_tools_directory(&self) -> Option<&Path> {
    self.local_tools_directory.as_deref()
  }

  /// Returns the debian settings.
  pub fn deb(&self) -> &DebianSettings {
    &self.bundle_settings.deb
  }

  /// Returns the appimage settings.
  pub fn appimage(&self) -> &AppImageSettings {
    &self.bundle_settings.appimage
  }

  /// Returns the RPM settings.
  pub fn rpm(&self) -> &RpmSettings {
    &self.bundle_settings.rpm
  }

  /// Returns the DMG settings.
  pub fn dmg(&self) -> &DmgSettings {
    &self.bundle_settings.dmg
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
}
