// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri configuration used at runtime.
//!
//! It is pulled from a `tauri.conf.json` file and the [`Config`] struct is generated at compile time.
//!
//! # Stability
//! This is a core functionality that is not considered part of the stable API.
//! If you use it, note that it may include breaking changes in the future.

use semver::Version;
use serde::{
  de::{Deserializer, Error as DeError, Visitor},
  Deserialize, Serialize, Serializer,
};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;
use url::Url;

use std::{
  collections::HashMap,
  fmt::{self, Display},
  fs::read_to_string,
  path::PathBuf,
  str::FromStr,
};

/// Items to help with parsing content into a [`Config`].
pub mod parse;

fn default_true() -> bool {
  true
}

/// An URL to open on a Tauri webview window.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum WindowUrl {
  /// An external URL.
  External(Url),
  /// The path portion of an app URL.
  /// For instance, to load `tauri://localhost/users/john`,
  /// you can simply provide `users/john` in this configuration.
  App(PathBuf),
}

impl fmt::Display for WindowUrl {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::External(url) => write!(f, "{url}"),
      Self::App(path) => write!(f, "{}", path.display()),
    }
  }
}

impl Default for WindowUrl {
  fn default() -> Self {
    Self::App("index.html".into())
  }
}

/// A bundle referenced by tauri-bundler.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
pub enum BundleType {
  /// The debian bundle (.deb).
  Deb,
  /// The AppImage bundle (.appimage).
  AppImage,
  /// The Microsoft Installer bundle (.msi).
  Msi,
  /// The NSIS bundle (.exe).
  Nsis,
  /// The macOS application bundle (.app).
  App,
  /// The Apple Disk Image bundle (.dmg).
  Dmg,
  /// The Tauri updater bundle.
  Updater,
}

impl Display for BundleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Deb => "deb",
        Self::AppImage => "appimage",
        Self::Msi => "msi",
        Self::Nsis => "nsis",
        Self::App => "app",
        Self::Dmg => "dmg",
        Self::Updater => "updater",
      }
    )
  }
}

impl Serialize for BundleType {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for BundleType {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
      "deb" => Ok(Self::Deb),
      "appimage" => Ok(Self::AppImage),
      "msi" => Ok(Self::Msi),
      "nsis" => Ok(Self::Nsis),
      "app" => Ok(Self::App),
      "dmg" => Ok(Self::Dmg),
      "updater" => Ok(Self::Updater),
      _ => Err(DeError::custom(format!("unknown bundle target '{s}'"))),
    }
  }
}

/// Targets to bundle. Each value is case insensitive.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum BundleTarget {
  /// Bundle all targets.
  #[default]
  All,
  /// A list of bundle targets.
  List(Vec<BundleType>),
  /// A single bundle target.
  One(BundleType),
}

#[cfg(feature = "schemars")]
pub(crate) trait Merge: Sized {
  fn merge(self, other: Self) -> Self;
}

#[cfg(feature = "schema")]
use schemars::schema::{Metadata, Schema};

#[cfg(feature = "schema")]
impl<T: Merge> Merge for Option<T> {
  fn merge(self, other: Self) -> Self {
    match (self, other) {
      (Some(x), Some(y)) => Some(x.merge(y)),
      (None, y) => y,
      (x, None) => x,
    }
  }
}

#[cfg(feature = "schema")]
impl<T: Merge> Merge for Box<T> {
  fn merge(mut self, other: Self) -> Self {
    *self = (*self).merge(*other);
    self
  }
}

#[cfg(feature = "schema")]
impl<T> Merge for Vec<T> {
  fn merge(mut self, other: Self) -> Self {
    self.extend(other);
    self
  }
}

#[cfg(feature = "schema")]
impl Merge for Metadata {
  fn merge(self, other: Self) -> Self {
    Metadata {
      id: self.id.or(other.id),
      title: self.title.or(other.title),
      description: self.description.or(other.description),
      default: self.default.or(other.default),
      deprecated: self.deprecated || other.deprecated,
      read_only: self.read_only || other.read_only,
      write_only: self.write_only || other.write_only,
      examples: self.examples.merge(other.examples),
    }
  }
}

#[cfg(feature = "schema")]
fn apply_metadata(schema: Schema, metadata: Metadata) -> Schema {
  if metadata == Metadata::default() {
    schema
  } else {
    let mut schema_obj = schema.into_object();
    schema_obj.metadata = Some(Box::new(metadata)).merge(schema_obj.metadata);
    Schema::Object(schema_obj)
  }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for BundleTarget {
  fn schema_name() -> std::string::String {
    "BundleTarget".to_owned()
  }

  fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let any_of = vec![
      schemars::schema::SchemaObject {
        enum_values: Some(vec!["all".into()]),
        metadata: Some(Box::new(schemars::schema::Metadata {
          description: Some("Bundle all targets.".to_owned()),
          ..Default::default()
        })),
        ..Default::default()
      }
      .into(),
      apply_metadata(
        generator.subschema_for::<Vec<BundleType>>(),
        schemars::schema::Metadata {
          description: Some("A list of bundle targets.".to_owned()),
          ..Default::default()
        },
      ),
      apply_metadata(
        generator.subschema_for::<BundleType>(),
        schemars::schema::Metadata {
          description: Some("A single bundle target.".to_owned()),
          ..Default::default()
        },
      ),
    ];

    schemars::schema::SchemaObject {
      subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
        any_of: Some(any_of),
        ..Default::default()
      })),
      metadata: Some(Box::new(schemars::schema::Metadata {
        description: Some("Targets to bundle. Each value is case insensitive.".to_owned()),
        ..Default::default()
      })),
      ..Default::default()
    }
    .into()
  }
}

impl Serialize for BundleTarget {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::All => serializer.serialize_str("all"),
      Self::List(l) => l.serialize(serializer),
      Self::One(t) => serializer.serialize_str(t.to_string().as_ref()),
    }
  }
}

impl<'de> Deserialize<'de> for BundleTarget {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum BundleTargetInner {
      List(Vec<BundleType>),
      One(BundleType),
      All(String),
    }

    match BundleTargetInner::deserialize(deserializer)? {
      BundleTargetInner::All(s) if s.to_lowercase() == "all" => Ok(Self::All),
      BundleTargetInner::All(t) => Err(DeError::custom(format!("invalid bundle type {t}"))),
      BundleTargetInner::List(l) => Ok(Self::List(l)),
      BundleTargetInner::One(t) => Ok(Self::One(t)),
    }
  }
}

/// Configuration for AppImage bundles.
///
/// See more: https://tauri.app/v1/api/config#appimageconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppImageConfig {
  /// Include additional gstreamer dependencies needed for audio and video playback.
  /// This increases the bundle size by ~15-35MB depending on your build system.
  #[serde(default, alias = "bundle-media-framework")]
  pub bundle_media_framework: bool,
}

/// Configuration for Debian (.deb) bundles.
///
/// See more: https://tauri.app/v1/api/config#debconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebConfig {
  /// The list of deb dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// The files to include on the package.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
  /// Path to a custom desktop file Handlebars template.
  ///
  /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
  pub desktop_template: Option<PathBuf>,
  /// Define the section in Debian Control file. See : https://www.debian.org/doc/debian-policy/ch-archive.html#s-subsections
  pub section: Option<String>,
  /// Change the priority of the Debian Package. By default, it is set to `optional`.
  /// Recognized Priorities as of now are :  `required`, `important`, `standard`, `optional`, `extra`
  pub priority: Option<String>,
  /// Path of the uncompressed Changelog file, to be stored at /usr/share/doc/package-name/changelog.gz. See
  /// https://www.debian.org/doc/debian-policy/ch-docs.html#changelog-files-and-release-notes
  pub changelog: Option<PathBuf>,
}

fn de_minimum_system_version<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let version = Option::<String>::deserialize(deserializer)?;
  match version {
    Some(v) if v.is_empty() => Ok(minimum_system_version()),
    e => Ok(e),
  }
}

/// Configuration for the macOS bundles.
///
/// See more: https://tauri.app/v1/api/config#macconfig
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MacConfig {
  /// A list of strings indicating any macOS X frameworks that need to be bundled with the application.
  ///
  /// If a name is used, ".framework" must be omitted and it will look for standard install locations. You may also use a path to a specific framework.
  pub frameworks: Option<Vec<String>>,
  /// A version string indicating the minimum macOS X version that the bundled application supports. Defaults to `10.13`.
  ///
  /// Setting it to `null` completely removes the `LSMinimumSystemVersion` field on the bundle's `Info.plist`
  /// and the `MACOSX_DEPLOYMENT_TARGET` environment variable.
  ///
  /// An empty string is considered an invalid value so the default value is used.
  #[serde(
    deserialize_with = "de_minimum_system_version",
    default = "minimum_system_version",
    alias = "minimum-system-version"
  )]
  pub minimum_system_version: Option<String>,
  /// Allows your application to communicate with the outside world.
  /// It should be a lowercase, without port and protocol domain name.
  #[serde(alias = "exception-domain")]
  pub exception_domain: Option<String>,
  /// The path to the license file to add to the DMG bundle.
  pub license: Option<String>,
  /// Identity to use for code signing.
  #[serde(alias = "signing-identity")]
  pub signing_identity: Option<String>,
  /// Provider short name for notarization.
  #[serde(alias = "provider-short-name")]
  pub provider_short_name: Option<String>,
  /// Path to the entitlements file.
  pub entitlements: Option<String>,
}

impl Default for MacConfig {
  fn default() -> Self {
    Self {
      frameworks: None,
      minimum_system_version: minimum_system_version(),
      exception_domain: None,
      license: None,
      signing_identity: None,
      provider_short_name: None,
      entitlements: None,
    }
  }
}

fn minimum_system_version() -> Option<String> {
  Some("10.13".into())
}

/// Configuration for a target language for the WiX build.
///
/// See more: https://tauri.app/v1/api/config#wixlanguageconfig
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixLanguageConfig {
  /// The path to a locale (`.wxl`) file. See <https://wixtoolset.org/documentation/manual/v3/howtos/ui_and_localization/build_a_localized_version.html>.
  #[serde(alias = "locale-path")]
  pub locale_path: Option<String>,
}

/// The languages to build using WiX.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum WixLanguage {
  /// A single language to build, without configuration.
  One(String),
  /// A list of languages to build, without configuration.
  List(Vec<String>),
  /// A map of languages and its configuration.
  Localized(HashMap<String, WixLanguageConfig>),
}

impl Default for WixLanguage {
  fn default() -> Self {
    Self::One("en-US".into())
  }
}

