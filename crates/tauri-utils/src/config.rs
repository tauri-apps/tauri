// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri configuration used at runtime.
//!
//! It is pulled from a `tauri.conf.json` file and the [`Config`] struct is generated at compile time.
//!
//! # Stability
//!
//! This is a core functionality that is not considered part of the stable API.
//! If you use it, note that it may include breaking changes in the future.
//!
//! These items are intended to be non-breaking from a de/serialization standpoint only.
//! Using and modifying existing config values will try to avoid breaking changes, but they are
//! free to add fields in the future - causing breaking changes for creating and full destructuring.
//!
//! To avoid this, [ignore unknown fields when destructuring] with the `{my, config, ..}` pattern.
//! If you need to create the Rust config directly without deserializing, then create the struct
//! the [Struct Update Syntax] with `..Default::default()`, which may need a
//! `#[allow(clippy::needless_update)]` attribute if you are declaring all fields.
//!
//! [ignore unknown fields when destructuring]: https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html#ignoring-remaining-parts-of-a-value-with-
//! [Struct Update Syntax]: https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax

use http::response::Builder;
#[cfg(feature = "schema")]
use schemars::schema::Schema;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use semver::Version;
use serde::{
  de::{Deserializer, Error as DeError, Visitor},
  Deserialize, Serialize, Serializer,
};
use serde_json::Value as JsonValue;
use serde_untagged::UntaggedEnumVisitor;
use serde_with::skip_serializing_none;
use url::Url;

use std::{
  collections::HashMap,
  fmt::{self, Display},
  fs::read_to_string,
  path::PathBuf,
  str::FromStr,
};

#[cfg(feature = "schema")]
fn add_description(schema: Schema, description: impl Into<String>) -> Schema {
  let value = description.into();
  if value.is_empty() {
    schema
  } else {
    let mut schema_obj = schema.into_object();
    schema_obj.metadata().description = value.into();
    Schema::Object(schema_obj)
  }
}

/// Items to help with parsing content into a [`Config`].
pub mod parse;

use crate::{acl::capability::Capability, TitleBarStyle, WindowEffect, WindowEffectState};

pub use self::parse::parse;

fn default_true() -> bool {
  true
}

/// An URL to open on a Tauri webview window.
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum WebviewUrl {
  /// An external URL. Must use either the `http` or `https` schemes.
  External(Url),
  /// The path portion of an app URL.
  /// For instance, to load `tauri://localhost/users/john`,
  /// you can simply provide `users/john` in this configuration.
  App(PathBuf),
  /// A custom protocol url, for example, `doom://index.html`
  CustomProtocol(Url),
}

impl<'de> Deserialize<'de> for WebviewUrl {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WebviewUrlDeserializer {
      Url(Url),
      Path(PathBuf),
    }

    match WebviewUrlDeserializer::deserialize(deserializer)? {
      WebviewUrlDeserializer::Url(u) => {
        if u.scheme() == "https" || u.scheme() == "http" {
          Ok(Self::External(u))
        } else {
          Ok(Self::CustomProtocol(u))
        }
      }
      WebviewUrlDeserializer::Path(p) => Ok(Self::App(p)),
    }
  }
}

impl fmt::Display for WebviewUrl {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::External(url) | Self::CustomProtocol(url) => write!(f, "{url}"),
      Self::App(path) => write!(f, "{}", path.display()),
    }
  }
}

impl Default for WebviewUrl {
  fn default() -> Self {
    Self::App("index.html".into())
  }
}

/// A bundle referenced by tauri-bundler.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
pub enum BundleType {
  /// The debian bundle (.deb).
  Deb,
  /// The RPM bundle (.rpm).
  Rpm,
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
}

impl BundleType {
  /// All bundle types.
  fn all() -> &'static [Self] {
    &[
      BundleType::Deb,
      BundleType::Rpm,
      BundleType::AppImage,
      BundleType::Msi,
      BundleType::Nsis,
      BundleType::App,
      BundleType::Dmg,
    ]
  }
}

impl Display for BundleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Deb => "deb",
        Self::Rpm => "rpm",
        Self::AppImage => "appimage",
        Self::Msi => "msi",
        Self::Nsis => "nsis",
        Self::App => "app",
        Self::Dmg => "dmg",
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
      "rpm" => Ok(Self::Rpm),
      "appimage" => Ok(Self::AppImage),
      "msi" => Ok(Self::Msi),
      "nsis" => Ok(Self::Nsis),
      "app" => Ok(Self::App),
      "dmg" => Ok(Self::Dmg),
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

#[cfg(feature = "schema")]
impl schemars::JsonSchema for BundleTarget {
  fn schema_name() -> std::string::String {
    "BundleTarget".to_owned()
  }

  fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let any_of = vec![
      schemars::schema::SchemaObject {
        const_value: Some("all".into()),
        metadata: Some(Box::new(schemars::schema::Metadata {
          description: Some("Bundle all targets.".to_owned()),
          ..Default::default()
        })),
        ..Default::default()
      }
      .into(),
      add_description(
        gen.subschema_for::<Vec<BundleType>>(),
        "A list of bundle targets.",
      ),
      add_description(gen.subschema_for::<BundleType>(), "A single bundle target."),
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
      BundleTargetInner::All(t) => Err(DeError::custom(format!(
        "invalid bundle type {t}, expected one of `all`, {}",
        BundleType::all()
          .iter()
          .map(|b| format!("`{b}`"))
          .collect::<Vec<_>>()
          .join(", ")
      ))),
      BundleTargetInner::List(l) => Ok(Self::List(l)),
      BundleTargetInner::One(t) => Ok(Self::One(t)),
    }
  }
}

impl BundleTarget {
  /// Gets the bundle targets as a [`Vec`]. The vector is empty when set to [`BundleTarget::All`].
  #[allow(dead_code)]
  pub fn to_vec(&self) -> Vec<BundleType> {
    match self {
      Self::All => BundleType::all().to_vec(),
      Self::List(list) => list.clone(),
      Self::One(i) => vec![i.clone()],
    }
  }
}

/// Configuration for AppImage bundles.
///
/// See more: <https://v2.tauri.app/reference/config/#appimageconfig>
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppImageConfig {
  /// Include additional gstreamer dependencies needed for audio and video playback.
  /// This increases the bundle size by ~15-35MB depending on your build system.
  #[serde(default, alias = "bundle-media-framework")]
  pub bundle_media_framework: bool,
  /// The files to include in the Appimage Binary.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
}

/// Configuration for Debian (.deb) bundles.
///
/// See more: <https://v2.tauri.app/reference/config/#debconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebConfig {
  /// The list of deb dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// The list of deb dependencies your application recommends.
  pub recommends: Option<Vec<String>>,
  /// The list of dependencies the package provides.
  pub provides: Option<Vec<String>>,
  /// The list of package conflicts.
  pub conflicts: Option<Vec<String>>,
  /// The list of package replaces.
  pub replaces: Option<Vec<String>>,
  /// The files to include on the package.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
  /// Define the section in Debian Control file. See : https://www.debian.org/doc/debian-policy/ch-archive.html#s-subsections
  pub section: Option<String>,
  /// Change the priority of the Debian Package. By default, it is set to `optional`.
  /// Recognized Priorities as of now are :  `required`, `important`, `standard`, `optional`, `extra`
  pub priority: Option<String>,
  /// Path of the uncompressed Changelog file, to be stored at /usr/share/doc/package-name/changelog.gz. See
  /// <https://www.debian.org/doc/debian-policy/ch-docs.html#changelog-files-and-release-notes>
  pub changelog: Option<PathBuf>,
  /// Path to a custom desktop file Handlebars template.
  ///
  /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
  #[serde(alias = "desktop-template")]
  pub desktop_template: Option<PathBuf>,
  /// Path to script that will be executed before the package is unpacked. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  #[serde(alias = "pre-install-script")]
  pub pre_install_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is unpacked. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  #[serde(alias = "post-install-script")]
  pub post_install_script: Option<PathBuf>,
  /// Path to script that will be executed before the package is removed. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  #[serde(alias = "pre-remove-script")]
  pub pre_remove_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is removed. See
  /// <https://www.debian.org/doc/debian-policy/ch-maintainerscripts.html>
  #[serde(alias = "post-remove-script")]
  pub post_remove_script: Option<PathBuf>,
}

/// Configuration for Linux bundles.
///
/// See more: <https://v2.tauri.app/reference/config/#linuxconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinuxConfig {
  /// Configuration for the AppImage bundle.
  #[serde(default)]
  pub appimage: AppImageConfig,
  /// Configuration for the Debian bundle.
  #[serde(default)]
  pub deb: DebConfig,
  /// Configuration for the RPM bundle.
  #[serde(default)]
  pub rpm: RpmConfig,
}

/// Compression algorithms used when bundling RPM packages.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, tag = "type")]
#[non_exhaustive]
pub enum RpmCompression {
  /// Gzip compression
  Gzip {
    /// Gzip compression level
    level: u32,
  },
  /// Zstd compression
  Zstd {
    /// Zstd compression level
    level: i32,
  },
  /// Xz compression
  Xz {
    /// Xz compression level
    level: u32,
  },
  /// Bzip2 compression
  Bzip2 {
    /// Bzip2 compression level
    level: u32,
  },
  /// Disable compression
  None,
}

/// Configuration for RPM bundles.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RpmConfig {
  /// The list of RPM dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// The list of RPM dependencies your application recommends.
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
  #[serde(default = "default_release")]
  pub release: String,
  /// The RPM epoch.
  #[serde(default)]
  pub epoch: u32,
  /// The files to include on the package.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
  /// Path to a custom desktop file Handlebars template.
  ///
  /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
  #[serde(alias = "desktop-template")]
  pub desktop_template: Option<PathBuf>,
  /// Path to script that will be executed before the package is unpacked. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  #[serde(alias = "pre-install-script")]
  pub pre_install_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is unpacked. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  #[serde(alias = "post-install-script")]
  pub post_install_script: Option<PathBuf>,
  /// Path to script that will be executed before the package is removed. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  #[serde(alias = "pre-remove-script")]
  pub pre_remove_script: Option<PathBuf>,
  /// Path to script that will be executed after the package is removed. See
  /// <http://ftp.rpm.org/max-rpm/s1-rpm-inside-scripts.html>
  #[serde(alias = "post-remove-script")]
  pub post_remove_script: Option<PathBuf>,
  /// Compression algorithm and level. Defaults to `Gzip` with level 6.
  pub compression: Option<RpmCompression>,
}

impl Default for RpmConfig {
  fn default() -> Self {
    Self {
      depends: None,
      recommends: None,
      provides: None,
      conflicts: None,
      obsoletes: None,
      release: default_release(),
      epoch: 0,
      files: Default::default(),
      desktop_template: None,
      pre_install_script: None,
      post_install_script: None,
      pre_remove_script: None,
      post_remove_script: None,
      compression: None,
    }
  }
}

fn default_release() -> String {
  "1".into()
}

/// Position coordinates struct.
#[derive(Default, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Position {
  /// X coordinate.
  pub x: u32,
  /// Y coordinate.
  pub y: u32,
}

/// Position coordinates struct.
#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogicalPosition {
  /// X coordinate.
  pub x: f64,
  /// Y coordinate.
  pub y: f64,
}

/// Size of the window.
#[derive(Default, Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Size {
  /// Width of the window.
  pub width: u32,
  /// Height of the window.
  pub height: u32,
}

/// Configuration for Apple Disk Image (.dmg) bundles.
///
/// See more: <https://v2.tauri.app/reference/config/#dmgconfig>
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DmgConfig {
  /// Image to use as the background in dmg file. Accepted formats: `png`/`jpg`/`gif`.
  pub background: Option<PathBuf>,
  /// Position of volume window on screen.
  pub window_position: Option<Position>,
  /// Size of volume window.
  #[serde(default = "dmg_window_size", alias = "window-size")]
  pub window_size: Size,
  /// Position of app file on window.
  #[serde(default = "dmg_app_position", alias = "app-position")]
  pub app_position: Position,
  /// Position of application folder on window.
  #[serde(
    default = "dmg_application_folder_position",
    alias = "application-folder-position"
  )]
  pub application_folder_position: Position,
}

impl Default for DmgConfig {
  fn default() -> Self {
    Self {
      background: None,
      window_position: None,
      window_size: dmg_window_size(),
      app_position: dmg_app_position(),
      application_folder_position: dmg_application_folder_position(),
    }
  }
}

fn dmg_window_size() -> Size {
  Size {
    width: 660,
    height: 400,
  }
}

fn dmg_app_position() -> Position {
  Position { x: 180, y: 170 }
}

fn dmg_application_folder_position() -> Position {
  Position { x: 480, y: 170 }
}

fn de_macos_minimum_system_version<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let version = Option::<String>::deserialize(deserializer)?;
  match version {
    Some(v) if v.is_empty() => Ok(macos_minimum_system_version()),
    e => Ok(e),
  }
}

/// Configuration for the macOS bundles.
///
/// See more: <https://v2.tauri.app/reference/config/#macconfig>
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MacConfig {
  /// A list of strings indicating any macOS X frameworks that need to be bundled with the application.
  ///
  /// If a name is used, ".framework" must be omitted and it will look for standard install locations. You may also use a path to a specific framework.
  pub frameworks: Option<Vec<String>>,
  /// The files to include in the application relative to the Contents directory.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
  /// The version of the build that identifies an iteration of the bundle.
  ///
  /// Translates to the bundle's CFBundleVersion property.
  #[serde(alias = "bundle-version")]
  pub bundle_version: Option<String>,
  /// The name of the builder that built the bundle.
  ///
  /// Translates to the bundle's CFBundleName property.
  ///
  /// If not set, defaults to the package's product name.
  #[serde(alias = "bundle-name")]
  pub bundle_name: Option<String>,
  /// A version string indicating the minimum macOS X version that the bundled application supports. Defaults to `10.13`.
  ///
  /// Setting it to `null` completely removes the `LSMinimumSystemVersion` field on the bundle's `Info.plist`
  /// and the `MACOSX_DEPLOYMENT_TARGET` environment variable.
  ///
  /// Ignored in `tauri dev`.
  ///
  /// An empty string is considered an invalid value so the default value is used.
  #[serde(
    deserialize_with = "de_macos_minimum_system_version",
    default = "macos_minimum_system_version",
    alias = "minimum-system-version"
  )]
  pub minimum_system_version: Option<String>,
  /// Allows your application to communicate with the outside world.
  /// It should be a lowercase, without port and protocol domain name.
  #[serde(alias = "exception-domain")]
  pub exception_domain: Option<String>,
  /// Identity to use for code signing.
  #[serde(alias = "signing-identity")]
  pub signing_identity: Option<String>,
  /// Whether the codesign should enable [hardened runtime](https://developer.apple.com/documentation/security/hardened_runtime) (for executables) or not.
  #[serde(alias = "hardened-runtime", default = "default_true")]
  pub hardened_runtime: bool,
  /// Provider short name for notarization.
  #[serde(alias = "provider-short-name")]
  pub provider_short_name: Option<String>,
  /// Path to the entitlements file.
  pub entitlements: Option<String>,
  /// Path to a Info.plist file to merge with the default Info.plist.
  ///
  /// Note that Tauri also looks for a `Info.plist` file in the same directory as the Tauri configuration file.
  #[serde(alias = "info-plist")]
  pub info_plist: Option<PathBuf>,
  /// DMG-specific settings.
  #[serde(default)]
  pub dmg: DmgConfig,
}