/// Configuration for the MSI bundle using WiX.
///
/// See more: https://tauri.app/v1/api/config#wixconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixConfig {
  /// The installer languages to build. See <https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables>.
  #[serde(default)]
  pub language: WixLanguage,
  /// A custom .wxs template to use.
  pub template: Option<PathBuf>,
  /// A list of paths to .wxs files with WiX fragments to use.
  #[serde(default, alias = "fragment-paths")]
  pub fragment_paths: Vec<PathBuf>,
  /// The ComponentGroup element ids you want to reference from the fragments.
  #[serde(default, alias = "component-group-refs")]
  pub component_group_refs: Vec<String>,
  /// The Component element ids you want to reference from the fragments.
  #[serde(default, alias = "component-refs")]
  pub component_refs: Vec<String>,
  /// The FeatureGroup element ids you want to reference from the fragments.
  #[serde(default, alias = "feature-group-refs")]
  pub feature_group_refs: Vec<String>,
  /// The Feature element ids you want to reference from the fragments.
  #[serde(default, alias = "feature-refs")]
  pub feature_refs: Vec<String>,
  /// The Merge element ids you want to reference from the fragments.
  #[serde(default, alias = "merge-refs")]
  pub merge_refs: Vec<String>,
  /// Disables the Webview2 runtime installation after app install.
  ///
  /// Will be removed in v2, prefer the [`WindowsConfig::webview_install_mode`] option.
  #[serde(default, alias = "skip-webview-install")]
  pub skip_webview_install: bool,
  /// The path to the license file to render on the installer.
  ///
  /// Must be an RTF file, so if a different extension is provided, we convert it to the RTF format.
  pub license: Option<PathBuf>,
  /// Create an elevated update task within Windows Task Scheduler.
  #[serde(default, alias = "enable-elevated-update-task")]
  pub enable_elevated_update_task: bool,
  /// Path to a bitmap file to use as the installation user interface banner.
  /// This bitmap will appear at the top of all but the first page of the installer.
  ///
  /// The required dimensions are 493px × 58px.
  #[serde(alias = "banner-path")]
  pub banner_path: Option<PathBuf>,
  /// Path to a bitmap file to use on the installation user interface dialogs.
  /// It is used on the welcome and completion dialogs.
  ///
  /// The required dimensions are 493px × 312px.
  #[serde(alias = "dialog-image-path")]
  pub dialog_image_path: Option<PathBuf>,
}

/// Compression algorithms used in the NSIS installer.
///
/// See <https://nsis.sourceforge.io/Reference/SetCompressor>
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum NsisCompression {
  /// ZLIB uses the deflate algorithm, it is a quick and simple method. With the default compression level it uses about 300 KB of memory.
  Zlib,
  /// BZIP2 usually gives better compression ratios than ZLIB, but it is a bit slower and uses more memory. With the default compression level it uses about 4 MB of memory.
  Bzip2,
  /// LZMA (default) is a new compression method that gives very good compression ratios. The decompression speed is high (10-20 MB/s on a 2 GHz CPU), the compression speed is lower. The memory size that will be used for decompression is the dictionary size plus a few KBs, the default is 8 MB.
  Lzma,
}

/// Configuration for the Installer bundle using NSIS.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NsisConfig {
  /// A custom .nsi template to use.
  pub template: Option<PathBuf>,
  /// The path to the license file to render on the installer.
  pub license: Option<PathBuf>,
  /// The path to a bitmap file to display on the header of installers pages.
  ///
  /// The recommended dimensions are 150px x 57px.
  #[serde(alias = "header-image")]
  pub header_image: Option<PathBuf>,
  /// The path to a bitmap file for the Welcome page and the Finish page.
  ///
  /// The recommended dimensions are 164px x 314px.
  #[serde(alias = "sidebar-image")]
  pub sidebar_image: Option<PathBuf>,
  /// The path to an icon file used as the installer icon.
  #[serde(alias = "install-icon")]
  pub installer_icon: Option<PathBuf>,
  /// Whether the installation will be for all users or just the current user.
  #[serde(default, alias = "install-mode")]
  pub install_mode: NSISInstallerMode,
  /// A list of installer languages.
  /// By default the OS language is used. If the OS language is not in the list of languages, the first language will be used.
  /// To allow the user to select the language, set `display_language_selector` to `true`.
  ///
  /// See <https://github.com/kichik/nsis/tree/9465c08046f00ccb6eda985abbdbf52c275c6c4d/Contrib/Language%20files> for the complete list of languages.
  pub languages: Option<Vec<String>>,
  /// A key-value pair where the key is the language and the
  /// value is the path to a custom `.nsh` file that holds the translated text for tauri's custom messages.
  ///
  /// See <https://github.com/tauri-apps/tauri/blob/dev/tooling/bundler/src/bundle/windows/templates/nsis-languages/English.nsh> for an example `.nsh` file.
  ///
  /// **Note**: the key must be a valid NSIS language and it must be added to [`NsisConfig`] languages array,
  pub custom_language_files: Option<HashMap<String, PathBuf>>,
  /// Whether to display a language selector dialog before the installer and uninstaller windows are rendered or not.
  /// By default the OS language is selected, with a fallback to the first language in the `languages` array.
  #[serde(default, alias = "display-language-selector")]
  pub display_language_selector: bool,
  /// Set the compression algorithm used to compress files in the installer.
  ///
  /// See <https://nsis.sourceforge.io/Reference/SetCompressor>
  pub compression: Option<NsisCompression>,
}

/// Install Modes for the NSIS installer.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum NSISInstallerMode {
  /// Default mode for the installer.
  ///
  /// Install the app by default in a directory that doesn't require Administrator access.
  ///
  /// Installer metadata will be saved under the `HKCU` registry path.
  #[default]
  CurrentUser,
  /// Install the app by default in the `Program Files` folder directory requires Administrator
  /// access for the installation.
  ///
  /// Installer metadata will be saved under the `HKLM` registry path.
  PerMachine,
  /// Combines both modes and allows the user to choose at install time
  /// whether to install for the current user or per machine. Note that this mode
  /// will require Administrator access even if the user wants to install it for the current user only.
  ///
  /// Installer metadata will be saved under the `HKLM` or `HKCU` registry path based on the user's choice.
  Both,
}

/// Install modes for the Webview2 runtime.
/// Note that for the updater bundle [`Self::DownloadBootstrapper`] is used.
///
/// For more information see <https://tauri.app/v1/guides/building/windows>.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum WebviewInstallMode {
  /// Do not install the Webview2 as part of the Windows Installer.
  Skip,
  /// Download the bootstrapper and run it.
  /// Requires an internet connection.
  /// Results in a smaller installer size, but is not recommended on Windows 7.
  DownloadBootstrapper {
    /// Instructs the installer to run the bootstrapper in silent mode. Defaults to `true`.
    #[serde(default = "default_true")]
    silent: bool,
  },
  /// Embed the bootstrapper and run it.
  /// Requires an internet connection.
  /// Increases the installer size by around 1.8MB, but offers better support on Windows 7.
  EmbedBootstrapper {
    /// Instructs the installer to run the bootstrapper in silent mode. Defaults to `true`.
    #[serde(default = "default_true")]
    silent: bool,
  },
  /// Embed the offline installer and run it.
  /// Does not require an internet connection.
  /// Increases the installer size by around 127MB.
  OfflineInstaller {
    /// Instructs the installer to run the installer in silent mode. Defaults to `true`.
    #[serde(default = "default_true")]
    silent: bool,
  },
  /// Embed a fixed webview2 version and use it at runtime.
  /// Increases the installer size by around 180MB.
  FixedRuntime {
    /// The path to the fixed runtime to use.
    ///
    /// The fixed version can be downloaded [on the official website](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).
    /// The `.cab` file must be extracted to a folder and this folder path must be defined on this field.
    path: PathBuf,
  },
}

impl Default for WebviewInstallMode {
  fn default() -> Self {
    Self::DownloadBootstrapper { silent: true }
  }
}

/// Windows bundler configuration.
///
/// See more: https://tauri.app/v1/api/config#windowsconfig
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowsConfig {
  /// Specifies the file digest algorithm to use for creating file signatures.
  /// Required for code signing. SHA-256 is recommended.
  #[serde(alias = "digest-algorithm")]
  pub digest_algorithm: Option<String>,
  /// Specifies the SHA1 hash of the signing certificate.
  #[serde(alias = "certificate-thumbprint")]
  pub certificate_thumbprint: Option<String>,
  /// Server to use during timestamping.
  #[serde(alias = "timestamp-url")]
  pub timestamp_url: Option<String>,
  /// Whether to use Time-Stamp Protocol (TSP, a.k.a. RFC 3161) for the timestamp server. Your code signing provider may
  /// use a TSP timestamp server, like e.g. SSL.com does. If so, enable TSP by setting to true.
  #[serde(default)]
  pub tsp: bool,
  /// The installation mode for the Webview2 runtime.
  #[serde(default, alias = "webview-install-mode")]
  pub webview_install_mode: WebviewInstallMode,
  /// Path to the webview fixed runtime to use. Overwrites [`Self::webview_install_mode`] if set.
  ///
  /// Will be removed in v2, prefer the [`Self::webview_install_mode`] option.
  ///
  /// The fixed version can be downloaded [on the official website](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).
  /// The `.cab` file must be extracted to a folder and this folder path must be defined on this field.
  #[serde(alias = "webview-fixed-runtime-path")]
  pub webview_fixed_runtime_path: Option<PathBuf>,
  /// Validates a second app installation, blocking the user from installing an older version if set to `false`.
  ///
  /// For instance, if `1.2.1` is installed, the user won't be able to install app version `1.2.0` or `1.1.5`.
  ///
  /// The default value of this flag is `true`.
  #[serde(default = "default_true", alias = "allow-downgrades")]
  pub allow_downgrades: bool,
  /// Configuration for the MSI generated with WiX.
  pub wix: Option<WixConfig>,
  /// Configuration for the installer generated with NSIS.
  pub nsis: Option<NsisConfig>,
}

impl Default for WindowsConfig {
  fn default() -> Self {
    Self {
      digest_algorithm: None,
      certificate_thumbprint: None,
      timestamp_url: None,
      tsp: false,
      webview_install_mode: Default::default(),
      webview_fixed_runtime_path: None,
      allow_downgrades: true,
      wix: None,
      nsis: None,
    }
  }
}

/// Definition for bundle resources.
/// Can be either a list of paths to include or a map of source to target paths.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum BundleResources {
  /// A list of paths to include.
  List(Vec<String>),
  /// A map of source to target paths.
  Map(HashMap<String, String>),
}

/// Configuration for tauri-bundler.
///
/// See more: https://tauri.app/v1/api/config#bundleconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BundleConfig {
  /// Whether Tauri should bundle your application or just output the executable.
  #[serde(default)]
  pub active: bool,
  /// The bundle targets, currently supports ["deb", "appimage", "nsis", "msi", "app", "dmg", "updater"] or "all".
  #[serde(default)]
  pub targets: BundleTarget,
  /// The application identifier in reverse domain name notation (e.g. `com.tauri.example`).
  /// This string must be unique across applications since it is used in system configurations like
  /// the bundle ID and path to the webview data directory.
  /// This string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-),
  /// and periods (.).
  pub identifier: String,
  /// The application's publisher. Defaults to the second element in the identifier string.
  /// Currently maps to the Manufacturer property of the Windows Installer.
  pub publisher: Option<String>,
  /// The app's icons
  #[serde(default)]
  pub icon: Vec<String>,
  /// App resources to bundle.
  /// Each resource is a path to a file or directory.
  /// Glob patterns are supported.
  pub resources: Option<BundleResources>,
  /// A copyright string associated with your application.
  pub copyright: Option<String>,
  /// The application kind.
  ///
  /// Should be one of the following:
  /// Business, DeveloperTool, Education, Entertainment, Finance, Game, ActionGame, AdventureGame, ArcadeGame, BoardGame, CardGame, CasinoGame, DiceGame, EducationalGame, FamilyGame, KidsGame, MusicGame, PuzzleGame, RacingGame, RolePlayingGame, SimulationGame, SportsGame, StrategyGame, TriviaGame, WordGame, GraphicsAndDesign, HealthcareAndFitness, Lifestyle, Medical, Music, News, Photography, Productivity, Reference, SocialNetworking, Sports, Travel, Utility, Video, Weather.
  pub category: Option<String>,
  /// A short description of your application.
  #[serde(alias = "short-description")]
  pub short_description: Option<String>,
  /// A longer, multi-line description of the application.
  #[serde(alias = "long-description")]
  pub long_description: Option<String>,
  /// Configuration for the AppImage bundle.
  #[serde(default)]
  pub appimage: AppImageConfig,
  /// Configuration for the Debian bundle.
  #[serde(default)]
  pub deb: DebConfig,
  /// Configuration for the macOS bundles.
  #[serde(rename = "macOS", default)]
  pub macos: MacConfig,
  /// A list of—either absolute or relative—paths to binaries to embed with your application.
  ///
  /// Note that Tauri will look for system-specific binaries following the pattern "binary-name{-target-triple}{.system-extension}".
  ///
  /// E.g. for the external binary "my-binary", Tauri looks for:
  ///
  /// - "my-binary-x86_64-pc-windows-msvc.exe" for Windows
  /// - "my-binary-x86_64-apple-darwin" for macOS
  /// - "my-binary-x86_64-unknown-linux-gnu" for Linux
  ///
  /// so don't forget to provide binaries for all targeted platforms.
  #[serde(alias = "external-bin")]
  pub external_bin: Option<Vec<String>>,
  /// Configuration for the Windows bundle.
  #[serde(default)]
  pub windows: WindowsConfig,
}

/// A CLI argument definition.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliArg {
  /// The short version of the argument, without the preceding -.
  ///
  /// NOTE: Any leading `-` characters will be stripped, and only the first non-character will be used as the short version.
  pub short: Option<char>,
  /// The unique argument name
  pub name: String,
  /// The argument description which will be shown on the help information.
  /// Typically, this is a short (one line) description of the arg.
  pub description: Option<String>,
  /// The argument long description which will be shown on the help information.
  /// Typically this a more detailed (multi-line) message that describes the argument.
  #[serde(alias = "long-description")]
  pub long_description: Option<String>,
  /// Specifies that the argument takes a value at run time.
  ///
  /// NOTE: values for arguments may be specified in any of the following methods
  /// - Using a space such as -o value or --option value
  /// - Using an equals and no space such as -o=value or --option=value
  /// - Use a short and no space such as -ovalue
  #[serde(default, alias = "takes-value")]
  pub takes_value: bool,
  /// Specifies that the argument may have an unknown number of multiple values. Without any other settings, this argument may appear only once.
  ///
  /// For example, --opt val1 val2 is allowed, but --opt val1 val2 --opt val3 is not.
  ///
  /// NOTE: Setting this requires `takes_value` to be set to true.
  #[serde(default)]
  pub multiple: bool,
  /// Specifies that the argument may appear more than once.
  /// For flags, this results in the number of occurrences of the flag being recorded. For example -ddd or -d -d -d would count as three occurrences.
  /// For options or arguments that take a value, this does not affect how many values they can accept. (i.e. only one at a time is allowed)
  ///
  /// For example, --opt val1 --opt val2 is allowed, but --opt val1 val2 is not.
  #[serde(default, alias = "multiple-occurrences")]
  pub multiple_occurrences: bool,
  /// Specifies how many values are required to satisfy this argument. For example, if you had a
  /// `-f <file>` argument where you wanted exactly 3 'files' you would set
  /// `number_of_values = 3`, and this argument wouldn't be satisfied unless the user provided
  /// 3 and only 3 values.
  ///
  /// **NOTE:** Does *not* require `multiple_occurrences = true` to be set. Setting
  /// `multiple_occurrences = true` would allow `-f <file> <file> <file> -f <file> <file> <file>` where
  /// as *not* setting it would only allow one occurrence of this argument.
  ///
  /// **NOTE:** implicitly sets `takes_value = true` and `multiple_values = true`.
  #[serde(alias = "number-of-values")]
  pub number_of_values: Option<usize>,
  /// Specifies a list of possible values for this argument.
  /// At runtime, the CLI verifies that only one of the specified values was used, or fails with an error message.
  #[serde(alias = "possible-values")]
  pub possible_values: Option<Vec<String>>,
  /// Specifies the minimum number of values for this argument.
  /// For example, if you had a -f `<file>` argument where you wanted at least 2 'files',
  /// you would set `minValues: 2`, and this argument would be satisfied if the user provided, 2 or more values.
  #[serde(alias = "min-values")]
  pub min_values: Option<usize>,
  /// Specifies the maximum number of values are for this argument.
  /// For example, if you had a -f `<file>` argument where you wanted up to 3 'files',
  /// you would set .max_values(3), and this argument would be satisfied if the user provided, 1, 2, or 3 values.
  #[serde(alias = "max-values")]
  pub max_values: Option<usize>,
  /// Sets whether or not the argument is required by default.
  ///
  /// - Required by default means it is required, when no other conflicting rules have been evaluated
  /// - Conflicting rules take precedence over being required.
  #[serde(default)]
  pub required: bool,
  /// Sets an arg that override this arg's required setting
  /// i.e. this arg will be required unless this other argument is present.
  #[serde(alias = "required-unless-present")]
  pub required_unless_present: Option<String>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless all these other arguments are present.
  #[serde(alias = "required-unless-present-all")]
  pub required_unless_present_all: Option<Vec<String>>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless at least one of these other arguments are present.
  #[serde(alias = "required-unless-present-any")]
  pub required_unless_present_any: Option<Vec<String>>,
  /// Sets a conflicting argument by name
  /// i.e. when using this argument, the following argument can't be present and vice versa.
  #[serde(alias = "conflicts-with")]
  pub conflicts_with: Option<String>,
  /// The same as conflictsWith but allows specifying multiple two-way conflicts per argument.
  #[serde(alias = "conflicts-with-all")]
  pub conflicts_with_all: Option<Vec<String>>,
  /// Tets an argument by name that is required when this one is present
  /// i.e. when using this argument, the following argument must be present.
  pub requires: Option<String>,
  /// Sts multiple arguments by names that are required when this one is present
  /// i.e. when using this argument, the following arguments must be present.
  #[serde(alias = "requires-all")]
  pub requires_all: Option<Vec<String>>,
  /// Allows a conditional requirement with the signature [arg, value]
  /// the requirement will only become valid if `arg`'s value equals `${value}`.
  #[serde(alias = "requires-if")]
  pub requires_if: Option<Vec<String>>,
  /// Allows specifying that an argument is required conditionally with the signature [arg, value]
  /// the requirement will only become valid if the `arg`'s value equals `${value}`.
  #[serde(alias = "requires-if-eq")]
  pub required_if_eq: Option<Vec<String>>,
  /// Requires that options use the --option=val syntax
  /// i.e. an equals between the option and associated value.
  #[serde(alias = "requires-equals")]
  pub require_equals: Option<bool>,
  /// The positional argument index, starting at 1.
  ///
  /// The index refers to position according to other positional argument.
  /// It does not define position in the argument list as a whole. When utilized with multiple=true,
  /// only the last positional argument may be defined as multiple (i.e. the one with the highest index).
  #[cfg_attr(feature = "schema", validate(range(min = 1)))]
  pub index: Option<usize>,
}

/// describes a CLI configuration
///
/// See more: https://tauri.app/v1/api/config#cliconfig
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliConfig {
  /// Command description which will be shown on the help information.
  pub description: Option<String>,
  /// Command long description which will be shown on the help information.
  #[serde(alias = "long-description")]
  pub long_description: Option<String>,
  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed before the auto-generated help information.
  /// This is often used for header information.
  #[serde(alias = "before-help")]
  pub before_help: Option<String>,
  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed after the auto-generated help information.
  /// This is often used to describe how to use the arguments, or caveats to be noted.
  #[serde(alias = "after-help")]
  pub after_help: Option<String>,
  /// List of arguments for the command
  pub args: Option<Vec<CliArg>>,
  /// List of subcommands of this command
  pub subcommands: Option<HashMap<String, CliConfig>>,
}

/// The window configuration object.
///
/// See more: https://tauri.app/v1/api/config#windowconfig
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowConfig {
  /// The window identifier. It must be alphanumeric.
  #[serde(default = "default_window_label")]
  pub label: String,
  /// The window webview URL.
  #[serde(default)]
  pub url: WindowUrl,
  /// The user agent for the webview
  #[serde(alias = "user-agent")]
  pub user_agent: Option<String>,
  /// Whether the file drop is enabled or not on the webview. By default it is enabled.
  ///
  /// Disabling it is required to use drag and drop on the frontend on Windows.
  #[serde(default = "default_true", alias = "file-drop-enabled")]
  pub file_drop_enabled: bool,
  /// Whether or not the window starts centered or not.
  #[serde(default)]
  pub center: bool,
  /// The horizontal position of the window's top left corner
  pub x: Option<f64>,
  /// The vertical position of the window's top left corner
  pub y: Option<f64>,
  /// The window width.
  #[serde(default = "default_width")]
  pub width: f64,
  /// The window height.
  #[serde(default = "default_height")]
  pub height: f64,
  /// The min window width.
  #[serde(alias = "min-width")]
  pub min_width: Option<f64>,
  /// The min window height.
  #[serde(alias = "min-height")]
  pub min_height: Option<f64>,
  /// The max window width.
  #[serde(alias = "max-width")]
  pub max_width: Option<f64>,
  /// The max window height.
  #[serde(alias = "max-height")]
  pub max_height: Option<f64>,
  /// Whether the window is resizable or not. When resizable is set to false, native window's maximize button is automatically disabled.
  #[serde(default = "default_true")]
  pub resizable: bool,
  /// Whether the window's native maximize button is enabled or not.
  /// If resizable is set to false, this setting is ignored.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
  /// - **Linux / iOS / Android:** Unsupported.
  #[serde(default = "default_true")]
  pub maximizable: bool,
  /// Whether the window's native minimize button is enabled or not.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  #[serde(default = "default_true")]
  pub minimizable: bool,
  /// Whether the window's native close button is enabled or not.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** "GTK+ will do its best to convince the window manager not to show a close button.
  ///   Depending on the system, this function may not have any effect when called on a window that is already visible"
  /// - **iOS / Android:** Unsupported.
  #[serde(default = "default_true")]
  pub closable: bool,
  /// The window title.
  #[serde(default = "default_title")]
  pub title: String,
  /// Whether the window starts as fullscreen or not.
  #[serde(default)]
  pub fullscreen: bool,
  /// Whether the window will be initially focused or not.
  #[serde(default = "default_true")]
  pub focus: bool,
  /// Whether the window is transparent or not.
  ///
  /// Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri > macOSPrivateApi`.
  /// WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
  #[serde(default)]
  pub transparent: bool,
  /// Whether the window is maximized or not.
  #[serde(default)]
  pub maximized: bool,
  /// Whether the window is visible or not.
  #[serde(default = "default_true")]
  pub visible: bool,
  /// Whether the window should have borders and bars.
  #[serde(default = "default_true")]
  pub decorations: bool,
  /// Whether the window should always be on top of other windows.
  #[serde(default, alias = "always-on-top")]
  pub always_on_top: bool,
  /// Prevents the window contents from being captured by other apps.
  #[serde(default, alias = "content-protected")]
  pub content_protected: bool,
  /// If `true`, hides the window icon from the taskbar on Windows and Linux.
  #[serde(default, alias = "skip-taskbar")]
  pub skip_taskbar: bool,
  /// The initial window theme. Defaults to the system theme. Only implemented on Windows and macOS 10.14+.
  pub theme: Option<Theme>,
  /// The style of the macOS title bar.
  #[serde(default, alias = "title-bar-style")]
  pub title_bar_style: TitleBarStyle,
  /// If `true`, sets the window title to be hidden on macOS.
  #[serde(default, alias = "hidden-title")]
  pub hidden_title: bool,
  /// Whether clicking an inactive window also clicks through to the webview on macOS.
  #[serde(default, alias = "accept-first-mouse")]
  pub accept_first_mouse: bool,
  /// Defines the window [tabbing identifier] for macOS.
  ///
  /// Windows with matching tabbing identifiers will be grouped together.
  /// If the tabbing identifier is not set, automatic tabbing will be disabled.
  ///
  /// [tabbing identifier]: <https://developer.apple.com/documentation/appkit/nswindow/1644704-tabbingidentifier>
  #[serde(default, alias = "tabbing-identifier")]
  pub tabbing_identifier: Option<String>,
  /// Defines additional browser arguments on Windows. By default wry passes `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`
  /// so if you use this method, you also need to disable these components by yourself if you want.
  #[serde(default, alias = "additional-browser-args")]
  pub additional_browser_args: Option<String>,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      label: default_window_label(),
      url: WindowUrl::default(),
      user_agent: None,
      file_drop_enabled: true,
      center: false,
      x: None,
      y: None,
      width: default_width(),
      height: default_height(),
      min_width: None,
      min_height: None,
      max_width: None,
      max_height: None,
      resizable: true,
      maximizable: true,
      minimizable: true,
      closable: true,
      title: default_title(),
      fullscreen: false,
      focus: false,
      transparent: false,
      maximized: false,
      visible: true,
      decorations: true,
      always_on_top: false,
      content_protected: false,
      skip_taskbar: false,
      theme: None,
      title_bar_style: Default::default(),
      hidden_title: false,
      accept_first_mouse: false,
      tabbing_identifier: None,
      additional_browser_args: None,
    }
  }
}