impl Default for MacConfig {
  fn default() -> Self {
    Self {
      frameworks: None,
      files: HashMap::new(),
      bundle_version: None,
      bundle_name: None,
      minimum_system_version: macos_minimum_system_version(),
      exception_domain: None,
      signing_identity: None,
      hardened_runtime: true,
      provider_short_name: None,
      entitlements: None,
      info_plist: None,
      dmg: Default::default(),
    }
  }
}

fn macos_minimum_system_version() -> Option<String> {
  Some("10.13".into())
}

fn ios_minimum_system_version() -> String {
  "14.0".into()
}

/// Configuration for a target language for the WiX build.
///
/// See more: <https://v2.tauri.app/reference/config/#wixlanguageconfig>
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixLanguageConfig {
  /// The path to a locale (`.wxl`) file. See <https://wixtoolset.org/documentation/manual/v3/howtos/ui_and_localization/build_a_localized_version.html>.
  #[serde(alias = "locale-path")]
  pub locale_path: Option<String>,
}

/// The languages to build using WiX.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
/// See more: <https://v2.tauri.app/reference/config/#wixconfig>
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixConfig {
  /// MSI installer version in the format `major.minor.patch.build` (build is optional).
  ///
  /// Because a valid version is required for MSI installer, it will be derived from [`Config::version`] if this field is not set.
  ///
  /// The first field is the major version and has a maximum value of 255. The second field is the minor version and has a maximum value of 255.
  /// The third and foruth fields have a maximum value of 65,535.
  ///
  /// See <https://learn.microsoft.com/en-us/windows/win32/msi/productversion> for more info.
  pub version: Option<String>,
  /// A GUID upgrade code for MSI installer. This code **_must stay the same across all of your updates_**,
  /// otherwise, Windows will treat your update as a different app and your users will have duplicate versions of your app.
  ///
  /// By default, tauri generates this code by generating a Uuid v5 using the string `<productName>.exe.app.x64` in the DNS namespace.
  /// You can use Tauri's CLI to generate and print this code for you, run `tauri inspect wix-upgrade-code`.
  ///
  /// It is recommended that you set this value in your tauri config file to avoid accidental changes in your upgrade code
  /// whenever you want to change your product name.
  #[serde(alias = "upgrade-code")]
  pub upgrade_code: Option<uuid::Uuid>,
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
  /// Enables FIPS compliant algorithms.
  /// Can also be enabled via the `TAURI_BUNDLER_WIX_FIPS_COMPLIANT` env var.
  #[serde(default, alias = "fips-compliant")]
  pub fips_compliant: bool,
}

/// Compression algorithms used in the NSIS installer.
///
/// See <https://nsis.sourceforge.io/Reference/SetCompressor>
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum NsisCompression {
  /// ZLIB uses the deflate algorithm, it is a quick and simple method. With the default compression level it uses about 300 KB of memory.
  Zlib,
  /// BZIP2 usually gives better compression ratios than ZLIB, but it is a bit slower and uses more memory. With the default compression level it uses about 4 MB of memory.
  Bzip2,
  /// LZMA (default) is a new compression method that gives very good compression ratios. The decompression speed is high (10-20 MB/s on a 2 GHz CPU), the compression speed is lower. The memory size that will be used for decompression is the dictionary size plus a few KBs, the default is 8 MB.
  #[default]
  Lzma,
  /// Disable compression
  None,
}

/// Install Modes for the NSIS installer.
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

/// Configuration for the Installer bundle using NSIS.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NsisConfig {
  /// A custom .nsi template to use.
  pub template: Option<PathBuf>,
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
  /// See <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/languages/English.nsh> for an example `.nsh` file.
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
  #[serde(default)]
  pub compression: NsisCompression,
  /// Set the folder name for the start menu shortcut.
  ///
  /// Use this option if you have multiple apps and wish to group their shortcuts under one folder
  /// or if you generally prefer to set your shortcut inside a folder.
  ///
  /// Examples:
  /// - `AwesomePublisher`, shortcut will be placed in `%AppData%\Microsoft\Windows\Start Menu\Programs\AwesomePublisher\<your-app>.lnk`
  /// - If unset, shortcut will be placed in `%AppData%\Microsoft\Windows\Start Menu\Programs\<your-app>.lnk`
  #[serde(alias = "start-menu-folder")]
  pub start_menu_folder: Option<String>,
  /// A path to a `.nsh` file that contains special NSIS macros to be hooked into the
  /// main installer.nsi script.
  ///
  /// Supported hooks are:
  ///
  /// - `NSIS_HOOK_PREINSTALL`: This hook runs before copying files, setting registry key values and creating shortcuts.
  /// - `NSIS_HOOK_POSTINSTALL`: This hook runs after the installer has finished copying all files, setting the registry keys and created shortcuts.
  /// - `NSIS_HOOK_PREUNINSTALL`: This hook runs before removing any files, registry keys and shortcuts.
  /// - `NSIS_HOOK_POSTUNINSTALL`: This hook runs after files, registry keys and shortcuts have been removed.
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
  #[serde(alias = "installer-hooks")]
  pub installer_hooks: Option<PathBuf>,
  /// Try to ensure that the WebView2 version is equal to or newer than this version,
  /// if the user's WebView2 is older than this version,
  /// the installer will try to trigger a WebView2 update.
  #[serde(alias = "minimum-webview2-version")]
  pub minimum_webview2_version: Option<String>,
}

/// Install modes for the Webview2 runtime.
/// Note that for the updater bundle [`Self::DownloadBootstrapper`] is used.
///
/// For more information see <https://v2.tauri.app/distribute/windows-installer/#webview2-installation-options>.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

/// Custom Signing Command configuration.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum CustomSignCommandConfig {
  /// A string notation of the script to execute.
  ///
  /// "%1" will be replaced with the path to the binary to be signed.
  ///
  /// This is a simpler notation for the command.
  /// Tauri will split the string with `' '` and use the first element as the command name and the rest as arguments.
  ///
  /// If you need to use whitespace in the command or arguments, use the object notation [`Self::ScriptWithOptions`].
  Command(String),
  /// An object notation of the command.
  ///
  /// This is more complex notation for the command but
  /// this allows you to use whitespace in the command and arguments.
  CommandWithOptions {
    /// The command to run to sign the binary.
    cmd: String,
    /// The arguments to pass to the command.
    ///
    /// "%1" will be replaced with the path to the binary to be signed.
    args: Vec<String>,
  },
}

/// Windows bundler configuration.
///
/// See more: <https://v2.tauri.app/reference/config/#windowsconfig>
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  /// Specify a custom command to sign the binaries.
  /// This command needs to have a `%1` in args which is just a placeholder for the binary path,
  /// which we will detect and replace before calling the command.
  ///
  /// By Default we use `signtool.exe` which can be found only on Windows so
  /// if you are on another platform and want to cross-compile and sign you will
  /// need to use another tool like `osslsigncode`.
  #[serde(alias = "sign-command")]
  pub sign_command: Option<CustomSignCommandConfig>,
}

impl Default for WindowsConfig {
  fn default() -> Self {
    Self {
      digest_algorithm: None,
      certificate_thumbprint: None,
      timestamp_url: None,
      tsp: false,
      webview_install_mode: Default::default(),
      allow_downgrades: true,
      wix: None,
      nsis: None,
      sign_command: None,
    }
  }
}

/// macOS-only. Corresponds to CFBundleTypeRole
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum BundleTypeRole {
  /// CFBundleTypeRole.Editor. Files can be read and edited.
  #[default]
  Editor,
  /// CFBundleTypeRole.Viewer. Files can be read.
  Viewer,
  /// CFBundleTypeRole.Shell
  Shell,
  /// CFBundleTypeRole.QLGenerator
  QLGenerator,
  /// CFBundleTypeRole.None
  None,
}

impl Display for BundleTypeRole {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Editor => write!(f, "Editor"),
      Self::Viewer => write!(f, "Viewer"),
      Self::Shell => write!(f, "Shell"),
      Self::QLGenerator => write!(f, "QLGenerator"),
      Self::None => write!(f, "None"),
    }
  }
}

// Issue #13159 - Missing the LSHandlerRank and Apple warns after uploading to App Store Connect.
// https://github.com/tauri-apps/tauri/issues/13159
/// Corresponds to LSHandlerRank
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum HandlerRank {
  /// LSHandlerRank.Default. This app is an opener of files of this type; this value is also used if no rank is specified.
  #[default]
  Default,
  /// LSHandlerRank.Owner. This app is the primary creator of files of this type.
  Owner,
  /// LSHandlerRank.Alternate. This app is a secondary viewer of files of this type.
  Alternate,
  /// LSHandlerRank.None. This app is never selected to open files of this type, but it accepts drops of files of this type.
  None,
}

impl Display for HandlerRank {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Default => write!(f, "Default"),
      Self::Owner => write!(f, "Owner"),
      Self::Alternate => write!(f, "Alternate"),
      Self::None => write!(f, "None"),
    }
  }
}

/// An extension for a [`FileAssociation`].
///
/// A leading `.` is automatically stripped.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct AssociationExt(pub String);

impl fmt::Display for AssociationExt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl<'d> serde::Deserialize<'d> for AssociationExt {
  fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
    let ext = String::deserialize(deserializer)?;
    if let Some(ext) = ext.strip_prefix('.') {
      Ok(AssociationExt(ext.into()))
    } else {
      Ok(AssociationExt(ext))
    }
  }
}

/// File association
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileAssociation {
  /// File extensions to associate with this app. e.g. 'png'
  pub ext: Vec<AssociationExt>,
  /// Declare support to a file with the given content type. Maps to `LSItemContentTypes` on macOS.
  ///
  /// This allows supporting any file format declared by another application that conforms to this type.
  /// Declaration of new types can be done with [`Self::exported_type`] and linking to certain content types are done via [`ExportedFileAssociation::conforms_to`].
  #[serde(alias = "content-types")]
  pub content_types: Option<Vec<String>>,
  /// The name. Maps to `CFBundleTypeName` on macOS. Default to `ext[0]`
  pub name: Option<String>,
  /// The association description. Windows-only. It is displayed on the `Type` column on Windows Explorer.
  pub description: Option<String>,
  /// The app's role with respect to the type. Maps to `CFBundleTypeRole` on macOS.
  #[serde(default)]
  pub role: BundleTypeRole,
  /// The mime-type e.g. 'image/png' or 'text/plain'. Linux-only.
  #[serde(alias = "mime-type")]
  pub mime_type: Option<String>,
  /// The ranking of this app among apps that declare themselves as editors or viewers of the given file type.  Maps to `LSHandlerRank` on macOS.
  #[serde(default)]
  pub rank: HandlerRank,
  /// The exported type definition. Maps to a `UTExportedTypeDeclarations` entry on macOS.
  ///
  /// You should define this if the associated file is a custom file type defined by your application.
  pub exported_type: Option<ExportedFileAssociation>,
}

/// The exported type definition. Maps to a `UTExportedTypeDeclarations` entry on macOS.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportedFileAssociation {
  /// The unique identifier for the exported type. Maps to `UTTypeIdentifier`.
  pub identifier: String,
  /// The types that this type conforms to. Maps to `UTTypeConformsTo`.
  ///
  /// Examples are `public.data`, `public.image`, `public.json` and `public.database`.
  #[serde(alias = "conforms-to")]
  pub conforms_to: Option<Vec<String>>,
}

/// Deep link protocol configuration.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepLinkProtocol {
  /// URL schemes to associate with this app without `://`. For example `my-app`
  #[serde(default)]
  pub schemes: Vec<String>,
  /// Domains to associate with this app. For example `example.com`.
  /// Currently only supported on macOS, translating to an [universal app link].
  ///
  /// Note that universal app links require signed apps with a provisioning profile to work.
  /// You can accomplish that by including the `embedded.provisionprofile` file in the `macOS > files` option.
  ///
  /// [universal app link]: https://developer.apple.com/documentation/xcode/supporting-universal-links-in-your-app
  #[serde(default)]
  pub domains: Vec<String>,
  /// The protocol name. **macOS-only** and maps to `CFBundleTypeName`. Defaults to `<bundle-id>.<schemes[0]>`
  pub name: Option<String>,
  /// The app's role for these schemes. **macOS-only** and maps to `CFBundleTypeRole`.
  #[serde(default)]
  pub role: BundleTypeRole,
}

/// Definition for bundle resources.
/// Can be either a list of paths to include or a map of source to target paths.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum BundleResources {
  /// A list of paths to include.
  List(Vec<String>),
  /// A map of source to target paths.
  Map(HashMap<String, String>),
}

impl BundleResources {
  /// Adds a path to the resource collection.
  pub fn push(&mut self, path: impl Into<String>) {
    match self {
      Self::List(l) => l.push(path.into()),
      Self::Map(l) => {
        let path = path.into();
        l.insert(path.clone(), path);
      }
    }
  }
}

/// Updater type
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, untagged)]
pub enum Updater {
  /// Generates legacy zipped v1 compatible updaters
  String(V1Compatible),
  /// Produce updaters and their signatures or not
  // Can't use untagged on enum field here: https://github.com/GREsau/schemars/issues/222
  Bool(bool),
}

impl Default for Updater {
  fn default() -> Self {
    Self::Bool(false)
  }
}

/// Generates legacy zipped v1 compatible updaters
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum V1Compatible {
  /// Generates legacy zipped v1 compatible updaters
  V1Compatible,
}

/// Configuration for tauri-bundler.
///
/// See more: <https://v2.tauri.app/reference/config/#bundleconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BundleConfig {
  /// Whether Tauri should bundle your application or just output the executable.
  #[serde(default)]
  pub active: bool,
  /// The bundle targets, currently supports ["deb", "rpm", "appimage", "nsis", "msi", "app", "dmg"] or "all".
  #[serde(default)]
  pub targets: BundleTarget,
  #[serde(default)]
  /// Produce updaters and their signatures or not
  pub create_updater_artifacts: Updater,
  /// The application's publisher. Defaults to the second element in the identifier string.
  ///
  /// Currently maps to the Manufacturer property of the Windows Installer
  /// and the Maintainer field of debian packages if the Cargo.toml does not have the authors field.
  pub publisher: Option<String>,
  /// A url to the home page of your application. If unset, will
  /// fallback to `homepage` defined in `Cargo.toml`.
  ///
  /// Supported bundle targets: `deb`, `rpm`, `nsis` and `msi`.
  pub homepage: Option<String>,
  /// The app's icons
  #[serde(default)]
  pub icon: Vec<String>,
  /// App resources to bundle.
  /// Each resource is a path to a file or directory.
  /// Glob patterns are supported.
  ///
  /// ## Examples
  ///
  /// To include a list of files:
  ///
  /// ```json
  /// {
  ///   "bundle": {
  ///     "resources": [
  ///       "./path/to/some-file.txt",
  ///       "/absolute/path/to/textfile.txt",
  ///       "../relative/path/to/jsonfile.json",
  ///       "some-folder/",
  ///       "resources/**/*.md"
  ///     ]
  ///   }
  /// }
  /// ```
  ///
  /// The bundled files will be in `$RESOURCES/` with the original directory structure preserved,
  /// for example: `./path/to/some-file.txt` -> `$RESOURCE/path/to/some-file.txt`
  ///
  /// To fine control where the files will get copied to, use a map instead
  ///
  /// ```json
  /// {
  ///   "bundle": {
  ///     "resources": {
  ///       "/absolute/path/to/textfile.txt": "resources/textfile.txt",
  ///       "relative/path/to/jsonfile.json": "resources/jsonfile.json",
  ///       "resources/": "",
  ///       "docs/**/*md": "website-docs/"
  ///     }
  ///   }
  /// }
  /// ```
  ///
  /// Note that when using glob pattern in this case, the original directory structure is not preserved,
  /// everything gets copied to the target directory directly
  ///
  /// See more: <https://v2.tauri.app/develop/resources/>
  pub resources: Option<BundleResources>,
  /// A copyright string associated with your application.
  pub copyright: Option<String>,
  /// The package's license identifier to be included in the appropriate bundles.
  /// If not set, defaults to the license from the Cargo.toml file.
  pub license: Option<String>,
  /// The path to the license file to be included in the appropriate bundles.
  #[serde(alias = "license-file")]
  pub license_file: Option<PathBuf>,
  /// The application kind.
  ///
  /// Should be one of the following:
  /// Business, DeveloperTool, Education, Entertainment, Finance, Game, ActionGame, AdventureGame, ArcadeGame, BoardGame, CardGame, CasinoGame, DiceGame, EducationalGame, FamilyGame, KidsGame, MusicGame, PuzzleGame, RacingGame, RolePlayingGame, SimulationGame, SportsGame, StrategyGame, TriviaGame, WordGame, GraphicsAndDesign, HealthcareAndFitness, Lifestyle, Medical, Music, News, Photography, Productivity, Reference, SocialNetworking, Sports, Travel, Utility, Video, Weather.
  pub category: Option<String>,
  /// File types to associate with the application.
  pub file_associations: Option<Vec<FileAssociation>>,
  /// A short description of your application.
  #[serde(alias = "short-description")]
  pub short_description: Option<String>,
  /// A longer, multi-line description of the application.
  #[serde(alias = "long-description")]
  pub long_description: Option<String>,
  /// Whether to use the project's `target` directory, for caching build tools (e.g., Wix and NSIS) when building this application. Defaults to `false`.
  ///
  /// If true, tools will be cached in `target/.tauri/`.
  /// If false, tools will be cached in the current user's platform-specific cache directory.
  ///
  /// An example where it can be appropriate to set this to `true` is when building this application as a Windows System user (e.g., AWS EC2 workloads),
  /// because the Window system's app data directory is restricted.
  #[serde(default, alias = "use-local-tools-dir")]
  pub use_local_tools_dir: bool,
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
  /// Configuration for the Windows bundles.
  #[serde(default)]
  pub windows: WindowsConfig,
  /// Configuration for the Linux bundles.
  #[serde(default)]
  pub linux: LinuxConfig,
  /// Configuration for the macOS bundles.
  #[serde(rename = "macOS", alias = "macos", default)]
  pub macos: MacConfig,
  /// iOS configuration.
  #[serde(rename = "iOS", alias = "ios", default)]
  pub ios: IosConfig,
  /// Android configuration.
  #[serde(default)]
  pub android: AndroidConfig,
}

/// A tuple struct of RGBA colors. Each value has minimum of 0 and maximum of 255.
#[derive(Debug, PartialEq, Eq, Serialize, Default, Clone, Copy)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl From<Color> for (u8, u8, u8, u8) {
  fn from(value: Color) -> Self {
    (value.0, value.1, value.2, value.3)
  }
}

impl From<Color> for (u8, u8, u8) {
  fn from(value: Color) -> Self {
    (value.0, value.1, value.2)
  }
}

impl From<(u8, u8, u8, u8)> for Color {
  fn from(value: (u8, u8, u8, u8)) -> Self {
    Color(value.0, value.1, value.2, value.3)
  }
}

impl From<(u8, u8, u8)> for Color {
  fn from(value: (u8, u8, u8)) -> Self {
    Color(value.0, value.1, value.2, 255)
  }
}

impl From<Color> for [u8; 4] {
  fn from(value: Color) -> Self {
    [value.0, value.1, value.2, value.3]
  }
}

impl From<Color> for [u8; 3] {
  fn from(value: Color) -> Self {
    [value.0, value.1, value.2]
  }
}

impl From<[u8; 4]> for Color {
  fn from(value: [u8; 4]) -> Self {
    Color(value[0], value[1], value[2], value[3])
  }
}

impl From<[u8; 3]> for Color {
  fn from(value: [u8; 3]) -> Self {
    Color(value[0], value[1], value[2], 255)
  }
}

impl FromStr for Color {
  type Err = String;
  fn from_str(mut color: &str) -> Result<Self, Self::Err> {
    color = color.trim().strip_prefix('#').unwrap_or(color);
    let color = match color.len() {
      // TODO: use repeat_n once our MSRV is bumped to 1.82
      3 => color.chars()
            .flat_map(|c| std::iter::repeat(c).take(2))
            .chain(std::iter::repeat('f').take(2))
            .collect(),
      6 => format!("{color}FF"),
      8 => color.to_string(),
      _ => return Err("Invalid hex color length, must be either 3, 6 or 8, for example: #fff, #ffffff, or #ffffffff".into()),
    };

    let r = u8::from_str_radix(&color[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&color[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&color[4..6], 16).map_err(|e| e.to_string())?;
    let a = u8::from_str_radix(&color[6..8], 16).map_err(|e| e.to_string())?;

    Ok(Color(r, g, b, a))
  }
}

fn default_alpha() -> u8 {
  255
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
enum InnerColor {
  /// Color hex string, for example: #fff, #ffffff, or #ffffffff.
  String(String),
  /// Array of RGB colors. Each value has minimum of 0 and maximum of 255.
  Rgb((u8, u8, u8)),
  /// Array of RGBA colors. Each value has minimum of 0 and maximum of 255.
  Rgba((u8, u8, u8, u8)),
  /// Object of red, green, blue, alpha color values. Each value has minimum of 0 and maximum of 255.
  RgbaObject {
    red: u8,
    green: u8,
    blue: u8,
    #[serde(default = "default_alpha")]
    alpha: u8,
  },
}

impl<'de> Deserialize<'de> for Color {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let color = InnerColor::deserialize(deserializer)?;
    let color = match color {
      InnerColor::String(string) => string.parse().map_err(serde::de::Error::custom)?,
      InnerColor::Rgb(rgb) => Color(rgb.0, rgb.1, rgb.2, 255),
      InnerColor::Rgba(rgb) => rgb.into(),
      InnerColor::RgbaObject {
        red,
        green,
        blue,
        alpha,
      } => Color(red, green, blue, alpha),
    };

    Ok(color)
  }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Color {
  fn schema_name() -> String {
    "Color".to_string()
  }

  fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let mut schema = schemars::schema_for!(InnerColor).schema;
    schema.metadata = None; // Remove `title: InnerColor` from schema

    // add hex color pattern validation
    let any_of = schema.subschemas().any_of.as_mut().unwrap();
    let schemars::schema::Schema::Object(str_schema) = any_of.first_mut().unwrap() else {
      unreachable!()
    };
    str_schema.string().pattern = Some("^#?([A-Fa-f0-9]{3}|[A-Fa-f0-9]{6}|[A-Fa-f0-9]{8})$".into());

    schema.into()
  }
}

/// Background throttling policy.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum BackgroundThrottlingPolicy {
  /// A policy where background throttling is disabled
  Disabled,
  /// A policy where a web view that's not in a window fully suspends tasks. This is usually the default behavior in case no policy is set.
  Suspend,
  /// A policy where a web view that's not in a window limits processing, but does not fully suspend tasks.
  Throttle,
}

/// The window effects configuration object
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowEffectsConfig {
  /// List of Window effects to apply to the Window.
  /// Conflicting effects will apply the first one and ignore the rest.
  pub effects: Vec<WindowEffect>,
  /// Window effect state **macOS Only**
  pub state: Option<WindowEffectState>,
  /// Window effect corner radius **macOS Only**
  pub radius: Option<f64>,
  /// Window effect color. Affects [`WindowEffect::Blur`] and [`WindowEffect::Acrylic`] only
  /// on Windows 10 v1903+. Doesn't have any effect on Windows 7 or Windows 11.
  pub color: Option<Color>,
}

/// Enable prevent overflow with a margin
/// so that the window's size + this margin won't overflow the workarea
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreventOverflowMargin {
  /// Horizontal margin in physical unit
  pub width: u32,
  /// Vertical margin in physical unit
  pub height: u32,
}

/// Prevent overflow with a margin
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PreventOverflowConfig {
  /// Enable prevent overflow or not
  Enable(bool),
  /// Enable prevent overflow with a margin
  /// so that the window's size + this margin won't overflow the workarea
  Margin(PreventOverflowMargin),
}

/// The scrollbar style to use in the webview.
///
/// ## Platform-specific
///
/// - **Windows**: This option must be given the same value for all webviews that target the same data directory.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub enum ScrollBarStyle {
  #[default]
  /// The scrollbar style to use in the webview.
  Default,

  /// Fluent UI style overlay scrollbars. **Windows Only**
  ///
  /// Requires WebView2 Runtime version 125.0.2535.41 or higher, does nothing on older versions,
  /// see https://learn.microsoft.com/en-us/microsoft-edge/webview2/release-notes/?tabs=dotnetcsharp#10253541
  FluentOverlay,
}