fn default_window_label() -> String {
  "main".to_string()
}

fn default_width() -> f64 {
  800f64
}

fn default_height() -> f64 {
  600f64
}

fn default_title() -> String {
  "Tauri App".to_string()
}

/// A Content-Security-Policy directive source list.
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy/Sources#sources>.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum CspDirectiveSources {
  /// An inline list of CSP sources. Same as [`Self::List`], but concatenated with a space separator.
  Inline(String),
  /// A list of CSP sources. The collection will be concatenated with a space separator for the CSP string.
  List(Vec<String>),
}

impl Default for CspDirectiveSources {
  fn default() -> Self {
    Self::List(Vec::new())
  }
}

impl From<CspDirectiveSources> for Vec<String> {
  fn from(sources: CspDirectiveSources) -> Self {
    match sources {
      CspDirectiveSources::Inline(source) => source.split(' ').map(|s| s.to_string()).collect(),
      CspDirectiveSources::List(l) => l,
    }
  }
}

impl CspDirectiveSources {
  /// Whether the given source is configured on this directive or not.
  pub fn contains(&self, source: &str) -> bool {
    match self {
      Self::Inline(s) => s.contains(&format!("{source} ")) || s.contains(&format!(" {source}")),
      Self::List(l) => l.contains(&source.into()),
    }
  }

  /// Appends the given source to this directive.
  pub fn push<S: AsRef<str>>(&mut self, source: S) {
    match self {
      Self::Inline(s) => {
        s.push(' ');
        s.push_str(source.as_ref());
      }
      Self::List(l) => {
        l.push(source.as_ref().to_string());
      }
    }
  }
}

/// A Content-Security-Policy definition.
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum Csp {
  /// The entire CSP policy in a single text string.
  Policy(String),
  /// An object mapping a directive with its sources values as a list of strings.
  DirectiveMap(HashMap<String, CspDirectiveSources>),
}

impl From<HashMap<String, CspDirectiveSources>> for Csp {
  fn from(map: HashMap<String, CspDirectiveSources>) -> Self {
    Self::DirectiveMap(map)
  }
}

impl From<Csp> for HashMap<String, CspDirectiveSources> {
  fn from(csp: Csp) -> Self {
    match csp {
      Csp::Policy(policy) => {
        let mut map = HashMap::new();
        for directive in policy.split(';') {
          let mut tokens = directive.trim().split(' ');
          if let Some(directive) = tokens.next() {
            let sources = tokens.map(|s| s.to_string()).collect::<Vec<String>>();
            map.insert(directive.to_string(), CspDirectiveSources::List(sources));
          }
        }
        map
      }
      Csp::DirectiveMap(m) => m,
    }
  }
}

impl Display for Csp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Policy(s) => write!(f, "{s}"),
      Self::DirectiveMap(m) => {
        let len = m.len();
        let mut i = 0;
        for (directive, sources) in m {
          let sources: Vec<String> = sources.clone().into();
          write!(f, "{} {}", directive, sources.join(" "))?;
          i += 1;
          if i != len {
            write!(f, "; ")?;
          }
        }
        Ok(())
      }
    }
  }
}

/// The possible values for the `dangerous_disable_asset_csp_modification` config option.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DisabledCspModificationKind {
  /// If `true`, disables all CSP modification.
  /// `false` is the default value and it configures Tauri to control the CSP.
  Flag(bool),
  /// Disables the given list of CSP directives modifications.
  List(Vec<String>),
}

impl Default for DisabledCspModificationKind {
  fn default() -> Self {
    Self::Flag(false)
  }
}

/// External command access definition.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteDomainAccessScope {
  /// The URL scheme to allow. By default, all schemas are allowed.
  pub scheme: Option<String>,
  /// The domain to allow.
  pub domain: String,
  /// The list of window labels this scope applies to.
  pub windows: Vec<String>,
  /// The list of plugins that are allowed in this scope.
  /// The names should be without the `tauri-plugin-` prefix, for example `"store"` for `tauri-plugin-store`.
  #[serde(default)]
  pub plugins: Vec<String>,
  /// Enables access to the Tauri API.
  #[serde(default, rename = "enableTauriAPI", alias = "enable-tauri-api")]
  pub enable_tauri_api: bool,
}

/// Security configuration.
///
/// See more: https://tauri.app/v1/api/config#securityconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityConfig {
  /// The Content Security Policy that will be injected on all HTML files on the built application.
  /// If [`dev_csp`](#SecurityConfig.devCsp) is not specified, this value is also injected on dev.
  ///
  /// This is a really important part of the configuration since it helps you ensure your WebView is secured.
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
  pub csp: Option<Csp>,
  /// The Content Security Policy that will be injected on all HTML files on development.
  ///
  /// This is a really important part of the configuration since it helps you ensure your WebView is secured.
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
  #[serde(alias = "dev-csp")]
  pub dev_csp: Option<Csp>,
  /// Freeze the `Object.prototype` when using the custom protocol.
  #[serde(default, alias = "freeze-prototype")]
  pub freeze_prototype: bool,
  /// Disables the Tauri-injected CSP sources.
  ///
  /// At compile time, Tauri parses all the frontend assets and changes the Content-Security-Policy
  /// to only allow loading of your own scripts and styles by injecting nonce and hash sources.
  /// This stricts your CSP, which may introduce issues when using along with other flexing sources.
  ///
  /// This configuration option allows both a boolean and a list of strings as value.
  /// A boolean instructs Tauri to disable the injection for all CSP injections,
  /// and a list of strings indicates the CSP directives that Tauri cannot inject.
  ///
  /// **WARNING:** Only disable this if you know what you are doing and have properly configured the CSP.
  /// Your application might be vulnerable to XSS attacks without this Tauri protection.
  #[serde(default, alias = "dangerous-disable-asset-csp-modification")]
  pub dangerous_disable_asset_csp_modification: DisabledCspModificationKind,
  /// Allow external domains to send command to Tauri.
  ///
  /// By default, external domains do not have access to `window.__TAURI__`, which means they cannot
  /// communicate with the commands defined in Rust. This prevents attacks where an externally
  /// loaded malicious or compromised sites could start executing commands on the user's device.
  ///
  /// This configuration allows a set of external domains to have access to the Tauri commands.
  /// When you configure a domain to be allowed to access the IPC, all subpaths are allowed. Subdomains are not allowed.
  ///
  /// **WARNING:** Only use this option if you either have internal checks against malicious
  /// external sites or you can trust the allowed external sites. You application might be
  /// vulnerable to dangerous Tauri command related attacks otherwise.
  #[serde(default, alias = "dangerous-remote-domain-ipc-access")]
  pub dangerous_remote_domain_ipc_access: Vec<RemoteDomainAccessScope>,
  /// Sets whether the custom protocols should use `http://<scheme>.localhost` instead of the default `https://<scheme>.localhost` on Windows.
  ///
  /// **WARNING:** Using a `http` scheme will allow mixed content when trying to fetch `http` endpoints and is therefore less secure but will match the behavior of the `<scheme>://localhost` protocols used on macOS and Linux.
  #[serde(default, alias = "dangerous-use-http-scheme")]
  pub dangerous_use_http_scheme: bool,
}

/// Defines an allowlist type.
pub trait Allowlist {
  /// Returns all features associated with the allowlist struct.
  fn all_features() -> Vec<&'static str>;
  /// Returns the tauri features enabled on this allowlist.
  fn to_features(&self) -> Vec<&'static str>;
}

macro_rules! check_feature {
  ($self:ident, $features:ident, $flag:ident, $feature_name: expr) => {
    if $self.$flag {
      $features.push($feature_name)
    }
  };
}

/// Filesystem scope definition.
/// It is a list of glob patterns that restrict the API access from the webview.
///
/// Each pattern can start with a variable that resolves to a system base directory.
/// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
/// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
/// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
/// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FsAllowlistScope {
  /// A list of paths that are allowed by this scope.
  AllowedPaths(Vec<PathBuf>),
  /// A complete scope configuration.
  #[serde(rename_all = "camelCase")]
  Scope {
    /// A list of paths that are allowed by this scope.
    #[serde(default)]
    allow: Vec<PathBuf>,
    /// A list of paths that are not allowed by this scope.
    /// This gets precedence over the [`Self::Scope::allow`] list.
    #[serde(default)]
    deny: Vec<PathBuf>,
    /// Whether or not paths that contain components that start with a `.`
    /// will require that `.` appears literally in the pattern; `*`, `?`, `**`,
    /// or `[...]` will not match. This is useful because such files are
    /// conventionally considered hidden on Unix systems and it might be
    /// desirable to skip them when listing files.
    ///
    /// Defaults to `true` on Unix systems and `false` on Windows
    // dotfiles are not supposed to be exposed by default on unix
    #[serde(alias = "require-literal-leading-dot")]
    require_literal_leading_dot: Option<bool>,
  },
}

impl Default for FsAllowlistScope {
  fn default() -> Self {
    Self::AllowedPaths(Vec::new())
  }
}

/// Allowlist for the file system APIs.
///
/// See more: https://tauri.app/v1/api/config#fsallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsAllowlistConfig {
  /// The access scope for the filesystem APIs.
  #[serde(default)]
  pub scope: FsAllowlistScope,
  /// Use this flag to enable all file system API features.
  #[serde(default)]
  pub all: bool,
  /// Read file from local filesystem.
  #[serde(default, alias = "read-file")]
  pub read_file: bool,
  /// Write file to local filesystem.
  #[serde(default, alias = "write-file")]
  pub write_file: bool,
  /// Read directory from local filesystem.
  #[serde(default, alias = "read-dir")]
  pub read_dir: bool,
  /// Copy file from local filesystem.
  #[serde(default, alias = "copy-file")]
  pub copy_file: bool,
  /// Create directory from local filesystem.
  #[serde(default, alias = "create-dir")]
  pub create_dir: bool,
  /// Remove directory from local filesystem.
  #[serde(default, alias = "remove-dir")]
  pub remove_dir: bool,
  /// Remove file from local filesystem.
  #[serde(default, alias = "remove-file")]
  pub remove_file: bool,
  /// Rename file from local filesystem.
  #[serde(default, alias = "rename-file")]
  pub rename_file: bool,
  /// Check if path exists on the local filesystem.
  #[serde(default)]
  pub exists: bool,
}

impl Allowlist for FsAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      scope: Default::default(),
      all: false,
      read_file: true,
      write_file: true,
      read_dir: true,
      copy_file: true,
      create_dir: true,
      remove_dir: true,
      remove_file: true,
      rename_file: true,
      exists: true,
    };
    let mut features = allowlist.to_features();
    features.push("fs-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["fs-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, read_file, "fs-read-file");
      check_feature!(self, features, write_file, "fs-write-file");
      check_feature!(self, features, read_dir, "fs-read-dir");
      check_feature!(self, features, copy_file, "fs-copy-file");
      check_feature!(self, features, create_dir, "fs-create-dir");
      check_feature!(self, features, remove_dir, "fs-remove-dir");
      check_feature!(self, features, remove_file, "fs-remove-file");
      check_feature!(self, features, rename_file, "fs-rename-file");
      check_feature!(self, features, exists, "fs-exists");
      features
    }
  }
}

/// Allowlist for the window APIs.
///
/// See more: https://tauri.app/v1/api/config#windowallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowAllowlistConfig {
  /// Use this flag to enable all window API features.
  #[serde(default)]
  pub all: bool,
  /// Allows dynamic window creation.
  #[serde(default)]
  pub create: bool,
  /// Allows centering the window.
  #[serde(default)]
  pub center: bool,
  /// Allows requesting user attention on the window.
  #[serde(default, alias = "request-user-attention")]
  pub request_user_attention: bool,
  /// Allows setting the resizable flag of the window.
  #[serde(default, alias = "set-resizable")]
  pub set_resizable: bool,
  /// Allows setting whether the window's native maximize button is enabled or not.
  #[serde(default, alias = "set-maximizable")]
  pub set_maximizable: bool,
  /// Allows setting whether the window's native minimize button is enabled or not.
  #[serde(default, alias = "set-minimizable")]
  pub set_minimizable: bool,
  /// Allows setting whether the window's native close button is enabled or not.
  #[serde(default, alias = "set-closable")]
  pub set_closable: bool,
  /// Allows changing the window title.
  #[serde(default, alias = "set-title")]
  pub set_title: bool,
  /// Allows maximizing the window.
  #[serde(default)]
  pub maximize: bool,
  /// Allows unmaximizing the window.
  #[serde(default)]
  pub unmaximize: bool,
  /// Allows minimizing the window.
  #[serde(default)]
  pub minimize: bool,
  /// Allows unminimizing the window.
  #[serde(default)]
  pub unminimize: bool,
  /// Allows showing the window.
  #[serde(default)]
  pub show: bool,
  /// Allows hiding the window.
  #[serde(default)]
  pub hide: bool,
  /// Allows closing the window.
  #[serde(default)]
  pub close: bool,
  /// Allows setting the decorations flag of the window.
  #[serde(default, alias = "set-decorations")]
  pub set_decorations: bool,
  /// Allows setting the always_on_top flag of the window.
  #[serde(default, alias = "set-always-on-top")]
  pub set_always_on_top: bool,
  /// Allows preventing the window contents from being captured by other apps.
  #[serde(default, alias = "set-content-protected")]
  pub set_content_protected: bool,
  /// Allows setting the window size.
  #[serde(default, alias = "set-size")]
  pub set_size: bool,
  /// Allows setting the window minimum size.
  #[serde(default, alias = "set-min-size")]
  pub set_min_size: bool,
  /// Allows setting the window maximum size.
  #[serde(default, alias = "set-max-size")]
  pub set_max_size: bool,
  /// Allows changing the position of the window.
  #[serde(default, alias = "set-position")]
  pub set_position: bool,
  /// Allows setting the fullscreen flag of the window.
  #[serde(default, alias = "set-fullscreen")]
  pub set_fullscreen: bool,
  /// Allows focusing the window.
  #[serde(default, alias = "set-focus")]
  pub set_focus: bool,
  /// Allows changing the window icon.
  #[serde(default, alias = "set-icon")]
  pub set_icon: bool,
  /// Allows setting the skip_taskbar flag of the window.
  #[serde(default, alias = "set-skip-taskbar")]
  pub set_skip_taskbar: bool,
  /// Allows grabbing the cursor.
  #[serde(default, alias = "set-cursor-grab")]
  pub set_cursor_grab: bool,
  /// Allows setting the cursor visibility.
  #[serde(default, alias = "set-cursor-visible")]
  pub set_cursor_visible: bool,
  /// Allows changing the cursor icon.
  #[serde(default, alias = "set-cursor-icon")]
  pub set_cursor_icon: bool,
  /// Allows setting the cursor position.
  #[serde(default, alias = "set-cursor-position")]
  pub set_cursor_position: bool,
  /// Allows ignoring cursor events.
  #[serde(default, alias = "set-ignore-cursor-events")]
  pub set_ignore_cursor_events: bool,
  /// Allows start dragging on the window.
  #[serde(default, alias = "start-dragging")]
  pub start_dragging: bool,
  /// Allows opening the system dialog to print the window content.
  #[serde(default)]
  pub print: bool,
}

impl Allowlist for WindowAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      all: false,
      create: true,
      center: true,
      request_user_attention: true,
      set_resizable: true,
      set_maximizable: true,
      set_minimizable: true,
      set_closable: true,
      set_title: true,
      maximize: true,
      unmaximize: true,
      minimize: true,
      unminimize: true,
      show: true,
      hide: true,
      close: true,
      set_decorations: true,
      set_always_on_top: true,
      set_content_protected: false,
      set_size: true,
      set_min_size: true,
      set_max_size: true,
      set_position: true,
      set_fullscreen: true,
      set_focus: true,
      set_icon: true,
      set_skip_taskbar: true,
      set_cursor_grab: true,
      set_cursor_visible: true,
      set_cursor_icon: true,
      set_cursor_position: true,
      set_ignore_cursor_events: true,
      start_dragging: true,
      print: true,
    };
    let mut features = allowlist.to_features();
    features.push("window-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["window-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, create, "window-create");
      check_feature!(self, features, center, "window-center");
      check_feature!(
        self,
        features,
        request_user_attention,
        "window-request-user-attention"
      );
      check_feature!(self, features, set_resizable, "window-set-resizable");
      check_feature!(self, features, set_maximizable, "window-set-maximizable");
      check_feature!(self, features, set_minimizable, "window-set-minimizable");
      check_feature!(self, features, set_closable, "window-set-closable");
      check_feature!(self, features, set_title, "window-set-title");
      check_feature!(self, features, maximize, "window-maximize");
      check_feature!(self, features, unmaximize, "window-unmaximize");
      check_feature!(self, features, minimize, "window-minimize");
      check_feature!(self, features, unminimize, "window-unminimize");
      check_feature!(self, features, show, "window-show");
      check_feature!(self, features, hide, "window-hide");
      check_feature!(self, features, close, "window-close");
      check_feature!(self, features, set_decorations, "window-set-decorations");
      check_feature!(
        self,
        features,
        set_always_on_top,
        "window-set-always-on-top"
      );
      check_feature!(
        self,
        features,
        set_content_protected,
        "window-set-content-protected"
      );
      check_feature!(self, features, set_size, "window-set-size");
      check_feature!(self, features, set_min_size, "window-set-min-size");
      check_feature!(self, features, set_max_size, "window-set-max-size");
      check_feature!(self, features, set_position, "window-set-position");
      check_feature!(self, features, set_fullscreen, "window-set-fullscreen");
      check_feature!(self, features, set_focus, "window-set-focus");
      check_feature!(self, features, set_icon, "window-set-icon");
      check_feature!(self, features, set_skip_taskbar, "window-set-skip-taskbar");
      check_feature!(self, features, set_cursor_grab, "window-set-cursor-grab");
      check_feature!(
        self,
        features,
        set_cursor_visible,
        "window-set-cursor-visible"
      );
      check_feature!(self, features, set_cursor_icon, "window-set-cursor-icon");
      check_feature!(
        self,
        features,
        set_cursor_position,
        "window-set-cursor-position"
      );
      check_feature!(
        self,
        features,
        set_ignore_cursor_events,
        "window-set-ignore-cursor-events"
      );
      check_feature!(self, features, start_dragging, "window-start-dragging");
      check_feature!(self, features, print, "window-print");
      features
    }
  }
}

/// A command allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShellAllowedCommand {
  /// The name for this allowed shell command configuration.
  ///
  /// This name will be used inside of the webview API to call this command along with
  /// any specified arguments.
  pub name: String,

  /// The command name.
  /// It can start with a variable that resolves to a system base directory.
  /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
  /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
  /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
  /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
  #[serde(rename = "cmd", default)] // use default just so the schema doesn't flag it as required
  pub command: PathBuf,

  /// The allowed arguments for the command execution.
  #[serde(default)]
  pub args: ShellAllowedArgs,

  /// If this command is a sidecar command.
  #[serde(default)]
  pub sidecar: bool,
}

impl<'de> Deserialize<'de> for ShellAllowedCommand {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct InnerShellAllowedCommand {
      name: String,
      #[serde(rename = "cmd")]
      command: Option<PathBuf>,
      #[serde(default)]
      args: ShellAllowedArgs,
      #[serde(default)]
      sidecar: bool,
    }

    let config = InnerShellAllowedCommand::deserialize(deserializer)?;

    if !config.sidecar && config.command.is_none() {
      return Err(DeError::custom(
        "The shell scope `command` value is required.",
      ));
    }

    Ok(ShellAllowedCommand {
      name: config.name,
      command: config.command.unwrap_or_default(),
      args: config.args,
      sidecar: config.sidecar,
    })
  }
}

/// A set of command arguments allowed to be executed by the webview API.
///
/// A value of `true` will allow any arguments to be passed to the command. `false` will disable all
/// arguments. A list of [`ShellAllowedArg`] will set those arguments as the only valid arguments to
/// be passed to the attached command configuration.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArgs {
  /// Use a simple boolean to allow all or disable all arguments to this command configuration.
  Flag(bool),

  /// A specific set of [`ShellAllowedArg`] that are valid to call for the command configuration.
  List(Vec<ShellAllowedArg>),
}