/// The window configuration object.
///
/// See more: <https://v2.tauri.app/reference/config/#windowconfig>
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowConfig {
  /// The window identifier. It must be alphanumeric.
  #[serde(default = "default_window_label")]
  pub label: String,
  /// Whether Tauri should create this window at app startup or not.
  ///
  /// When this is set to `false` you must manually grab the config object via `app.config().app.windows`
  /// and create it with [`WebviewWindowBuilder::from_config`](https://docs.rs/tauri/2/tauri/webview/struct.WebviewWindowBuilder.html#method.from_config).
  ///
  /// ## Example:
  ///
  /// ```rust
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     tauri::WebviewWindowBuilder::from_config(app.handle(), app.config().app.windows[0])?.build()?;
  ///     Ok(())
  ///   });
  /// ```
  #[serde(default = "default_true")]
  pub create: bool,
  /// The window webview URL.
  #[serde(default)]
  pub url: WebviewUrl,
  /// The user agent for the webview
  #[serde(alias = "user-agent")]
  pub user_agent: Option<String>,
  /// Whether the drag and drop is enabled or not on the webview. By default it is enabled.
  ///
  /// Disabling it is required to use HTML5 drag and drop on the frontend on Windows.
  #[serde(default = "default_true", alias = "drag-drop-enabled")]
  pub drag_drop_enabled: bool,
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
  /// Whether or not to prevent the window from overflowing the workarea
  ///
  /// ## Platform-specific
  ///
  /// - **iOS / Android:** Unsupported.
  #[serde(alias = "prevent-overflow")]
  pub prevent_overflow: Option<PreventOverflowConfig>,
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
  /// Whether the window will be focusable or not.
  #[serde(default = "default_true")]
  pub focusable: bool,
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
  /// Whether the window should always be below other windows.
  #[serde(default, alias = "always-on-bottom")]
  pub always_on_bottom: bool,
  /// Whether the window should always be on top of other windows.
  #[serde(default, alias = "always-on-top")]
  pub always_on_top: bool,
  /// Whether the window should be visible on all workspaces or virtual desktops.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / iOS / Android:** Unsupported.
  #[serde(default, alias = "visible-on-all-workspaces")]
  pub visible_on_all_workspaces: bool,
  /// Prevents the window contents from being captured by other apps.
  #[serde(default, alias = "content-protected")]
  pub content_protected: bool,
  /// If `true`, hides the window icon from the taskbar on Windows and Linux.
  #[serde(default, alias = "skip-taskbar")]
  pub skip_taskbar: bool,
  /// The name of the window class created on Windows to create the window. **Windows only**.
  pub window_classname: Option<String>,
  /// The initial window theme. Defaults to the system theme. Only implemented on Windows and macOS 10.14+.
  pub theme: Option<crate::Theme>,
  /// The style of the macOS title bar.
  #[serde(default, alias = "title-bar-style")]
  pub title_bar_style: TitleBarStyle,
  /// The position of the window controls on macOS.
  ///
  /// Requires titleBarStyle: Overlay and decorations: true.
  #[serde(default, alias = "traffic-light-position")]
  pub traffic_light_position: Option<LogicalPosition>,
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
  /// Whether or not the window has shadow.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:**
  ///   - `false` has no effect on decorated window, shadow are always ON.
  ///   - `true` will make undecorated window have a 1px white border,
  /// and on Windows 11, it will have a rounded corners.
  /// - **Linux:** Unsupported.
  #[serde(default = "default_true")]
  pub shadow: bool,
  /// Window effects.
  ///
  /// Requires the window to be transparent.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: If using decorations or shadows, you may want to try this workaround <https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891>
  /// - **Linux**: Unsupported
  #[serde(default, alias = "window-effects")]
  pub window_effects: Option<WindowEffectsConfig>,
  /// Whether or not the webview should be launched in incognito  mode.
  ///
  /// ## Platform-specific:
  ///
  /// - **Android**: Unsupported.
  #[serde(default)]
  pub incognito: bool,
  /// Sets the window associated with this label to be the parent of the window to be created.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: This sets the passed parent as an owner window to the window to be created.
  ///   From [MSDN owned windows docs](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows):
  ///     - An owned window is always above its owner in the z-order.
  ///     - The system automatically destroys an owned window when its owner is destroyed.
  ///     - An owned window is hidden when its owner is minimized.
  /// - **Linux**: This makes the new window transient for parent, see <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  /// - **macOS**: This adds the window as a child of parent, see <https://developer.apple.com/documentation/appkit/nswindow/1419152-addchildwindow?language=objc>
  pub parent: Option<String>,
  /// The proxy URL for the WebView for all network requests.
  ///
  /// Must be either a `http://` or a `socks5://` URL.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Requires the `macos-proxy` feature flag and only compiles for macOS 14+.
  #[serde(alias = "proxy-url")]
  pub proxy_url: Option<Url>,
  /// Whether page zooming by hotkeys is enabled
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Controls WebView2's [`IsZoomControlEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.2420.47#iszoomcontrolenabled) setting.
  /// - **MacOS / Linux**: Injects a polyfill that zooms in and out with `ctrl/command` + `-/=`,
  /// 20% in each step, ranging from 20% to 1000%. Requires `webview:allow-set-webview-zoom` permission
  ///
  /// - **Android / iOS**: Unsupported.
  #[serde(default, alias = "zoom-hotkeys-enabled")]
  pub zoom_hotkeys_enabled: bool,
  /// Whether browser extensions can be installed for the webview process
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Enables the WebView2 environment's [`AreBrowserExtensionsEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2environmentoptions?view=webview2-winrt-1.0.2739.15#arebrowserextensionsenabled)
  /// - **MacOS / Linux / iOS / Android** - Unsupported.
  #[serde(default, alias = "browser-extensions-enabled")]
  pub browser_extensions_enabled: bool,

  /// Sets whether the custom protocols should use `https://<scheme>.localhost` instead of the default `http://<scheme>.localhost` on Windows and Android. Defaults to `false`.
  ///
  /// ## Note
  ///
  /// Using a `https` scheme will NOT allow mixed content when trying to fetch `http` endpoints and therefore will not match the behavior of the `<scheme>://localhost` protocols used on macOS and Linux.
  ///
  /// ## Warning
  ///
  /// Changing this value between releases will change the IndexedDB, cookies and localstorage location and your app will not be able to access the old data.
  #[serde(default, alias = "use-https-scheme")]
  pub use_https_scheme: bool,
  /// Enable web inspector which is usually called browser devtools. Enabled by default.
  ///
  /// This API works in **debug** builds, but requires `devtools` feature flag to enable it in **release** builds.
  ///
  /// ## Platform-specific
  ///
  /// - macOS: This will call private functions on **macOS**.
  /// - Android: Open `chrome://inspect/#devices` in Chrome to get the devtools window. Wry's `WebView` devtools API isn't supported on Android.
  /// - iOS: Open Safari > Develop > [Your Device Name] > [Your WebView] to get the devtools window.
  pub devtools: Option<bool>,

  /// Set the window and webview background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: alpha channel is ignored for the window layer.
  /// - **Windows**: On Windows 7, alpha channel is ignored for the webview layer.
  /// - **Windows**: On Windows 8 and newer, if alpha channel is not `0`, it will be ignored for the webview layer.
  #[serde(alias = "background-color")]
  pub background_color: Option<Color>,

  /// Change the default background throttling behaviour.
  ///
  /// By default, browsers use a suspend policy that will throttle timers and even unload
  /// the whole tab (view) to free resources after roughly 5 minutes when a view became
  /// minimized or hidden. This will pause all tasks until the documents visibility state
  /// changes back from hidden to visible by bringing the view back to the foreground.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / Windows / Android**: Unsupported. Workarounds like a pending WebLock transaction might suffice.
  /// - **iOS**: Supported since version 17.0+.
  /// - **macOS**: Supported since version 14.0+.
  ///
  /// see https://github.com/tauri-apps/tauri/issues/5250#issuecomment-2569380578
  #[serde(default, alias = "background-throttling")]
  pub background_throttling: Option<BackgroundThrottlingPolicy>,
  /// Whether we should disable JavaScript code execution on the webview or not.
  #[serde(default, alias = "javascript-disabled")]
  pub javascript_disabled: bool,
  /// on macOS and iOS there is a link preview on long pressing links, this is enabled by default.
  /// see https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html#method.allowsLinkPreview
  #[serde(default = "default_true", alias = "allow-link-preview")]
  pub allow_link_preview: bool,
  /// Allows disabling the input accessory view on iOS.
  ///
  /// The accessory view is the view that appears above the keyboard when a text input element is focused.
  /// It usually displays a view with "Done", "Next" buttons.
  #[serde(
    default,
    alias = "disable-input-accessory-view",
    alias = "disable_input_accessory_view"
  )]
  pub disable_input_accessory_view: bool,
  ///
  /// Set a custom path for the webview's data directory (localStorage, cache, etc.) **relative to [`appDataDir()`]/${label}**.
  ///
  /// To set absolute paths, use [`WebviewWindowBuilder::data_directory`](https://docs.rs/tauri/2/tauri/webview/struct.WebviewWindowBuilder.html#method.data_directory)
  ///
  /// #### Platform-specific:
  ///
  /// - **Windows**: WebViews with different values for settings like `additionalBrowserArgs`, `browserExtensionsEnabled` or `scrollBarStyle` must have different data directories.
  /// - **macOS / iOS**: Unsupported, use `dataStoreIdentifier` instead.
  /// - **Android**: Unsupported.
  #[serde(default, alias = "data-directory")]
  pub data_directory: Option<PathBuf>,
  ///
  /// Initialize the WebView with a custom data store identifier. This can be seen as a replacement for `dataDirectory` which is unavailable in WKWebView.
  /// See https://developer.apple.com/documentation/webkit/wkwebsitedatastore/init(foridentifier:)?language=objc
  ///
  /// The array must contain 16 u8 numbers.
  ///
  /// #### Platform-specific:
  ///
  /// - **iOS**: Supported since version 17.0+.
  /// - **macOS**: Supported since version 14.0+.
  /// - **Windows / Linux / Android**: Unsupported.
  #[serde(default, alias = "data-store-identifier")]
  pub data_store_identifier: Option<[u8; 16]>,

  /// Specifies the native scrollbar style to use with the webview.
  /// CSS styles that modify the scrollbar are applied on top of the native appearance configured here.
  ///
  /// Defaults to `default`, which is the browser default.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**:
  ///   - `fluentOverlay` requires WebView2 Runtime version 125.0.2535.41 or higher,
  ///     and does nothing on older versions.
  ///   - This option must be given the same value for all webviews that target the same data directory.
  /// - **Linux / Android / iOS / macOS**: Unsupported. Only supports `Default` and performs no operation.
  #[serde(default, alias = "scroll-bar-style")]
  pub scroll_bar_style: ScrollBarStyle,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      label: default_window_label(),
      url: WebviewUrl::default(),
      create: true,
      user_agent: None,
      drag_drop_enabled: true,
      center: false,
      x: None,
      y: None,
      width: default_width(),
      height: default_height(),
      min_width: None,
      min_height: None,
      max_width: None,
      max_height: None,
      prevent_overflow: None,
      resizable: true,
      maximizable: true,
      minimizable: true,
      closable: true,
      title: default_title(),
      fullscreen: false,
      focus: false,
      focusable: true,
      transparent: false,
      maximized: false,
      visible: true,
      decorations: true,
      always_on_bottom: false,
      always_on_top: false,
      visible_on_all_workspaces: false,
      content_protected: false,
      skip_taskbar: false,
      window_classname: None,
      theme: None,
      title_bar_style: Default::default(),
      traffic_light_position: None,
      hidden_title: false,
      accept_first_mouse: false,
      tabbing_identifier: None,
      additional_browser_args: None,
      shadow: true,
      window_effects: None,
      incognito: false,
      parent: None,
      proxy_url: None,
      zoom_hotkeys_enabled: false,
      browser_extensions_enabled: false,
      use_https_scheme: false,
      devtools: None,
      background_color: None,
      background_throttling: None,
      javascript_disabled: false,
      allow_link_preview: true,
      disable_input_accessory_view: false,
      data_directory: None,
      data_store_identifier: None,
      scroll_bar_style: ScrollBarStyle::Default,
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

  /// Extends this CSP directive source list with the given array of sources.
  pub fn extend(&mut self, sources: Vec<String>) {
    for s in sources {
      self.push(s);
    }
  }
}

/// A Content-Security-Policy definition.
/// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum DisabledCspModificationKind {
  /// If `true`, disables all CSP modification.
  /// `false` is the default value and it configures Tauri to control the CSP.
  Flag(bool),
  /// Disables the given list of CSP directives modifications.
  List(Vec<String>),
}

impl DisabledCspModificationKind {
  /// Determines whether the given CSP directive can be modified or not.
  pub fn can_modify(&self, directive: &str) -> bool {
    match self {
      Self::Flag(f) => !f,
      Self::List(l) => !l.contains(&directive.into()),
    }
  }
}

impl Default for DisabledCspModificationKind {
  fn default() -> Self {
    Self::Flag(false)
  }
}

/// Protocol scope definition.
/// It is a list of glob patterns that restrict the API access from the webview.
///
/// Each pattern can start with a variable that resolves to a system base directory.
/// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
/// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
/// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
/// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum FsScope {
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

impl Default for FsScope {
  fn default() -> Self {
    Self::AllowedPaths(Vec::new())
  }
}

impl FsScope {
  /// The list of allowed paths.
  pub fn allowed_paths(&self) -> &Vec<PathBuf> {
    match self {
      Self::AllowedPaths(p) => p,
      Self::Scope { allow, .. } => allow,
    }
  }

  /// The list of forbidden paths.
  pub fn forbidden_paths(&self) -> Option<&Vec<PathBuf>> {
    match self {
      Self::AllowedPaths(_) => None,
      Self::Scope { deny, .. } => Some(deny),
    }
  }
}

/// Config for the asset custom protocol.
///
/// See more: <https://v2.tauri.app/reference/config/#assetprotocolconfig>
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetProtocolConfig {
  /// The access scope for the asset protocol.
  #[serde(default)]
  pub scope: FsScope,
  /// Enables the asset protocol.
  #[serde(default)]
  pub enable: bool,
}

/// definition of a header source
///
/// The header value to a header name
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum HeaderSource {
  /// string version of the header Value
  Inline(String),
  /// list version of the header value. Item are joined by "," for the real header value
  List(Vec<String>),
  /// (Rust struct | Json | JavaScript Object) equivalent of the header value. Items are composed from: key + space + value. Item are then joined by ";" for the real header value
  Map(HashMap<String, String>),
}