impl Default for ShellAllowedArgs {
  fn default() -> Self {
    Self::Flag(false)
  }
}

/// A command argument allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArg {
  /// A non-configurable argument that is passed to the command in the order it was specified.
  Fixed(String),

  /// A variable that is set while calling the command from the webview API.
  ///
  Var {
    /// [regex] validator to require passed values to conform to an expected input.
    ///
    /// This will require the argument value passed to this variable to match the `validator` regex
    /// before it will be executed.
    ///
    /// [regex]: https://docs.rs/regex/latest/regex/#syntax
    validator: String,
  },
}

/// Shell scope definition.
/// It is a list of command names and associated CLI arguments that restrict the API access from the webview.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ShellAllowlistScope(pub Vec<ShellAllowedCommand>);

/// Defines the `shell > open` api scope.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowlistOpen {
  /// If the shell open API should be enabled.
  ///
  /// If enabled, the default validation regex (`^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`) is used.
  Flag(bool),

  /// Enable the shell open API, with a custom regex that the opened path must match against.
  ///
  /// If using a custom regex to support a non-http(s) schema, care should be used to prevent values
  /// that allow flag-like strings to pass validation. e.g. `--enable-debugging`, `-i`, `/R`.
  Validate(String),
}

impl Default for ShellAllowlistOpen {
  fn default() -> Self {
    Self::Flag(false)
  }
}

/// Allowlist for the shell APIs.
///
/// See more: https://tauri.app/v1/api/config#shellallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellAllowlistConfig {
  /// Access scope for the binary execution APIs.
  /// Sidecars are automatically enabled.
  #[serde(default)]
  pub scope: ShellAllowlistScope,
  /// Use this flag to enable all shell API features.
  #[serde(default)]
  pub all: bool,
  /// Enable binary execution.
  #[serde(default)]
  pub execute: bool,
  /// Enable sidecar execution, allowing the JavaScript layer to spawn a sidecar command,
  /// an executable that is shipped with the application.
  /// For more information see <https://tauri.app/v1/guides/building/sidecar>.
  #[serde(default)]
  pub sidecar: bool,
  /// Open URL with the user's default application.
  #[serde(default)]
  pub open: ShellAllowlistOpen,
}

impl Allowlist for ShellAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      scope: Default::default(),
      all: false,
      execute: true,
      sidecar: true,
      open: ShellAllowlistOpen::Flag(true),
    };
    let mut features = allowlist.to_features();
    features.push("shell-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["shell-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, execute, "shell-execute");
      check_feature!(self, features, sidecar, "shell-sidecar");

      if !matches!(self.open, ShellAllowlistOpen::Flag(false)) {
        features.push("shell-open")
      }

      features
    }
  }
}

/// Allowlist for the dialog APIs.
///
/// See more: https://tauri.app/v1/api/config#dialogallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DialogAllowlistConfig {
  /// Use this flag to enable all dialog API features.
  #[serde(default)]
  pub all: bool,
  /// Allows the API to open a dialog window to pick files.
  #[serde(default)]
  pub open: bool,
  /// Allows the API to open a dialog window to pick where to save files.
  #[serde(default)]
  pub save: bool,
  /// Allows the API to show a message dialog window.
  #[serde(default)]
  pub message: bool,
  /// Allows the API to show a dialog window with Yes/No buttons.
  #[serde(default)]
  pub ask: bool,
  /// Allows the API to show a dialog window with Ok/Cancel buttons.
  #[serde(default)]
  pub confirm: bool,
}

impl Allowlist for DialogAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      all: false,
      open: true,
      save: true,
      message: true,
      ask: true,
      confirm: true,
    };
    let mut features = allowlist.to_features();
    features.push("dialog-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["dialog-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, open, "dialog-open");
      check_feature!(self, features, save, "dialog-save");
      check_feature!(self, features, message, "dialog-message");
      check_feature!(self, features, ask, "dialog-ask");
      check_feature!(self, features, confirm, "dialog-confirm");
      features
    }
  }
}

/// HTTP API scope definition.
/// It is a list of URLs that can be accessed by the webview when using the HTTP APIs.
/// The scoped URL is matched against the request URL using a glob pattern.
///
/// Examples:
/// - "https://*": allows all HTTPS urls
/// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
/// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
#[allow(rustdoc::bare_urls)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
// TODO: in v2, parse into a String or a custom type that perserves the
// glob string because Url type will add a trailing slash
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HttpAllowlistScope(pub Vec<Url>);

/// Allowlist for the HTTP APIs.
///
/// See more: https://tauri.app/v1/api/config#httpallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpAllowlistConfig {
  /// The access scope for the HTTP APIs.
  #[serde(default)]
  pub scope: HttpAllowlistScope,
  /// Use this flag to enable all HTTP API features.
  #[serde(default)]
  pub all: bool,
  /// Allows making HTTP requests.
  #[serde(default)]
  pub request: bool,
}

impl Allowlist for HttpAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      scope: Default::default(),
      all: false,
      request: true,
    };
    let mut features = allowlist.to_features();
    features.push("http-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["http-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, request, "http-request");
      features
    }
  }
}

/// Allowlist for the notification APIs.
///
/// See more: https://tauri.app/v1/api/config#notificationallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationAllowlistConfig {
  /// Use this flag to enable all notification API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for NotificationAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self { all: false };
    let mut features = allowlist.to_features();
    features.push("notification-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["notification-all"]
    } else {
      vec![]
    }
  }
}

/// Allowlist for the global shortcut APIs.
///
/// See more: https://tauri.app/v1/api/config#globalshortcutallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlobalShortcutAllowlistConfig {
  /// Use this flag to enable all global shortcut API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for GlobalShortcutAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self { all: false };
    let mut features = allowlist.to_features();
    features.push("global-shortcut-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["global-shortcut-all"]
    } else {
      vec![]
    }
  }
}

/// Allowlist for the OS APIs.
///
/// See more: https://tauri.app/v1/api/config#osallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OsAllowlistConfig {
  /// Use this flag to enable all OS API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for OsAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self { all: false };
    let mut features = allowlist.to_features();
    features.push("os-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["os-all"]
    } else {
      vec![]
    }
  }
}

/// Allowlist for the path APIs.
///
/// See more: https://tauri.app/v1/api/config#pathallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathAllowlistConfig {
  /// Use this flag to enable all path API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for PathAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self { all: false };
    let mut features = allowlist.to_features();
    features.push("path-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["path-all"]
    } else {
      vec![]
    }
  }
}

/// Allowlist for the custom protocols.
///
/// See more: https://tauri.app/v1/api/config#protocolallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProtocolAllowlistConfig {
  /// The access scope for the asset protocol.
  #[serde(default, alias = "asset-scope")]
  pub asset_scope: FsAllowlistScope,
  /// Use this flag to enable all custom protocols.
  #[serde(default)]
  pub all: bool,
  /// Enables the asset protocol.
  #[serde(default)]
  pub asset: bool,
}

impl Allowlist for ProtocolAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      asset_scope: Default::default(),
      all: false,
      asset: true,
    };
    let mut features = allowlist.to_features();
    features.push("protocol-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["protocol-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, asset, "protocol-asset");
      features
    }
  }
}

/// Allowlist for the process APIs.
///
/// See more: https://tauri.app/v1/api/config#processallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessAllowlistConfig {
  /// Use this flag to enable all process APIs.
  #[serde(default)]
  pub all: bool,
  /// Enables the relaunch API.
  #[serde(default)]
  pub relaunch: bool,
  /// Dangerous option that allows macOS to relaunch even if the binary contains a symlink.
  ///
  /// This is due to macOS having less symlink protection. Highly recommended to not set this flag
  /// unless you have a very specific reason too, and understand the implications of it.
  #[serde(
    default,
    alias = "relaunchDangerousAllowSymlinkMacOS",
    alias = "relaunch-dangerous-allow-symlink-macos"
  )]
  pub relaunch_dangerous_allow_symlink_macos: bool,
  /// Enables the exit API.
  #[serde(default)]
  pub exit: bool,
}

impl Allowlist for ProcessAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      all: false,
      relaunch: true,
      relaunch_dangerous_allow_symlink_macos: false,
      exit: true,
    };
    let mut features = allowlist.to_features();
    features.push("process-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["process-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, relaunch, "process-relaunch");
      check_feature!(
        self,
        features,
        relaunch_dangerous_allow_symlink_macos,
        "process-relaunch-dangerous-allow-symlink-macos"
      );
      check_feature!(self, features, exit, "process-exit");
      features
    }
  }
}

/// Allowlist for the clipboard APIs.
///
/// See more: https://tauri.app/v1/api/config#clipboardallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipboardAllowlistConfig {
  /// Use this flag to enable all clipboard APIs.
  #[serde(default)]
  pub all: bool,
  /// Enables the clipboard's `writeText` API.
  #[serde(default, alias = "writeText")]
  pub write_text: bool,
  /// Enables the clipboard's `readText` API.
  #[serde(default, alias = "readText")]
  pub read_text: bool,
}

impl Allowlist for ClipboardAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      all: false,
      write_text: true,
      read_text: true,
    };
    let mut features = allowlist.to_features();
    features.push("clipboard-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["clipboard-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, write_text, "clipboard-write-text");
      check_feature!(self, features, read_text, "clipboard-read-text");
      features
    }
  }
}

/// Allowlist for the app APIs.
///
/// See more: https://tauri.app/v1/api/config#appallowlistconfig
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppAllowlistConfig {
  /// Use this flag to enable all app APIs.
  #[serde(default)]
  pub all: bool,
  /// Enables the app's `show` API.
  #[serde(default)]
  pub show: bool,
  /// Enables the app's `hide` API.
  #[serde(default)]
  pub hide: bool,
}

impl Allowlist for AppAllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let allowlist = Self {
      all: false,
      show: true,
      hide: true,
    };
    let mut features = allowlist.to_features();
    features.push("app-all");
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["app-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, show, "app-show");
      check_feature!(self, features, hide, "app-hide");
      features
    }
  }
}

/// Allowlist configuration. The allowlist is a translation of the [Cargo allowlist features](https://docs.rs/tauri/latest/tauri/#cargo-allowlist-features).
///
/// # Notes
///
/// - Endpoints that don't have their own allowlist option are enabled by default.
/// - There is only "opt-in", no "opt-out". Setting an option to `false` has no effect.
///
/// # Examples
///
/// - * [`"app-all": true`](https://tauri.app/v1/api/config/#appallowlistconfig.all) will make the [hide](https://tauri.app/v1/api/js/app#hide) endpoint be available regardless of whether `hide` is set to `false` or `true` in the allowlist.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllowlistConfig {
  /// Use this flag to enable all API features.
  #[serde(default)]
  pub all: bool,
  /// File system API allowlist.
  #[serde(default)]
  pub fs: FsAllowlistConfig,
  /// Window API allowlist.
  #[serde(default)]
  pub window: WindowAllowlistConfig,
  /// Shell API allowlist.
  #[serde(default)]
  pub shell: ShellAllowlistConfig,
  /// Dialog API allowlist.
  #[serde(default)]
  pub dialog: DialogAllowlistConfig,
  /// HTTP API allowlist.
  #[serde(default)]
  pub http: HttpAllowlistConfig,
  /// Notification API allowlist.
  #[serde(default)]
  pub notification: NotificationAllowlistConfig,
  /// Global shortcut API allowlist.
  #[serde(default, alias = "global-shortcut")]
  pub global_shortcut: GlobalShortcutAllowlistConfig,
  /// OS allowlist.
  #[serde(default)]
  pub os: OsAllowlistConfig,
  /// Path API allowlist.
  #[serde(default)]
  pub path: PathAllowlistConfig,
  /// Custom protocol allowlist.
  #[serde(default)]
  pub protocol: ProtocolAllowlistConfig,
  /// Process API allowlist.
  #[serde(default)]
  pub process: ProcessAllowlistConfig,
  /// Clipboard APIs allowlist.
  #[serde(default)]
  pub clipboard: ClipboardAllowlistConfig,
  /// App APIs allowlist.
  #[serde(default)]
  pub app: AppAllowlistConfig,
}

impl Allowlist for AllowlistConfig {
  fn all_features() -> Vec<&'static str> {
    let mut features = vec!["api-all"];
    features.extend(FsAllowlistConfig::all_features());
    features.extend(WindowAllowlistConfig::all_features());
    features.extend(ShellAllowlistConfig::all_features());
    features.extend(DialogAllowlistConfig::all_features());
    features.extend(HttpAllowlistConfig::all_features());
    features.extend(NotificationAllowlistConfig::all_features());
    features.extend(GlobalShortcutAllowlistConfig::all_features());
    features.extend(OsAllowlistConfig::all_features());
    features.extend(PathAllowlistConfig::all_features());
    features.extend(ProtocolAllowlistConfig::all_features());
    features.extend(ProcessAllowlistConfig::all_features());
    features.extend(ClipboardAllowlistConfig::all_features());
    features.extend(AppAllowlistConfig::all_features());
    features
  }

  fn to_features(&self) -> Vec<&'static str> {
    if self.all {
      vec!["api-all"]
    } else {
      let mut features = Vec::new();
      features.extend(self.fs.to_features());
      features.extend(self.window.to_features());
      features.extend(self.shell.to_features());
      features.extend(self.dialog.to_features());
      features.extend(self.http.to_features());
      features.extend(self.notification.to_features());
      features.extend(self.global_shortcut.to_features());
      features.extend(self.os.to_features());
      features.extend(self.path.to_features());
      features.extend(self.protocol.to_features());
      features.extend(self.process.to_features());
      features.extend(self.clipboard.to_features());
      features.extend(self.app.to_features());
      features
    }
  }
}

/// The application pattern.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase", tag = "use", content = "options")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PatternKind {
  /// Brownfield pattern.
  #[default]
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  Isolation {
    /// The dir containing the index.html file that contains the secure isolation application.
    dir: PathBuf,
  },
}

/// The Tauri configuration object.
///
/// See more: https://tauri.app/v1/api/config#tauriconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TauriConfig {
  /// The pattern to use.
  #[serde(default)]
  pub pattern: PatternKind,
  /// The windows configuration.
  #[serde(default)]
  pub windows: Vec<WindowConfig>,
  /// The CLI configuration.
  pub cli: Option<CliConfig>,
  /// The bundler configuration.
  #[serde(default)]
  pub bundle: BundleConfig,
  /// The allowlist configuration.
  #[serde(default)]
  pub allowlist: AllowlistConfig,
  /// Security configuration.
  #[serde(default)]
  pub security: SecurityConfig,
  /// The updater configuration.
  #[serde(default)]
  pub updater: UpdaterConfig,
  /// Configuration for app system tray.
  #[serde(alias = "system-tray")]
  pub system_tray: Option<SystemTrayConfig>,
  /// MacOS private API configuration. Enables the transparent background API and sets the `fullScreenEnabled` preference to `true`.
  #[serde(rename = "macOSPrivateApi", alias = "macos-private-api", default)]
  pub macos_private_api: bool,
}

/// A URL to an updater server.
///
/// The URL must use the `https` scheme on production.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UpdaterEndpoint(pub Url);

impl std::fmt::Display for UpdaterEndpoint {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl<'de> Deserialize<'de> for UpdaterEndpoint {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let url = Url::deserialize(deserializer)?;
    #[cfg(all(not(debug_assertions), not(feature = "schema")))]
    {
      if url.scheme() != "https" {
        return Err(serde::de::Error::custom(
          "The configured updater endpoint must use the `https` protocol.",
        ));
      }
    }
    Ok(Self(url))
  }
}

/// Install modes for the Windows update.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "camelCase"))]
pub enum WindowsUpdateInstallMode {
  /// Specifies there's a basic UI during the installation process, including a final dialog box at the end.
  BasicUi,
  /// The quiet mode means there's no user interaction required.
  /// Requires admin privileges if the installer does.
  Quiet,
  /// Specifies unattended mode, which means the installation only shows a progress bar.
  #[default]
  Passive,
  // to add more modes, we need to check if the updater relaunch makes sense
  // i.e. for a full UI mode, the user can also mark the installer to start the app
}

impl Display for WindowsUpdateInstallMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::BasicUi => "basicUI",
        Self::Quiet => "quiet",
        Self::Passive => "passive",
      }
    )
  }
}

impl Serialize for WindowsUpdateInstallMode {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for WindowsUpdateInstallMode {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
      "basicui" => Ok(Self::BasicUi),
      "quiet" => Ok(Self::Quiet),
      "passive" => Ok(Self::Passive),
      _ => Err(DeError::custom(format!(
        "unknown update install mode '{s}'"
      ))),
    }
  }
}

/// The updater configuration for Windows.
///
/// See more: https://tauri.app/v1/api/config#updaterwindowsconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterWindowsConfig {
  /// Additional arguments given to the NSIS or WiX installer.
  #[serde(default, alias = "installer-args")]
  pub installer_args: Vec<String>,
  /// The installation mode for the update on Windows. Defaults to `passive`.
  #[serde(default, alias = "install-mode")]
  pub install_mode: WindowsUpdateInstallMode,
}

/// The Updater configuration object.
///
/// See more: https://tauri.app/v1/api/config#updaterconfig
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterConfig {
  /// Whether the updater is active or not.
  #[serde(default)]
  pub active: bool,
  /// Display built-in dialog or use event system if disabled.
  #[serde(default = "default_true")]
  pub dialog: bool,
  /// The updater endpoints. TLS is enforced on production.
  ///
  /// The updater URL can contain the following variables:
  /// - {{current_version}}: The version of the app that is requesting the update
  /// - {{target}}: The operating system name (one of `linux`, `windows` or `darwin`).
  /// - {{arch}}: The architecture of the machine (one of `x86_64`, `i686`, `aarch64` or `armv7`).
  ///
  /// # Examples
  /// - "https://my.cdn.com/latest.json": a raw JSON endpoint that returns the latest version and download links for each platform.
  /// - "https://updates.app.dev/{{target}}?version={{current_version}}&arch={{arch}}": a dedicated API with positional and query string arguments.
  #[allow(rustdoc::bare_urls)]
  pub endpoints: Option<Vec<UpdaterEndpoint>>,
  /// Signature public key.
  #[serde(default)] // use default just so the schema doesn't flag it as required
  pub pubkey: String,
  /// The Windows configuration for the updater.
  #[serde(default)]
  pub windows: UpdaterWindowsConfig,
}

impl<'de> Deserialize<'de> for UpdaterConfig {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct InnerUpdaterConfig {
      #[serde(default)]
      active: bool,
      #[serde(default = "default_true")]
      dialog: bool,
      endpoints: Option<Vec<UpdaterEndpoint>>,
      pubkey: Option<String>,
      #[serde(default)]
      windows: UpdaterWindowsConfig,
    }

    let config = InnerUpdaterConfig::deserialize(deserializer)?;

    if config.active && config.pubkey.is_none() {
      return Err(DeError::custom(
        "The updater `pubkey` configuration is required.",
      ));
    }

    Ok(UpdaterConfig {
      active: config.active,
      dialog: config.dialog,
      endpoints: config.endpoints,
      pubkey: config.pubkey.unwrap_or_default(),
      windows: config.windows,
    })
  }
}

impl Default for UpdaterConfig {
  fn default() -> Self {
    Self {
      active: false,
      dialog: true,
      endpoints: None,
      pubkey: "".into(),
      windows: Default::default(),
    }
  }
}

/// Configuration for application system tray icon.
///
/// See more: https://tauri.app/v1/api/config#systemtrayconfig
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemTrayConfig {
  /// Path to the default icon to use on the system tray.
  #[serde(alias = "icon-path")]
  pub icon_path: PathBuf,
  /// A Boolean value that determines whether the image represents a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image on macOS.
  #[serde(default, alias = "icon-as-template")]
  pub icon_as_template: bool,
  /// A Boolean value that determines whether the menu should appear when the tray icon receives a left click on macOS.
  #[serde(default = "default_true", alias = "menu-on-left-click")]
  pub menu_on_left_click: bool,
  /// Title for MacOS tray
  pub title: Option<String>,
}

/// Defines the URL or assets to embed in the application.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum AppUrl {
  /// The app's external URL, or the path to the directory containing the app assets.
  Url(WindowUrl),
  /// An array of files to embed on the app.
  Files(Vec<PathBuf>),
}

impl std::fmt::Display for AppUrl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Url(url) => write!(f, "{url}"),
      Self::Files(files) => write!(f, "{}", serde_json::to_string(files).unwrap()),
    }
  }
}

/// Describes the shell command to run before `tauri dev`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum BeforeDevCommand {
  /// Run the given script with the default options.
  Script(String),
  /// Run the given script with custom options.
  ScriptWithOptions {
    /// The script to execute.
    script: String,
    /// The current working directory.
    cwd: Option<String>,
    /// Whether `tauri dev` should wait for the command to finish or not. Defaults to `false`.
    #[serde(default)]
    wait: bool,
  },
}

/// Describes a shell command to be executed when a CLI hook is triggered.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum HookCommand {
  /// Run the given script with the default options.
  Script(String),
  /// Run the given script with custom options.
  ScriptWithOptions {
    /// The script to execute.
    script: String,
    /// The current working directory.
    cwd: Option<String>,
  },
}