impl Display for HeaderSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Inline(s) => write!(f, "{s}"),
      Self::List(l) => write!(f, "{}", l.join(", ")),
      Self::Map(m) => {
        let len = m.len();
        let mut i = 0;
        for (key, value) in m {
          write!(f, "{key} {value}")?;
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

/// A trait which implements on the [`Builder`] of the http create
///
/// Must add headers defined in the tauri configuration file to http responses
pub trait HeaderAddition {
  /// adds all headers defined on the config file, given the current HeaderConfig
  fn add_configured_headers(self, headers: Option<&HeaderConfig>) -> http::response::Builder;
}

impl HeaderAddition for Builder {
  /// Add the headers defined in the tauri configuration file to http responses
  ///
  /// this is a utility function, which is used in the same way as the `.header(..)` of the rust http library
  fn add_configured_headers(mut self, headers: Option<&HeaderConfig>) -> http::response::Builder {
    if let Some(headers) = headers {
      // Add the header Access-Control-Allow-Credentials, if we find a value for it
      if let Some(value) = &headers.access_control_allow_credentials {
        self = self.header("Access-Control-Allow-Credentials", value.to_string());
      };

      // Add the header Access-Control-Allow-Headers, if we find a value for it
      if let Some(value) = &headers.access_control_allow_headers {
        self = self.header("Access-Control-Allow-Headers", value.to_string());
      };

      // Add the header Access-Control-Allow-Methods, if we find a value for it
      if let Some(value) = &headers.access_control_allow_methods {
        self = self.header("Access-Control-Allow-Methods", value.to_string());
      };

      // Add the header Access-Control-Expose-Headers, if we find a value for it
      if let Some(value) = &headers.access_control_expose_headers {
        self = self.header("Access-Control-Expose-Headers", value.to_string());
      };

      // Add the header Access-Control-Max-Age, if we find a value for it
      if let Some(value) = &headers.access_control_max_age {
        self = self.header("Access-Control-Max-Age", value.to_string());
      };

      // Add the header Cross-Origin-Embedder-Policy, if we find a value for it
      if let Some(value) = &headers.cross_origin_embedder_policy {
        self = self.header("Cross-Origin-Embedder-Policy", value.to_string());
      };

      // Add the header Cross-Origin-Opener-Policy, if we find a value for it
      if let Some(value) = &headers.cross_origin_opener_policy {
        self = self.header("Cross-Origin-Opener-Policy", value.to_string());
      };

      // Add the header Cross-Origin-Resource-Policy, if we find a value for it
      if let Some(value) = &headers.cross_origin_resource_policy {
        self = self.header("Cross-Origin-Resource-Policy", value.to_string());
      };

      // Add the header Permission-Policy, if we find a value for it
      if let Some(value) = &headers.permissions_policy {
        self = self.header("Permission-Policy", value.to_string());
      };

      if let Some(value) = &headers.service_worker_allowed {
        self = self.header("Service-Worker-Allowed", value.to_string());
      }

      // Add the header Timing-Allow-Origin, if we find a value for it
      if let Some(value) = &headers.timing_allow_origin {
        self = self.header("Timing-Allow-Origin", value.to_string());
      };

      // Add the header X-Content-Type-Options, if we find a value for it
      if let Some(value) = &headers.x_content_type_options {
        self = self.header("X-Content-Type-Options", value.to_string());
      };

      // Add the header Tauri-Custom-Header, if we find a value for it
      if let Some(value) = &headers.tauri_custom_header {
        // Keep in mind to correctly set the Access-Control-Expose-Headers
        self = self.header("Tauri-Custom-Header", value.to_string());
      };
    }
    self
  }
}

/// A struct, where the keys are some specific http header names.
///
/// If the values to those keys are defined, then they will be send as part of a response message.
/// This does not include error messages and ipc messages
///
/// ## Example configuration
/// ```javascript
/// {
///  //..
///   app:{
///     //..
///     security: {
///       headers: {
///         "Cross-Origin-Opener-Policy": "same-origin",
///         "Cross-Origin-Embedder-Policy": "require-corp",
///         "Timing-Allow-Origin": [
///           "https://developer.mozilla.org",
///           "https://example.com",
///         ],
///         "Access-Control-Expose-Headers": "Tauri-Custom-Header",
///         "Tauri-Custom-Header": {
///           "key1": "'value1' 'value2'",
///           "key2": "'value3'"
///         }
///       },
///       csp: "default-src 'self'; connect-src ipc: http://ipc.localhost",
///     }
///     //..
///   }
///  //..
/// }
/// ```
/// In this example `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` are set to allow for the use of [`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer).
/// The result is, that those headers are then set on every response sent via the `get_response` function in crates/tauri/src/protocol/tauri.rs.
/// The Content-Security-Policy header is defined separately, because it is also handled separately.
///
/// For the helloworld example, this config translates into those response headers:
/// ```http
/// access-control-allow-origin:  http://tauri.localhost
/// access-control-expose-headers: Tauri-Custom-Header
/// content-security-policy: default-src 'self'; connect-src ipc: http://ipc.localhost; script-src 'self' 'sha256-Wjjrs6qinmnr+tOry8x8PPwI77eGpUFR3EEGZktjJNs='
/// content-type: text/html
/// cross-origin-embedder-policy: require-corp
/// cross-origin-opener-policy: same-origin
/// tauri-custom-header: key1 'value1' 'value2'; key2 'value3'
/// timing-allow-origin: https://developer.mozilla.org, https://example.com
/// ```
/// Since the resulting header values are always 'string-like'. So depending on the what data type the HeaderSource is, they need to be converted.
///  - `String`(JS/Rust): stay the same for the resulting header value
///  - `Array`(JS)/`Vec\<String\>`(Rust): Item are joined by ", " for the resulting header value
///  - `Object`(JS)/ `Hashmap\<String,String\>`(Rust): Items are composed from: key + space + value. Item are then joined by "; " for the resulting header value
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct HeaderConfig {
  /// The Access-Control-Allow-Credentials response header tells browsers whether the
  /// server allows cross-origin HTTP requests to include credentials.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials>
  #[serde(rename = "Access-Control-Allow-Credentials")]
  pub access_control_allow_credentials: Option<HeaderSource>,
  /// The Access-Control-Allow-Headers response header is used in response
  /// to a preflight request which includes the Access-Control-Request-Headers
  /// to indicate which HTTP headers can be used during the actual request.
  ///
  /// This header is required if the request has an Access-Control-Request-Headers header.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers>
  #[serde(rename = "Access-Control-Allow-Headers")]
  pub access_control_allow_headers: Option<HeaderSource>,
  /// The Access-Control-Allow-Methods response header specifies one or more methods
  /// allowed when accessing a resource in response to a preflight request.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods>
  #[serde(rename = "Access-Control-Allow-Methods")]
  pub access_control_allow_methods: Option<HeaderSource>,
  /// The Access-Control-Expose-Headers response header allows a server to indicate
  /// which response headers should be made available to scripts running in the browser,
  /// in response to a cross-origin request.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers>
  #[serde(rename = "Access-Control-Expose-Headers")]
  pub access_control_expose_headers: Option<HeaderSource>,
  /// The Access-Control-Max-Age response header indicates how long the results of a
  /// preflight request (that is the information contained in the
  /// Access-Control-Allow-Methods and Access-Control-Allow-Headers headers) can
  /// be cached.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age>
  #[serde(rename = "Access-Control-Max-Age")]
  pub access_control_max_age: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Embedder-Policy (COEP) response header configures embedding
  /// cross-origin resources into the document.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Embedder-Policy>
  #[serde(rename = "Cross-Origin-Embedder-Policy")]
  pub cross_origin_embedder_policy: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Opener-Policy (COOP) response header allows you to ensure a
  /// top-level document does not share a browsing context group with cross-origin documents.
  /// COOP will process-isolate your document and potential attackers can't access your global
  /// object if they were to open it in a popup, preventing a set of cross-origin attacks dubbed XS-Leaks.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy>
  #[serde(rename = "Cross-Origin-Opener-Policy")]
  pub cross_origin_opener_policy: Option<HeaderSource>,
  /// The HTTP Cross-Origin-Resource-Policy response header conveys a desire that the
  /// browser blocks no-cors cross-origin/cross-site requests to the given resource.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Resource-Policy>
  #[serde(rename = "Cross-Origin-Resource-Policy")]
  pub cross_origin_resource_policy: Option<HeaderSource>,
  /// The HTTP Permissions-Policy header provides a mechanism to allow and deny the
  /// use of browser features in a document or within any \<iframe\> elements in the document.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Permissions-Policy>
  #[serde(rename = "Permissions-Policy")]
  pub permissions_policy: Option<HeaderSource>,
  /// The HTTP Service-Worker-Allowed response header is used to broaden the path restriction for a
  /// service worker's default scope.
  ///
  /// By default, the scope for a service worker registration is the directory where the service
  /// worker script is located. For example, if the script `sw.js` is located in `/js/sw.js`,
  /// it can only control URLs under `/js/` by default. Servers can use the `Service-Worker-Allowed`
  /// header to allow a service worker to control URLs outside of its own directory.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Service-Worker-Allowed>
  #[serde(rename = "Service-Worker-Allowed")]
  pub service_worker_allowed: Option<HeaderSource>,
  /// The Timing-Allow-Origin response header specifies origins that are allowed to see values
  /// of attributes retrieved via features of the Resource Timing API, which would otherwise be
  /// reported as zero due to cross-origin restrictions.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Timing-Allow-Origin>
  #[serde(rename = "Timing-Allow-Origin")]
  pub timing_allow_origin: Option<HeaderSource>,
  /// The X-Content-Type-Options response HTTP header is a marker used by the server to indicate
  /// that the MIME types advertised in the Content-Type headers should be followed and not be
  /// changed. The header allows you to avoid MIME type sniffing by saying that the MIME types
  /// are deliberately configured.
  ///
  /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options>
  #[serde(rename = "X-Content-Type-Options")]
  pub x_content_type_options: Option<HeaderSource>,
  /// A custom header field Tauri-Custom-Header, don't use it.
  /// Remember to set Access-Control-Expose-Headers accordingly
  ///
  /// **NOT INTENDED FOR PRODUCTION USE**
  #[serde(rename = "Tauri-Custom-Header")]
  pub tauri_custom_header: Option<HeaderSource>,
}

impl HeaderConfig {
  /// creates a new header config
  pub fn new() -> Self {
    HeaderConfig {
      access_control_allow_credentials: None,
      access_control_allow_methods: None,
      access_control_allow_headers: None,
      access_control_expose_headers: None,
      access_control_max_age: None,
      cross_origin_embedder_policy: None,
      cross_origin_opener_policy: None,
      cross_origin_resource_policy: None,
      permissions_policy: None,
      service_worker_allowed: None,
      timing_allow_origin: None,
      x_content_type_options: None,
      tauri_custom_header: None,
    }
  }
}

/// Security configuration.
///
/// See more: <https://v2.tauri.app/reference/config/#securityconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  /// Custom protocol config.
  #[serde(default, alias = "asset-protocol")]
  pub asset_protocol: AssetProtocolConfig,
  /// The pattern to use.
  #[serde(default)]
  pub pattern: PatternKind,
  /// List of capabilities that are enabled on the application.
  ///
  /// By default (not set or empty list), all capability files from `./capabilities/` are included,
  /// by setting values in this entry, you have fine grained control over which capabilities are included
  ///
  /// You can either reference a capability file defined in `./capabilities/` with its identifier or inline a [`Capability`]
  ///
  /// ### Example
  ///
  /// ```json
  /// {
  ///   "app": {
  ///     "capabilities": [
  ///       "main-window",
  ///       {
  ///         "identifier": "drag-window",
  ///         "permissions": ["core:window:allow-start-dragging"]
  ///       }
  ///     ]
  ///   }
  /// }
  /// ```
  #[serde(default)]
  pub capabilities: Vec<CapabilityEntry>,
  /// The headers, which are added to every http response from tauri to the web view
  /// This doesn't include IPC Messages and error responses
  #[serde(default)]
  pub headers: Option<HeaderConfig>,
}

/// A capability entry which can be either an inlined capability or a reference to a capability defined on its own file.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum CapabilityEntry {
  /// An inlined capability.
  Inlined(Capability),
  /// Reference to a capability identifier.
  Reference(String),
}

impl<'de> Deserialize<'de> for CapabilityEntry {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    UntaggedEnumVisitor::new()
      .string(|string| Ok(Self::Reference(string.to_owned())))
      .map(|map| map.deserialize::<Capability>().map(Self::Inlined))
      .deserialize(deserializer)
  }
}

/// The application pattern.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase", tag = "use", content = "options")]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

/// The App configuration object.
///
/// See more: <https://v2.tauri.app/reference/config/#appconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
  /// The app windows configuration.
  ///
  /// ## Example:
  ///
  /// To create a window at app startup
  ///
  /// ```json
  /// {
  ///   "app": {
  ///     "windows": [
  ///       { "width": 800, "height": 600 }
  ///     ]
  ///   }
  /// }
  /// ```
  ///
  /// If not specified, the window's label (its identifier) defaults to "main",
  /// you can use this label to get the window through
  /// `app.get_webview_window` in Rust or `WebviewWindow.getByLabel` in JavaScript
  ///
  /// When working with multiple windows, each window will need an unique label
  ///
  /// ```json
  /// {
  ///   "app": {
  ///     "windows": [
  ///       { "label": "main", "width": 800, "height": 600 },
  ///       { "label": "secondary", "width": 800, "height": 600 }
  ///     ]
  ///   }
  /// }
  /// ```
  ///
  /// You can also set `create` to false and use this config through the Rust APIs
  ///
  /// ```json
  /// {
  ///   "app": {
  ///     "windows": [
  ///       { "create": false, "width": 800, "height": 600 }
  ///     ]
  ///   }
  /// }
  /// ```
  ///
  /// and use it like this
  ///
  /// ```rust
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     tauri::WebviewWindowBuilder::from_config(app.handle(), app.config().app.windows[0])?.build()?;
  ///     Ok(())
  ///   });
  /// ```
  #[serde(default)]
  pub windows: Vec<WindowConfig>,
  /// Security configuration.
  #[serde(default)]
  pub security: SecurityConfig,
  /// Configuration for app tray icon.
  #[serde(alias = "tray-icon")]
  pub tray_icon: Option<TrayIconConfig>,
  /// MacOS private API configuration. Enables the transparent background API and sets the `fullScreenEnabled` preference to `true`.
  #[serde(rename = "macOSPrivateApi", alias = "macos-private-api", default)]
  pub macos_private_api: bool,
  /// Whether we should inject the Tauri API on `window.__TAURI__` or not.
  #[serde(default, alias = "with-global-tauri")]
  pub with_global_tauri: bool,
  /// If set to true "identifier" will be set as GTK app ID (on systems that use GTK).
  #[serde(rename = "enableGTKAppId", alias = "enable-gtk-app-id", default)]
  pub enable_gtk_app_id: bool,
}

impl AppConfig {
  /// Returns all Cargo features.
  pub fn all_features() -> Vec<&'static str> {
    vec![
      "tray-icon",
      "macos-private-api",
      "protocol-asset",
      "isolation",
    ]
  }

  /// Returns the enabled Cargo features.
  pub fn features(&self) -> Vec<&str> {
    let mut features = Vec::new();
    if self.tray_icon.is_some() {
      features.push("tray-icon");
    }
    if self.macos_private_api {
      features.push("macos-private-api");
    }
    if self.security.asset_protocol.enable {
      features.push("protocol-asset");
    }

    if let PatternKind::Isolation { .. } = self.security.pattern {
      features.push("isolation");
    }

    features.sort_unstable();
    features
  }
}

/// Configuration for application tray icon.
///
/// See more: <https://v2.tauri.app/reference/config/#trayiconconfig>
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrayIconConfig {
  /// Set an id for this tray icon so you can reference it later, defaults to `main`.
  pub id: Option<String>,
  /// Path to the default icon to use for the tray icon.
  ///
  /// Note: this stores the image in raw pixels to the final binary,
  /// so keep the icon size (width and height) small
  /// or else it's going to bloat your final executable
  #[serde(alias = "icon-path")]
  pub icon_path: PathBuf,
  /// A Boolean value that determines whether the image represents a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image on macOS.
  #[serde(default, alias = "icon-as-template")]
  pub icon_as_template: bool,
  /// A Boolean value that determines whether the menu should appear when the tray icon receives a left click.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: Unsupported.
  #[serde(default = "default_true", alias = "menu-on-left-click")]
  #[deprecated(since = "2.2.0", note = "Use `show_menu_on_left_click` instead.")]
  pub menu_on_left_click: bool,
  /// A Boolean value that determines whether the menu should appear when the tray icon receives a left click.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: Unsupported.
  #[serde(default = "default_true", alias = "show-menu-on-left-click")]
  pub show_menu_on_left_click: bool,
  /// Title for MacOS tray
  pub title: Option<String>,
  /// Tray icon tooltip on Windows and macOS
  pub tooltip: Option<String>,
}

/// General configuration for the iOS target.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IosConfig {
  /// A custom [XcodeGen] project.yml template to use.
  ///
  /// [XcodeGen]: <https://github.com/yonaskolb/XcodeGen>
  pub template: Option<PathBuf>,
  /// A list of strings indicating any iOS frameworks that need to be bundled with the application.
  ///
  /// Note that you need to recreate the iOS project for the changes to be applied.
  pub frameworks: Option<Vec<String>>,
  /// The development team. This value is required for iOS development because code signing is enforced.
  /// The `APPLE_DEVELOPMENT_TEAM` environment variable can be set to overwrite it.
  #[serde(alias = "development-team")]
  pub development_team: Option<String>,
  /// The version of the build that identifies an iteration of the bundle.
  ///
  /// Translates to the bundle's CFBundleVersion property.
  #[serde(alias = "bundle-version")]
  pub bundle_version: Option<String>,
  /// A version string indicating the minimum iOS version that the bundled application supports. Defaults to `13.0`.
  ///
  /// Maps to the IPHONEOS_DEPLOYMENT_TARGET value.
  #[serde(
    alias = "minimum-system-version",
    default = "ios_minimum_system_version"
  )]
  pub minimum_system_version: String,
  /// Path to a Info.plist file to merge with the default Info.plist.
  ///
  /// Note that Tauri also looks for a `Info.plist` and `Info.ios.plist` file in the same directory as the Tauri configuration file.
  #[serde(alias = "info-plist")]
  pub info_plist: Option<PathBuf>,
}

impl Default for IosConfig {
  fn default() -> Self {
    Self {
      template: None,
      frameworks: None,
      development_team: None,
      bundle_version: None,
      minimum_system_version: ios_minimum_system_version(),
      info_plist: None,
    }
  }
}

/// General configuration for the Android target.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AndroidConfig {
  /// The minimum API level required for the application to run.
  /// The Android system will prevent the user from installing the application if the system's API level is lower than the value specified.
  #[serde(alias = "min-sdk-version", default = "default_min_sdk_version")]
  pub min_sdk_version: u32,

  /// The version code of the application.
  /// It is limited to 2,100,000,000 as per Google Play Store requirements.
  ///
  /// By default we use your configured version and perform the following math:
  /// versionCode = version.major * 1000000 + version.minor * 1000 + version.patch
  #[serde(alias = "version-code")]
  #[cfg_attr(feature = "schema", validate(range(min = 1, max = 2_100_000_000)))]
  pub version_code: Option<u32>,

  /// Whether to automatically increment the `versionCode` on each build.
  ///
  /// - If `true`, the generator will try to read the last `versionCode` from
  ///   `tauri.properties` and increment it by 1 for every build.
  /// - If `false` or not set, it falls back to `version_code` or semver-derived logic.
  ///
  /// Note that to use this feature, you should remove `/tauri.properties` from `src-tauri/gen/android/app/.gitignore` so the current versionCode is committed to the repository.
  #[serde(alias = "auto-increment-version-code", default)]
  pub auto_increment_version_code: bool,
}

impl Default for AndroidConfig {
  fn default() -> Self {
    Self {
      min_sdk_version: default_min_sdk_version(),
      version_code: None,
      auto_increment_version_code: false,
    }
  }
}

fn default_min_sdk_version() -> u32 {
  24
}

/// Defines the URL or assets to embed in the application.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum FrontendDist {
  /// An external URL that should be used as the default application URL.
  Url(Url),
  /// Path to a directory containing the frontend dist assets.
  Directory(PathBuf),
  /// An array of files to embed on the app.
  Files(Vec<PathBuf>),
}

impl std::fmt::Display for FrontendDist {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Url(url) => write!(f, "{url}"),
      Self::Directory(p) => write!(f, "{}", p.display()),
      Self::Files(files) => write!(f, "{}", serde_json::to_string(files).unwrap()),
    }
  }
}

/// Describes the shell command to run before `tauri dev`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

/// The runner configuration.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum RunnerConfig {
  /// A string specifying the binary to run.
  String(String),
  /// An object with advanced configuration options.
  Object {
    /// The binary to run.
    cmd: String,
    /// The current working directory to run the command from.
    cwd: Option<String>,
    /// Arguments to pass to the command.
    args: Option<Vec<String>>,
  },
}

impl Default for RunnerConfig {
  fn default() -> Self {
    RunnerConfig::String("cargo".to_string())
  }
}

impl RunnerConfig {
  /// Returns the command to run.
  pub fn cmd(&self) -> &str {
    match self {
      RunnerConfig::String(cmd) => cmd,
      RunnerConfig::Object { cmd, .. } => cmd,
    }
  }

  /// Returns the working directory.
  pub fn cwd(&self) -> Option<&str> {
    match self {
      RunnerConfig::String(_) => None,
      RunnerConfig::Object { cwd, .. } => cwd.as_deref(),
    }
  }

  /// Returns the arguments.
  pub fn args(&self) -> Option<&[String]> {
    match self {
      RunnerConfig::String(_) => None,
      RunnerConfig::Object { args, .. } => args.as_deref(),
    }
  }
}

impl std::str::FromStr for RunnerConfig {
  type Err = std::convert::Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(RunnerConfig::String(s.to_string()))
  }
}

impl From<&str> for RunnerConfig {
  fn from(s: &str) -> Self {
    RunnerConfig::String(s.to_string())
  }
}

impl From<String> for RunnerConfig {
  fn from(s: String) -> Self {
    RunnerConfig::String(s)
  }
}

/// The Build configuration object.
///
/// See more: <https://v2.tauri.app/reference/config/#buildconfig>
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildConfig {
  /// The binary used to build and run the application.
  pub runner: Option<RunnerConfig>,
  /// The URL to load in development.
  ///
  /// This is usually an URL to a dev server, which serves your application assets with hot-reload and HMR.
  /// Most modern JavaScript bundlers like [Vite](https://vite.dev/guide/) provides a way to start a dev server by default.
  ///
  /// If you don't have a dev server or don't want to use one, ignore this option and use [`frontendDist`](BuildConfig::frontend_dist)
  /// and point to a web assets directory, and Tauri CLI will run its built-in dev server and provide a simple hot-reload experience.
  #[serde(alias = "dev-url")]
  pub dev_url: Option<Url>,
  /// The path to the application assets (usually the `dist` folder of your javascript bundler)
  /// or a URL that could be either a custom protocol registered in the tauri app (for example: `myprotocol://`)
  /// or a remote URL (for example: `https://site.com/app`).
  ///
  /// When a path relative to the configuration file is provided,
  /// it is read recursively and all files are embedded in the application binary.
  /// Tauri then looks for an `index.html` and serves it as the default entry point for your application.
  ///
  /// You can also provide a list of paths to be embedded, which allows granular control over what files are added to the binary.
  /// In this case, all files are added to the root and you must reference it that way in your HTML files.
  ///
  /// When a URL is provided, the application won't have bundled assets
  /// and the application will load that URL by default.
  #[serde(alias = "frontend-dist")]
  pub frontend_dist: Option<FrontendDist>,
  /// A shell command to run before `tauri dev` kicks in.
  ///
  /// The TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-dev-command")]
  pub before_dev_command: Option<BeforeDevCommand>,
  /// A shell command to run before `tauri build` kicks in.
  ///
  /// The TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-build-command")]
  pub before_build_command: Option<HookCommand>,
  /// A shell command to run before the bundling phase in `tauri build` kicks in.
  ///
  /// The TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG environment variables are set if you perform conditional compilation.
  #[serde(alias = "before-bundle-command")]
  pub before_bundle_command: Option<HookCommand>,
  /// Features passed to `cargo` commands.
  pub features: Option<Vec<String>>,
  /// Try to remove unused commands registered from plugins base on the ACL list during `tauri build`,
  /// the way it works is that tauri-cli will read this and set the environment variables for the build script and macros,
  /// and they'll try to get all the allowed commands and remove the rest
  ///
  /// Note:
  ///   - This won't be accounting for dynamically added ACLs when you use features from the `dynamic-acl` (currently enabled by default) feature flag, so make sure to check it when using this
  ///   - This feature requires tauri-plugin 2.1 and tauri 2.4
  #[serde(alias = "remove-unused-commands", default)]
  pub remove_unused_commands: bool,
  /// Additional paths to watch for changes when running `tauri dev`.
  #[serde(alias = "additional-watch-directories", default)]
  pub additional_watch_folders: Vec<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct PackageVersion(String);

impl<'d> serde::Deserialize<'d> for PackageVersion {
  fn deserialize<D: Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
    struct PackageVersionVisitor;

    impl Visitor<'_> for PackageVersionVisitor {
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

fn version_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<PackageVersion>::deserialize(deserializer).map(|v| v.map(|v| v.0))
}

/// The Tauri configuration object.
/// It is read from a file where you can define your frontend assets,
/// configure the bundler and define a tray icon.
///
/// The configuration file is generated by the
/// [`tauri init`](https://v2.tauri.app/reference/cli/#init) command that lives in
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
/// `tauri.windows.conf.json`, `tauri.macos.conf.json`, `tauri.android.conf.json` and `tauri.ios.conf.json`
/// (or `Tauri.linux.toml`, `Tauri.windows.toml`, `Tauri.macos.toml`, `Tauri.android.toml` and `Tauri.ios.toml` if the `Tauri.toml` format is used),
/// which gets merged with the main configuration object.
///
/// ## Configuration Structure
///
/// The configuration is composed of the following objects:
///
/// - [`app`](#appconfig): The Tauri configuration
/// - [`build`](#buildconfig): The build configuration
/// - [`bundle`](#bundleconfig): The bundle configurations
/// - [`plugins`](#pluginconfig): The plugins configuration
///
/// Example tauri.config.json file:
///
/// ```json
/// {
///   "productName": "tauri-app",
///   "version": "0.1.0",
///   "build": {
///     "beforeBuildCommand": "",
///     "beforeDevCommand": "",
///     "devUrl": "http://localhost:3000",
///     "frontendDist": "../dist"
///   },
///   "app": {
///     "security": {
///       "csp": null
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
///   },
///   "bundle": {},
///   "plugins": {}
/// }
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
  /// The JSON schema for the Tauri config.
  #[serde(rename = "$schema")]
  pub schema: Option<String>,
  /// App name.
  #[serde(alias = "product-name")]
  #[cfg_attr(feature = "schema", validate(regex(pattern = "^[^/\\:*?\"<>|]+$")))]
  pub product_name: Option<String>,
  /// Overrides app's main binary filename.
  ///
  /// By default, Tauri uses the output binary from `cargo`, by setting this, we will rename that binary in `tauri-cli`'s
  /// `tauri build` command, and target `tauri bundle` to it
  ///
  /// If possible, change the [`package name`] or set the [`name field`] instead,
  /// and if that's not enough and you're using nightly, consider using the [`different-binary-name`] feature instead
  ///
  /// Note: this config should not include the binary extension (e.g. `.exe`), we'll add that for you
  ///
  /// [`package name`]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
  /// [`name field`]: https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field
  /// [`different-binary-name`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#different-binary-name
  #[serde(alias = "main-binary-name")]
  pub main_binary_name: Option<String>,
  /// App version. It is a semver version number or a path to a `package.json` file containing the `version` field.
  ///
  /// If removed the version number from `Cargo.toml` is used.
  /// It's recommended to manage the app versioning in the Tauri config.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Translates to the bundle's CFBundleShortVersionString property and is used as the default CFBundleVersion.
  ///    You can set an specific bundle version using [`bundle > macOS > bundleVersion`](MacConfig::bundle_version).
  /// - **iOS**: Translates to the bundle's CFBundleShortVersionString property and is used as the default CFBundleVersion.
  ///    You can set an specific bundle version using [`bundle > iOS > bundleVersion`](IosConfig::bundle_version).
  ///    The `tauri ios build` CLI command has a `--build-number <number>` option that lets you append a build number to the app version.
  /// - **Android**: By default version 1.0 is used. You can set a version code using [`bundle > android > versionCode`](AndroidConfig::version_code).
  ///
  /// By default version 1.0 is used on Android.
  #[serde(deserialize_with = "version_deserializer", default)]
  pub version: Option<String>,
  /// The application identifier in reverse domain name notation (e.g. `com.tauri.example`).
  /// This string must be unique across applications since it is used in system configurations like
  /// the bundle ID and path to the webview data directory.
  /// This string must contain only alphanumeric characters (A-Z, a-z, and 0-9), hyphens (-),
  /// and periods (.).
  pub identifier: String,
  /// The App configuration.
  #[serde(default)]
  pub app: AppConfig,
  /// The build configuration.
  #[serde(default)]
  pub build: BuildConfig,
  /// The bundler configuration.
  #[serde(default)]
  pub bundle: BundleConfig,
  /// The plugins config.
  #[serde(default)]
  pub plugins: PluginConfig,
}

/// The plugin configs holds a HashMap mapping a plugin name to its configuration object.
///
/// See more: <https://v2.tauri.app/reference/config/#pluginconfig>
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PluginConfig(pub HashMap<String, JsonValue>);