/// The Build configuration object.
///
/// See more: https://tauri.app/v1/api/config#buildconfig
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildConfig {
  /// The binary used to build and run the application.
  pub runner: Option<String>,
  /// The path to the application assets or URL to load in development.
  ///
  /// This is usually an URL to a dev server, which serves your application assets
  /// with live reloading. Most modern JavaScript bundlers provides a way to start a dev server by default.
  ///
  /// See [vite](https://vitejs.dev/guide/), [Webpack DevServer](https://webpack.js.org/configuration/dev-server/) and [sirv](https://github.com/lukeed/sirv)
  /// for examples on how to set up a dev server.
  #[serde(default = "default_dev_path", alias = "dev-path")]
  pub dev_path: AppUrl,
  /// The path to the application assets or URL to load in production.
  ///
  /// When a path relative to the configuration file is provided,
  /// it is read recursively and all files are embedded in the application binary.
  /// Tauri then looks for an `index.html` file unless you provide a custom window URL.
  ///
  /// You can also provide a list of paths to be embedded, which allows granular control over what files are added to the binary.
  /// In this case, all files are added to the root and you must reference it that way in your HTML files.
  ///
  /// When an URL is provided, the application won't have bundled assets
  /// and the application will load that URL by default.
  #[serde(default = "default_dist_dir", alias = "dist-dir")]
  pub dist_dir: AppUrl,
  /// A shell command to run before `tauri dev` kicks in.
  ///
  /// The TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-dev-command")]
  pub before_dev_command: Option<BeforeDevCommand>,
  /// A shell command to run before `tauri build` kicks in.
  ///
  /// The TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-build-command")]
  pub before_build_command: Option<HookCommand>,
  /// A shell command to run before the bundling phase in `tauri build` kicks in.
  ///
  /// The TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-bundle-command")]
  pub before_bundle_command: Option<HookCommand>,
  /// Features passed to `cargo` commands.
  pub features: Option<Vec<String>>,
  /// Whether we should inject the Tauri API on `window.__TAURI__` or not.
  #[serde(default, alias = "with-global-tauri")]
  pub with_global_tauri: bool,
}

impl Default for BuildConfig {
  fn default() -> Self {
    Self {
      runner: None,
      dev_path: default_dev_path(),
      dist_dir: default_dist_dir(),
      before_dev_command: None,
      before_build_command: None,
      before_bundle_command: None,
      features: None,
      with_global_tauri: false,
    }
  }
}

fn default_dev_path() -> AppUrl {
  AppUrl::Url(WindowUrl::External(
    Url::parse("http://localhost:8080").unwrap(),
  ))
}

fn default_dist_dir() -> AppUrl {
  AppUrl::Url(WindowUrl::App("../dist".into()))
}

#[derive(Debug, PartialEq, Eq)]
struct PackageVersion(String);

impl<'d> serde::Deserialize<'d> for PackageVersion {
  fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<PackageVersion, D::Error> {
    struct PackageVersionVisitor;

    impl<'d> Visitor<'d> for PackageVersionVisitor {
      type Value = PackageVersion;

      fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
          formatter,
          "a semver string or a path to a package.json file"
        )
      }

      fn visit_str<E: DeError>(self, value: &str) -> Result<PackageVersion, E> {
        let path = PathBuf::from(value);
        if path.exists() {
          let json_str = read_to_string(&path)
            .map_err(|e| DeError::custom(format!("failed to read version JSON file: {e}")))?;
          let package_json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| DeError::custom(format!("failed to read version JSON file: {e}")))?;
          if let Some(obj) = package_json.as_object() {
            let version = obj
              .get("version")
              .ok_or_else(|| DeError::custom("JSON must contain a `version` field"))?
              .as_str()
              .ok_or_else(|| {
                DeError::custom(format!("`{} > version` must be a string", path.display()))
              })?;
            Ok(PackageVersion(
              Version::from_str(version)
                .map_err(|_| DeError::custom("`package > version` must be a semver string"))?
                .to_string(),
            ))
          } else {
            Err(DeError::custom(
              "`package > version` value is not a path to a JSON object",
            ))
          }
        } else {
          Ok(PackageVersion(
            Version::from_str(value)
              .map_err(|_| DeError::custom("`package > version` must be a semver string"))?
              .to_string(),
          ))
        }
      }
    }

    deserializer.deserialize_string(PackageVersionVisitor {})
  }
}

/// The package configuration.
///
/// See more: https://tauri.app/v1/api/config#packageconfig
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageConfig {
  /// App name.
  #[serde(alias = "product-name")]
  #[cfg_attr(feature = "schema", validate(regex(pattern = "^[^/\\:*?\"<>|]+$")))]
  pub product_name: Option<String>,
  /// App version. It is a semver version number or a path to a `package.json` file containing the `version` field. If removed the version number from `Cargo.toml` is used.
  #[serde(deserialize_with = "version_deserializer", default)]
  pub version: Option<String>,
}

fn version_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<PackageVersion>::deserialize(deserializer).map(|v| v.map(|v| v.0))
}

/// The Tauri configuration object.
/// It is read from a file where you can define your frontend assets,
/// configure the bundler, enable the app updater, define a system tray,
/// enable APIs via the allowlist and more.
///
/// The configuration file is generated by the
/// [`tauri init`](https://tauri.app/v1/api/cli#init) command that lives in
/// your Tauri application source directory (src-tauri).
///
/// Once generated, you may modify it at will to customize your Tauri application.
///
/// ## File Formats
///
/// By default, the configuration is defined as a JSON file named `tauri.conf.json`.
///
/// Tauri also supports JSON5 and TOML files via the `config-json5` and `config-toml` Cargo features, respectively.
/// The JSON5 file name must be either `tauri.conf.json` or `tauri.conf.json5`.
/// The TOML file name is `Tauri.toml`.
///
/// ## Platform-Specific Configuration
///
/// In addition to the default configuration file, Tauri can
/// read a platform-specific configuration from `tauri.linux.conf.json`,
/// `tauri.windows.conf.json`, and `tauri.macos.conf.json`
/// (or `Tauri.linux.toml`, `Tauri.windows.toml` and `Tauri.macos.toml` if the `Tauri.toml` format is used),
/// which gets merged with the main configuration object.
///
/// ## Configuration Structure
///
/// The configuration is composed of the following objects:
///
/// - [`package`](#packageconfig): Package settings
/// - [`tauri`](#tauriconfig): The Tauri config
/// - [`build`](#buildconfig): The build configuration
/// - [`plugins`](#pluginconfig): The plugins config
///
/// ```json title="Example tauri.config.json file"
/// {
///   "build": {
///     "beforeBuildCommand": "",
///     "beforeDevCommand": "",
///     "devPath": "../dist",
///     "distDir": "../dist"
///   },
///   "package": {
///     "productName": "tauri-app",
///     "version": "0.1.0"
///   },
///   "tauri": {
///     "allowlist": {
///       "all": true
///     },
///     "bundle": {},
///     "security": {
///       "csp": null
///     },
///     "updater": {
///       "active": false
///     },
///     "windows": [
///       {
///         "fullscreen": false,
///         "height": 600,
///         "resizable": true,
///         "title": "Tauri App",
///         "width": 800
///       }
///     ]
///   }
/// }
/// ```
#[allow(rustdoc::invalid_codeblock_attributes)]
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
  /// The JSON schema for the Tauri config.
  #[serde(rename = "$schema")]
  pub schema: Option<String>,
  /// Package settings.
  #[serde(default)]
  pub package: PackageConfig,
  /// The Tauri configuration.
  #[serde(default)]
  pub tauri: TauriConfig,
  /// The build configuration.
  #[serde(default = "default_build")]
  pub build: BuildConfig,
  /// The plugins config.
  #[serde(default)]
  pub plugins: PluginConfig,
}

/// The plugin configs holds a HashMap mapping a plugin name to its configuration object.
///
/// See more: https://tauri.app/v1/api/config#pluginconfig
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PluginConfig(pub HashMap<String, JsonValue>);

fn default_build() -> BuildConfig {
  BuildConfig {
    runner: None,
    dev_path: default_dev_path(),
    dist_dir: default_dist_dir(),
    before_dev_command: None,
    before_build_command: None,
    before_bundle_command: None,
    features: None,
    with_global_tauri: false,
  }
}

/// How the window title bar should be displayed on macOS.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TitleBarStyle {
  /// A normal title bar.
  #[default]
  Visible,
  /// Makes the title bar transparent, so the window background color is shown instead.
  ///
  /// Useful if you don't need to have actual HTML under the title bar. This lets you avoid the caveats of using `TitleBarStyle::Overlay`. Will be more useful when Tauri lets you set a custom window background color.
  Transparent,
  /// Shows the title bar as a transparent overlay over the window's content.
  ///
  /// Keep in mind:
  /// - The height of the title bar is different on different OS versions, which can lead to window the controls and title not being where you don't expect.
  /// - You need to define a custom drag region to make your window draggable, however due to a limitation you can't drag the window when it's not in focus <https://github.com/tauri-apps/tauri/issues/4316>.
  /// - The color of the window title depends on the system theme.
  Overlay,
}

impl Serialize for TitleBarStyle {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for TitleBarStyle {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(match s.to_lowercase().as_str() {
      "transparent" => Self::Transparent,
      "overlay" => Self::Overlay,
      _ => Self::Visible,
    })
  }
}

impl Display for TitleBarStyle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Visible => "Visible",
        Self::Transparent => "Transparent",
        Self::Overlay => "Overlay",
      }
    )
  }
}

/// System theme.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum Theme {
  /// Light theme.
  Light,
  /// Dark theme.
  Dark,
}

impl Serialize for Theme {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for Theme {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(match s.to_lowercase().as_str() {
      "dark" => Self::Dark,
      _ => Self::Light,
    })
  }
}

impl Display for Theme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Light => "light",
        Self::Dark => "dark",
      }
    )
  }
}

#[cfg(test)]
mod test {
  use super::*;

  // TODO: create a test that compares a config to a json config

  #[test]
  // test all of the default functions
  fn test_defaults() {
    // get default tauri config
    let t_config = TauriConfig::default();
    // get default build config
    let b_config = BuildConfig::default();
    // get default dev path
    let d_path = default_dev_path();
    // get default window
    let d_windows: Vec<WindowConfig> = vec![];
    // get default bundle
    let d_bundle = BundleConfig::default();
    // get default updater
    let d_updater = UpdaterConfig::default();

    // create a tauri config.
    let tauri = TauriConfig {
      pattern: Default::default(),
      windows: vec![],
      bundle: BundleConfig {
        active: false,
        targets: Default::default(),
        identifier: String::from(""),
        publisher: None,
        icon: Vec::new(),
        resources: None,
        copyright: None,
        category: None,
        short_description: None,
        long_description: None,
        appimage: Default::default(),
        deb: Default::default(),
        macos: Default::default(),
        external_bin: None,
        windows: Default::default(),
      },
      cli: None,
      updater: UpdaterConfig {
        active: false,
        dialog: true,
        pubkey: "".into(),
        endpoints: None,
        windows: Default::default(),
      },
      security: SecurityConfig {
        csp: None,
        dev_csp: None,
        freeze_prototype: false,
        dangerous_disable_asset_csp_modification: DisabledCspModificationKind::Flag(false),
        dangerous_remote_domain_ipc_access: Vec::new(),
        dangerous_use_http_scheme: false,
      },
      allowlist: AllowlistConfig::default(),
      system_tray: None,
      macos_private_api: false,
    };

    // create a build config
    let build = BuildConfig {
      runner: None,
      dev_path: AppUrl::Url(WindowUrl::External(
        Url::parse("http://localhost:8080").unwrap(),
      )),
      dist_dir: AppUrl::Url(WindowUrl::App("../dist".into())),
      before_dev_command: None,
      before_build_command: None,
      before_bundle_command: None,
      features: None,
      with_global_tauri: false,
    };

    // test the configs
    assert_eq!(t_config, tauri);
    assert_eq!(b_config, build);
    assert_eq!(d_bundle, tauri.bundle);
    assert_eq!(d_updater, tauri.updater);
    assert_eq!(
      d_path,
      AppUrl::Url(WindowUrl::External(
        Url::parse("http://localhost:8080").unwrap()
      ))
    );
    assert_eq!(d_windows, tauri.windows);
  }
}