/// Implement `ToTokens` for all config structs, allowing a literal `Config` to be built.
///
/// This allows for a build script to output the values in a `Config` to a `TokenStream`, which can
/// then be consumed by another crate. Useful for passing a config to both the build script and the
/// application using tauri while only parsing it once (in the build script).
#[cfg(feature = "build")]
mod build {
  use super::*;
  use crate::{literal_struct, tokens::*};
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};
  use std::convert::identity;

  impl ToTokens for WebviewUrl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::WebviewUrl };

      tokens.append_all(match self {
        Self::App(path) => {
          let path = path_buf_lit(path);
          quote! { #prefix::App(#path) }
        }
        Self::External(url) => {
          let url = url_lit(url);
          quote! { #prefix::External(#url) }
        }
        Self::CustomProtocol(url) => {
          let url = url_lit(url);
          quote! { #prefix::CustomProtocol(#url) }
        }
      })
    }
  }

  impl ToTokens for BackgroundThrottlingPolicy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::BackgroundThrottlingPolicy };
      tokens.append_all(match self {
        Self::Disabled => quote! { #prefix::Disabled },
        Self::Throttle => quote! { #prefix::Throttle },
        Self::Suspend => quote! { #prefix::Suspend },
      })
    }
  }

  impl ToTokens for crate::Theme {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::Theme };

      tokens.append_all(match self {
        Self::Light => quote! { #prefix::Light },
        Self::Dark => quote! { #prefix::Dark },
      })
    }
  }

  impl ToTokens for Color {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let Color(r, g, b, a) = self;
      tokens.append_all(quote! {::tauri::utils::config::Color(#r,#g,#b,#a)});
    }
  }
  impl ToTokens for WindowEffectsConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let effects = vec_lit(self.effects.clone(), |d| d);
      let state = opt_lit(self.state.as_ref());
      let radius = opt_lit(self.radius.as_ref());
      let color = opt_lit(self.color.as_ref());

      literal_struct!(
        tokens,
        ::tauri::utils::config::WindowEffectsConfig,
        effects,
        state,
        radius,
        color
      )
    }
  }

  impl ToTokens for crate::TitleBarStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::TitleBarStyle };

      tokens.append_all(match self {
        Self::Visible => quote! { #prefix::Visible },
        Self::Transparent => quote! { #prefix::Transparent },
        Self::Overlay => quote! { #prefix::Overlay },
      })
    }
  }

  impl ToTokens for LogicalPosition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let LogicalPosition { x, y } = self;
      literal_struct!(tokens, ::tauri::utils::config::LogicalPosition, x, y)
    }
  }

  impl ToTokens for crate::WindowEffect {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::WindowEffect };

      #[allow(deprecated)]
      tokens.append_all(match self {
        WindowEffect::AppearanceBased => quote! { #prefix::AppearanceBased},
        WindowEffect::Light => quote! { #prefix::Light},
        WindowEffect::Dark => quote! { #prefix::Dark},
        WindowEffect::MediumLight => quote! { #prefix::MediumLight},
        WindowEffect::UltraDark => quote! { #prefix::UltraDark},
        WindowEffect::Titlebar => quote! { #prefix::Titlebar},
        WindowEffect::Selection => quote! { #prefix::Selection},
        WindowEffect::Menu => quote! { #prefix::Menu},
        WindowEffect::Popover => quote! { #prefix::Popover},
        WindowEffect::Sidebar => quote! { #prefix::Sidebar},
        WindowEffect::HeaderView => quote! { #prefix::HeaderView},
        WindowEffect::Sheet => quote! { #prefix::Sheet},
        WindowEffect::WindowBackground => quote! { #prefix::WindowBackground},
        WindowEffect::HudWindow => quote! { #prefix::HudWindow},
        WindowEffect::FullScreenUI => quote! { #prefix::FullScreenUI},
        WindowEffect::Tooltip => quote! { #prefix::Tooltip},
        WindowEffect::ContentBackground => quote! { #prefix::ContentBackground},
        WindowEffect::UnderWindowBackground => quote! { #prefix::UnderWindowBackground},
        WindowEffect::UnderPageBackground => quote! { #prefix::UnderPageBackground},
        WindowEffect::Mica => quote! { #prefix::Mica},
        WindowEffect::MicaDark => quote! { #prefix::MicaDark},
        WindowEffect::MicaLight => quote! { #prefix::MicaLight},
        WindowEffect::Blur => quote! { #prefix::Blur},
        WindowEffect::Acrylic => quote! { #prefix::Acrylic},
        WindowEffect::Tabbed => quote! { #prefix::Tabbed },
        WindowEffect::TabbedDark => quote! { #prefix::TabbedDark },
        WindowEffect::TabbedLight => quote! { #prefix::TabbedLight },
      })
    }
  }

  impl ToTokens for crate::WindowEffectState {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::WindowEffectState };

      #[allow(deprecated)]
      tokens.append_all(match self {
        WindowEffectState::Active => quote! { #prefix::Active},
        WindowEffectState::FollowsWindowActiveState => quote! { #prefix::FollowsWindowActiveState},
        WindowEffectState::Inactive => quote! { #prefix::Inactive},
      })
    }
  }

  impl ToTokens for PreventOverflowMargin {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let width = self.width;
      let height = self.height;

      literal_struct!(
        tokens,
        ::tauri::utils::config::PreventOverflowMargin,
        width,
        height
      )
    }
  }

  impl ToTokens for PreventOverflowConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::PreventOverflowConfig };

      #[allow(deprecated)]
      tokens.append_all(match self {
        Self::Enable(enable) => quote! { #prefix::Enable(#enable) },
        Self::Margin(margin) => quote! { #prefix::Margin(#margin) },
      })
    }
  }

  impl ToTokens for ScrollBarStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::ScrollBarStyle };

      tokens.append_all(match self {
        Self::Default => quote! { #prefix::Default },
        Self::FluentOverlay => quote! { #prefix::FluentOverlay },
      })
    }
  }

  impl ToTokens for WindowConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let label = str_lit(&self.label);
      let create = &self.create;
      let url = &self.url;
      let user_agent = opt_str_lit(self.user_agent.as_ref());
      let drag_drop_enabled = self.drag_drop_enabled;
      let center = self.center;
      let x = opt_lit(self.x.as_ref());
      let y = opt_lit(self.y.as_ref());
      let width = self.width;
      let height = self.height;
      let min_width = opt_lit(self.min_width.as_ref());
      let min_height = opt_lit(self.min_height.as_ref());
      let max_width = opt_lit(self.max_width.as_ref());
      let max_height = opt_lit(self.max_height.as_ref());
      let prevent_overflow = opt_lit(self.prevent_overflow.as_ref());
      let resizable = self.resizable;
      let maximizable = self.maximizable;
      let minimizable = self.minimizable;
      let closable = self.closable;
      let title = str_lit(&self.title);
      let proxy_url = opt_lit(self.proxy_url.as_ref().map(url_lit).as_ref());
      let fullscreen = self.fullscreen;
      let focus = self.focus;
      let focusable = self.focusable;
      let transparent = self.transparent;
      let maximized = self.maximized;
      let visible = self.visible;
      let decorations = self.decorations;
      let always_on_bottom = self.always_on_bottom;
      let always_on_top = self.always_on_top;
      let visible_on_all_workspaces = self.visible_on_all_workspaces;
      let content_protected = self.content_protected;
      let skip_taskbar = self.skip_taskbar;
      let window_classname = opt_str_lit(self.window_classname.as_ref());
      let theme = opt_lit(self.theme.as_ref());
      let title_bar_style = &self.title_bar_style;
      let traffic_light_position = opt_lit(self.traffic_light_position.as_ref());
      let hidden_title = self.hidden_title;
      let accept_first_mouse = self.accept_first_mouse;
      let tabbing_identifier = opt_str_lit(self.tabbing_identifier.as_ref());
      let additional_browser_args = opt_str_lit(self.additional_browser_args.as_ref());
      let shadow = self.shadow;
      let window_effects = opt_lit(self.window_effects.as_ref());
      let incognito = self.incognito;
      let parent = opt_str_lit(self.parent.as_ref());
      let zoom_hotkeys_enabled = self.zoom_hotkeys_enabled;
      let browser_extensions_enabled = self.browser_extensions_enabled;
      let use_https_scheme = self.use_https_scheme;
      let devtools = opt_lit(self.devtools.as_ref());
      let background_color = opt_lit(self.background_color.as_ref());
      let background_throttling = opt_lit(self.background_throttling.as_ref());
      let javascript_disabled = self.javascript_disabled;
      let allow_link_preview = self.allow_link_preview;
      let disable_input_accessory_view = self.disable_input_accessory_view;
      let data_directory = opt_lit(self.data_directory.as_ref().map(path_buf_lit).as_ref());
      let data_store_identifier = opt_vec_lit(self.data_store_identifier, identity);
      let scroll_bar_style = &self.scroll_bar_style;

      literal_struct!(
        tokens,
        ::tauri::utils::config::WindowConfig,
        label,
        url,
        create,
        user_agent,
        drag_drop_enabled,
        center,
        x,
        y,
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        prevent_overflow,
        resizable,
        maximizable,
        minimizable,
        closable,
        title,
        proxy_url,
        fullscreen,
        focus,
        focusable,
        transparent,
        maximized,
        visible,
        decorations,
        always_on_bottom,
        always_on_top,
        visible_on_all_workspaces,
        content_protected,
        skip_taskbar,
        window_classname,
        theme,
        title_bar_style,
        traffic_light_position,
        hidden_title,
        accept_first_mouse,
        tabbing_identifier,
        additional_browser_args,
        shadow,
        window_effects,
        incognito,
        parent,
        zoom_hotkeys_enabled,
        browser_extensions_enabled,
        use_https_scheme,
        devtools,
        background_color,
        background_throttling,
        javascript_disabled,
        allow_link_preview,
        disable_input_accessory_view,
        data_directory,
        data_store_identifier,
        scroll_bar_style
      );
    }
  }

  impl ToTokens for PatternKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::PatternKind };

      tokens.append_all(match self {
        Self::Brownfield => quote! { #prefix::Brownfield },
        #[cfg(not(feature = "isolation"))]
        Self::Isolation { dir: _ } => quote! { #prefix::Brownfield },
        #[cfg(feature = "isolation")]
        Self::Isolation { dir } => {
          let dir = path_buf_lit(dir);
          quote! { #prefix::Isolation { dir: #dir } }
        }
      })
    }
  }

  impl ToTokens for WebviewInstallMode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::WebviewInstallMode };

      tokens.append_all(match self {
        Self::Skip => quote! { #prefix::Skip },
        Self::DownloadBootstrapper { silent } => {
          quote! { #prefix::DownloadBootstrapper { silent: #silent } }
        }
        Self::EmbedBootstrapper { silent } => {
          quote! { #prefix::EmbedBootstrapper { silent: #silent } }
        }
        Self::OfflineInstaller { silent } => {
          quote! { #prefix::OfflineInstaller { silent: #silent } }
        }
        Self::FixedRuntime { path } => {
          let path = path_buf_lit(path);
          quote! { #prefix::FixedRuntime { path: #path } }
        }
      })
    }
  }

  impl ToTokens for WindowsConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let webview_install_mode = &self.webview_install_mode;
      tokens.append_all(quote! { ::tauri::utils::config::WindowsConfig {
        webview_install_mode: #webview_install_mode,
        ..Default::default()
      }})
    }
  }

  impl ToTokens for BundleConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let publisher = quote!(None);
      let homepage = quote!(None);
      let icon = vec_lit(&self.icon, str_lit);
      let active = self.active;
      let targets = quote!(Default::default());
      let create_updater_artifacts = quote!(Default::default());
      let resources = quote!(None);
      let copyright = quote!(None);
      let category = quote!(None);
      let file_associations = quote!(None);
      let short_description = quote!(None);
      let long_description = quote!(None);
      let use_local_tools_dir = self.use_local_tools_dir;
      let external_bin = opt_vec_lit(self.external_bin.as_ref(), str_lit);
      let windows = &self.windows;
      let license = opt_str_lit(self.license.as_ref());
      let license_file = opt_lit(self.license_file.as_ref().map(path_buf_lit).as_ref());
      let linux = quote!(Default::default());
      let macos = quote!(Default::default());
      let ios = quote!(Default::default());
      let android = quote!(Default::default());

      literal_struct!(
        tokens,
        ::tauri::utils::config::BundleConfig,
        active,
        publisher,
        homepage,
        icon,
        targets,
        create_updater_artifacts,
        resources,
        copyright,
        category,
        license,
        license_file,
        file_associations,
        short_description,
        long_description,
        use_local_tools_dir,
        external_bin,
        windows,
        linux,
        macos,
        ios,
        android
      );
    }
  }

  impl ToTokens for FrontendDist {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::FrontendDist };

      tokens.append_all(match self {
        Self::Url(url) => {
          let url = url_lit(url);
          quote! { #prefix::Url(#url) }
        }
        Self::Directory(path) => {
          let path = path_buf_lit(path);
          quote! { #prefix::Directory(#path) }
        }
        Self::Files(files) => {
          let files = vec_lit(files, path_buf_lit);
          quote! { #prefix::Files(#files) }
        }
      })
    }
  }

  impl ToTokens for RunnerConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::RunnerConfig };

      tokens.append_all(match self {
        Self::String(cmd) => {
          let cmd = cmd.as_str();
          quote!(#prefix::String(#cmd.into()))
        }
        Self::Object { cmd, cwd, args } => {
          let cmd = cmd.as_str();
          let cwd = opt_str_lit(cwd.as_ref());
          let args = opt_lit(args.as_ref().map(|v| vec_lit(v, str_lit)).as_ref());
          quote!(#prefix::Object {
            cmd: #cmd.into(),
            cwd: #cwd,
            args: #args,
          })
        }
      })
    }
  }

  impl ToTokens for BuildConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let dev_url = opt_lit(self.dev_url.as_ref().map(url_lit).as_ref());
      let frontend_dist = opt_lit(self.frontend_dist.as_ref());
      let runner = opt_lit(self.runner.as_ref());
      let before_dev_command = quote!(None);
      let before_build_command = quote!(None);
      let before_bundle_command = quote!(None);
      let features = quote!(None);
      let remove_unused_commands = quote!(false);
      let additional_watch_folders = quote!(Vec::new());

      literal_struct!(
        tokens,
        ::tauri::utils::config::BuildConfig,
        runner,
        dev_url,
        frontend_dist,
        before_dev_command,
        before_build_command,
        before_bundle_command,
        features,
        remove_unused_commands,
        additional_watch_folders
      );
    }
  }

  impl ToTokens for CspDirectiveSources {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::CspDirectiveSources };

      tokens.append_all(match self {
        Self::Inline(sources) => {
          let sources = sources.as_str();
          quote!(#prefix::Inline(#sources.into()))
        }
        Self::List(list) => {
          let list = vec_lit(list, str_lit);
          quote!(#prefix::List(#list))
        }
      })
    }
  }

  impl ToTokens for Csp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::Csp };

      tokens.append_all(match self {
        Self::Policy(policy) => {
          let policy = policy.as_str();
          quote!(#prefix::Policy(#policy.into()))
        }
        Self::DirectiveMap(list) => {
          let map = map_lit(
            quote! { ::std::collections::HashMap },
            list,
            str_lit,
            identity,
          );
          quote!(#prefix::DirectiveMap(#map))
        }
      })
    }
  }

  impl ToTokens for DisabledCspModificationKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::DisabledCspModificationKind };

      tokens.append_all(match self {
        Self::Flag(flag) => {
          quote! { #prefix::Flag(#flag) }
        }
        Self::List(directives) => {
          let directives = vec_lit(directives, str_lit);
          quote! { #prefix::List(#directives) }
        }
      });
    }
  }

  impl ToTokens for CapabilityEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::CapabilityEntry };

      tokens.append_all(match self {
        Self::Inlined(capability) => {
          quote! { #prefix::Inlined(#capability) }
        }
        Self::Reference(id) => {
          let id = str_lit(id);
          quote! { #prefix::Reference(#id) }
        }
      });
    }
  }

  impl ToTokens for HeaderSource {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::HeaderSource };

      tokens.append_all(match self {
        Self::Inline(s) => {
          let line = s.as_str();
          quote!(#prefix::Inline(#line.into()))
        }
        Self::List(l) => {
          let list = vec_lit(l, str_lit);
          quote!(#prefix::List(#list))
        }
        Self::Map(m) => {
          let map = map_lit(quote! { ::std::collections::HashMap }, m, str_lit, str_lit);
          quote!(#prefix::Map(#map))
        }
      })
    }
  }

  impl ToTokens for HeaderConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let access_control_allow_credentials =
        opt_lit(self.access_control_allow_credentials.as_ref());
      let access_control_allow_headers = opt_lit(self.access_control_allow_headers.as_ref());
      let access_control_allow_methods = opt_lit(self.access_control_allow_methods.as_ref());
      let access_control_expose_headers = opt_lit(self.access_control_expose_headers.as_ref());
      let access_control_max_age = opt_lit(self.access_control_max_age.as_ref());
      let cross_origin_embedder_policy = opt_lit(self.cross_origin_embedder_policy.as_ref());
      let cross_origin_opener_policy = opt_lit(self.cross_origin_opener_policy.as_ref());
      let cross_origin_resource_policy = opt_lit(self.cross_origin_resource_policy.as_ref());
      let permissions_policy = opt_lit(self.permissions_policy.as_ref());
      let service_worker_allowed = opt_lit(self.service_worker_allowed.as_ref());
      let timing_allow_origin = opt_lit(self.timing_allow_origin.as_ref());
      let x_content_type_options = opt_lit(self.x_content_type_options.as_ref());
      let tauri_custom_header = opt_lit(self.tauri_custom_header.as_ref());

      literal_struct!(
        tokens,
        ::tauri::utils::config::HeaderConfig,
        access_control_allow_credentials,
        access_control_allow_headers,
        access_control_allow_methods,
        access_control_expose_headers,
        access_control_max_age,
        cross_origin_embedder_policy,
        cross_origin_opener_policy,
        cross_origin_resource_policy,
        permissions_policy,
        service_worker_allowed,
        timing_allow_origin,
        x_content_type_options,
        tauri_custom_header
      );
    }
  }

  impl ToTokens for SecurityConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let csp = opt_lit(self.csp.as_ref());
      let dev_csp = opt_lit(self.dev_csp.as_ref());
      let freeze_prototype = self.freeze_prototype;
      let dangerous_disable_asset_csp_modification = &self.dangerous_disable_asset_csp_modification;
      let asset_protocol = &self.asset_protocol;
      let pattern = &self.pattern;
      let capabilities = vec_lit(&self.capabilities, identity);
      let headers = opt_lit(self.headers.as_ref());

      literal_struct!(
        tokens,
        ::tauri::utils::config::SecurityConfig,
        csp,
        dev_csp,
        freeze_prototype,
        dangerous_disable_asset_csp_modification,
        asset_protocol,
        pattern,
        capabilities,
        headers
      );
    }
  }

  impl ToTokens for TrayIconConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      // For [`Self::menu_on_left_click`]
      tokens.append_all(quote!(#[allow(deprecated)]));

      let id = opt_str_lit(self.id.as_ref());
      let icon_as_template = self.icon_as_template;
      #[allow(deprecated)]
      let menu_on_left_click = self.menu_on_left_click;
      let show_menu_on_left_click = self.show_menu_on_left_click;
      let icon_path = path_buf_lit(&self.icon_path);
      let title = opt_str_lit(self.title.as_ref());
      let tooltip = opt_str_lit(self.tooltip.as_ref());
      literal_struct!(
        tokens,
        ::tauri::utils::config::TrayIconConfig,
        id,
        icon_path,
        icon_as_template,
        menu_on_left_click,
        show_menu_on_left_click,
        title,
        tooltip
      );
    }
  }

  impl ToTokens for FsScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::FsScope };

      tokens.append_all(match self {
        Self::AllowedPaths(allow) => {
          let allowed_paths = vec_lit(allow, path_buf_lit);
          quote! { #prefix::AllowedPaths(#allowed_paths) }
        }
        Self::Scope { allow, deny , require_literal_leading_dot} => {
          let allow = vec_lit(allow, path_buf_lit);
          let deny = vec_lit(deny, path_buf_lit);
          let  require_literal_leading_dot = opt_lit(require_literal_leading_dot.as_ref());
          quote! { #prefix::Scope { allow: #allow, deny: #deny, require_literal_leading_dot: #require_literal_leading_dot } }
        }
      });
    }
  }

  impl ToTokens for AssetProtocolConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let scope = &self.scope;
      tokens.append_all(quote! { ::tauri::utils::config::AssetProtocolConfig { scope: #scope, ..Default::default() } })
    }
  }

  impl ToTokens for AppConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let windows = vec_lit(&self.windows, identity);
      let security = &self.security;
      let tray_icon = opt_lit(self.tray_icon.as_ref());
      let macos_private_api = self.macos_private_api;
      let with_global_tauri = self.with_global_tauri;
      let enable_gtk_app_id = self.enable_gtk_app_id;

      literal_struct!(
        tokens,
        ::tauri::utils::config::AppConfig,
        windows,
        security,
        tray_icon,
        macos_private_api,
        with_global_tauri,
        enable_gtk_app_id
      );
    }
  }

  impl ToTokens for PluginConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let config = map_lit(
        quote! { ::std::collections::HashMap },
        &self.0,
        str_lit,
        json_value_lit,
      );
      tokens.append_all(quote! { ::tauri::utils::config::PluginConfig(#config) })
    }
  }

  impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let schema = quote!(None);
      let product_name = opt_str_lit(self.product_name.as_ref());
      let main_binary_name = opt_str_lit(self.main_binary_name.as_ref());
      let version = opt_str_lit(self.version.as_ref());
      let identifier = str_lit(&self.identifier);
      let app = &self.app;
      let build = &self.build;
      let bundle = &self.bundle;
      let plugins = &self.plugins;

      literal_struct!(
        tokens,
        ::tauri::utils::config::Config,
        schema,
        product_name,
        main_binary_name,
        version,
        identifier,
        app,
        build,
        bundle,
        plugins
      );
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  // TODO: create a test that compares a config to a json config

  #[test]
  // test all of the default functions
  fn test_defaults() {
    // get default app config
    let a_config = AppConfig::default();
    // get default build config
    let b_config = BuildConfig::default();
    // get default window
    let d_windows: Vec<WindowConfig> = vec![];
    // get default bundle
    let d_bundle = BundleConfig::default();

    // create a tauri config.
    let app = AppConfig {
      windows: vec![],
      security: SecurityConfig {
        csp: None,
        dev_csp: None,
        freeze_prototype: false,
        dangerous_disable_asset_csp_modification: DisabledCspModificationKind::Flag(false),
        asset_protocol: AssetProtocolConfig::default(),
        pattern: Default::default(),
        capabilities: Vec::new(),
        headers: None,
      },
      tray_icon: None,
      macos_private_api: false,
      with_global_tauri: false,
      enable_gtk_app_id: false,
    };

    // create a build config
    let build = BuildConfig {
      runner: None,
      dev_url: None,
      frontend_dist: None,
      before_dev_command: None,
      before_build_command: None,
      before_bundle_command: None,
      features: None,
      remove_unused_commands: false,
      additional_watch_folders: Vec::new(),
    };

    // create a bundle config
    let bundle = BundleConfig {
      active: false,
      targets: Default::default(),
      create_updater_artifacts: Default::default(),
      publisher: None,
      homepage: None,
      icon: Vec::new(),
      resources: None,
      copyright: None,
      category: None,
      file_associations: None,
      short_description: None,
      long_description: None,
      use_local_tools_dir: false,
      license: None,
      license_file: None,
      linux: Default::default(),
      macos: Default::default(),
      external_bin: None,
      windows: Default::default(),
      ios: Default::default(),
      android: Default::default(),
    };

    // test the configs
    assert_eq!(a_config, app);
    assert_eq!(b_config, build);
    assert_eq!(d_bundle, bundle);
    assert_eq!(d_windows, app.windows);
  }

  #[test]
  fn parse_hex_color() {
    use super::Color;

    assert_eq!(Color(255, 255, 255, 255), "fff".parse().unwrap());
    assert_eq!(Color(255, 255, 255, 255), "#fff".parse().unwrap());
    assert_eq!(Color(0, 0, 0, 255), "#000000".parse().unwrap());
    assert_eq!(Color(0, 0, 0, 255), "#000000ff".parse().unwrap());
    assert_eq!(Color(0, 255, 0, 255), "#00ff00ff".parse().unwrap());
  }

  #[test]
  fn test_runner_config_string_format() {
    use super::RunnerConfig;

    // Test string format deserialization
    let json = r#""cargo""#;
    let runner: RunnerConfig = serde_json::from_str(json).unwrap();

    assert_eq!(runner.cmd(), "cargo");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);

    // Test string format serialization
    let serialized = serde_json::to_string(&runner).unwrap();
    assert_eq!(serialized, r#""cargo""#);
  }

  #[test]
  fn test_runner_config_object_format_full() {
    use super::RunnerConfig;

    // Test object format with all fields
    let json = r#"{"cmd": "my_runner", "cwd": "/tmp/build", "args": ["--quiet", "--verbose"]}"#;
    let runner: RunnerConfig = serde_json::from_str(json).unwrap();

    assert_eq!(runner.cmd(), "my_runner");
    assert_eq!(runner.cwd(), Some("/tmp/build"));
    assert_eq!(
      runner.args(),
      Some(&["--quiet".to_string(), "--verbose".to_string()][..])
    );

    // Test object format serialization
    let serialized = serde_json::to_string(&runner).unwrap();
    let deserialized: RunnerConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(runner, deserialized);
  }

  #[test]
  fn test_runner_config_object_format_minimal() {
    use super::RunnerConfig;

    // Test object format with only cmd field
    let json = r#"{"cmd": "cross"}"#;
    let runner: RunnerConfig = serde_json::from_str(json).unwrap();

    assert_eq!(runner.cmd(), "cross");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);
  }

  #[test]
  fn test_runner_config_default() {
    use super::RunnerConfig;

    let default_runner = RunnerConfig::default();
    assert_eq!(default_runner.cmd(), "cargo");
    assert_eq!(default_runner.cwd(), None);
    assert_eq!(default_runner.args(), None);
  }

  #[test]
  fn test_runner_config_from_str() {
    use super::RunnerConfig;

    // Test From<&str> trait
    let runner: RunnerConfig = "my_runner".into();
    assert_eq!(runner.cmd(), "my_runner");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);
  }

  #[test]
  fn test_runner_config_from_string() {
    use super::RunnerConfig;

    // Test From<String> trait
    let runner: RunnerConfig = "another_runner".to_string().into();
    assert_eq!(runner.cmd(), "another_runner");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);
  }

  #[test]
  fn test_runner_config_from_str_parse() {
    use super::RunnerConfig;
    use std::str::FromStr;

    // Test FromStr trait
    let runner = RunnerConfig::from_str("parsed_runner").unwrap();
    assert_eq!(runner.cmd(), "parsed_runner");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);
  }

  #[test]
  fn test_runner_config_in_build_config() {
    use super::BuildConfig;

    // Test string format in BuildConfig
    let json = r#"{"runner": "cargo"}"#;
    let build_config: BuildConfig = serde_json::from_str(json).unwrap();

    let runner = build_config.runner.unwrap();
    assert_eq!(runner.cmd(), "cargo");
    assert_eq!(runner.cwd(), None);
    assert_eq!(runner.args(), None);
  }

  #[test]
  fn test_runner_config_in_build_config_object() {
    use super::BuildConfig;

    // Test object format in BuildConfig
    let json = r#"{"runner": {"cmd": "cross", "cwd": "/workspace", "args": ["--target", "x86_64-unknown-linux-gnu"]}}"#;
    let build_config: BuildConfig = serde_json::from_str(json).unwrap();

    let runner = build_config.runner.unwrap();
    assert_eq!(runner.cmd(), "cross");
    assert_eq!(runner.cwd(), Some("/workspace"));
    assert_eq!(
      runner.args(),
      Some(
        &[
          "--target".to_string(),
          "x86_64-unknown-linux-gnu".to_string()
        ][..]
      )
    );
  }

  #[test]
  fn test_runner_config_in_full_config() {
    use super::Config;

    // Test runner config in full Tauri config
    let json = r#"{
      "productName": "Test App",
      "version": "1.0.0",
      "identifier": "com.test.app",
      "build": {
        "runner": {
          "cmd": "my_custom_cargo",
          "cwd": "/tmp/build",
          "args": ["--quiet", "--verbose"]
        }
      }
    }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    let runner = config.build.runner.unwrap();

    assert_eq!(runner.cmd(), "my_custom_cargo");
    assert_eq!(runner.cwd(), Some("/tmp/build"));
    assert_eq!(
      runner.args(),
      Some(&["--quiet".to_string(), "--verbose".to_string()][..])
    );
  }

  #[test]
  fn test_runner_config_equality() {
    use super::RunnerConfig;

    let runner1 = RunnerConfig::String("cargo".to_string());
    let runner2 = RunnerConfig::String("cargo".to_string());
    let runner3 = RunnerConfig::String("cross".to_string());

    assert_eq!(runner1, runner2);
    assert_ne!(runner1, runner3);

    let runner4 = RunnerConfig::Object {
      cmd: "cargo".to_string(),
      cwd: Some("/tmp".to_string()),
      args: Some(vec!["--quiet".to_string()]),
    };
    let runner5 = RunnerConfig::Object {
      cmd: "cargo".to_string(),
      cwd: Some("/tmp".to_string()),
      args: Some(vec!["--quiet".to_string()]),
    };

    assert_eq!(runner4, runner5);
    assert_ne!(runner1, runner4);
  }

  #[test]
  fn test_runner_config_untagged_serialization() {
    use super::RunnerConfig;

    // Test that serde untagged works correctly - string should serialize as string, not object
    let string_runner = RunnerConfig::String("cargo".to_string());
    let string_json = serde_json::to_string(&string_runner).unwrap();
    assert_eq!(string_json, r#""cargo""#);

    // Test that object serializes as object
    let object_runner = RunnerConfig::Object {
      cmd: "cross".to_string(),
      cwd: None,
      args: None,
    };
    let object_json = serde_json::to_string(&object_runner).unwrap();
    assert!(object_json.contains("\"cmd\":\"cross\""));
    // With skip_serializing_none, null values should not be included
    assert!(object_json.contains("\"cwd\":null") || !object_json.contains("cwd"));
    assert!(object_json.contains("\"args\":null") || !object_json.contains("args"));
  }
}
