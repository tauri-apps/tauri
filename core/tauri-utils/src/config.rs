// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri configuration used at runtime.
//!
//! It is pulled from a `tauri.conf.json` file and the [`Config`] struct is generated at compile time.
//!
//! # Stability
//! This is a core functionality that is not considered part of the stable API.
//! If you use it, note that it may include breaking changes in the future.

#[cfg(target_os = "linux")]
use heck::ToKebabCase;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
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

pub use self::parse::parse;

/// An URL to open on a Tauri webview window.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
      Self::External(url) => write!(f, "{}", url),
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "lowercase"))]
pub enum BundleType {
  /// The debian bundle (.deb).
  Deb,
  /// The AppImage bundle (.appimage).
  AppImage,
  /// The Microsoft Installer bundle (.msi).
  Msi,
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
      "app" => Ok(Self::App),
      "dmg" => Ok(Self::Dmg),
      "updater" => Ok(Self::Updater),
      _ => Err(DeError::custom(format!("unknown bundle target '{}'", s))),
    }
  }
}

/// Targets to bundle. Each value is case insensitive.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BundleTarget {
  /// Bundle all targets.
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
        enum_values: Some(vec!["all".into()]),
        metadata: Some(Box::new(schemars::schema::Metadata {
          description: Some("Bundle all targets.".to_owned()),
          ..Default::default()
        })),
        ..Default::default()
      }
      .into(),
      schemars::_private::apply_metadata(
        gen.subschema_for::<Vec<BundleType>>(),
        schemars::schema::Metadata {
          description: Some("A list of bundle targets.".to_owned()),
          ..Default::default()
        },
      ),
      schemars::_private::apply_metadata(
        gen.subschema_for::<BundleType>(),
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

impl Default for BundleTarget {
  fn default() -> Self {
    Self::All
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
      BundleTargetInner::All(t) => Err(DeError::custom(format!("invalid bundle type {}", t))),
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
      Self::All => vec![],
      Self::List(list) => list.clone(),
      Self::One(i) => vec![i.clone()],
    }
  }
}

/// Configuration for AppImage bundles.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppImageConfig {
  /// Include additional gstreamer dependencies needed for audio and video playback.
  /// This increases the bundle size by ~15-35MB depending on your build system.
  #[serde(default)]
  pub bundle_media_framework: bool,
}

/// Configuration for Debian (.deb) bundles.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebConfig {
  /// The list of deb dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// The files to include on the package.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
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
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
    default = "minimum_system_version"
  )]
  pub minimum_system_version: Option<String>,
  /// Allows your application to communicate with the outside world.
  /// It should be a lowercase, without port and protocol domain name.
  pub exception_domain: Option<String>,
  /// The path to the license file to add to the DMG bundle.
  pub license: Option<String>,
  /// Identity to use for code signing.
  pub signing_identity: Option<String>,
  /// Provider short name for notarization.
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
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixLanguageConfig {
  /// The path to a locale (`.wxl`) file. See <https://wixtoolset.org/documentation/manual/v3/howtos/ui_and_localization/build_a_localized_version.html>.
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixConfig {
  /// The installer languages to build. See <https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables>.
  #[serde(default)]
  pub language: WixLanguage,
  /// A custom .wxs template to use.
  pub template: Option<PathBuf>,
  /// A list of paths to .wxs files with WiX fragments to use.
  #[serde(default)]
  pub fragment_paths: Vec<PathBuf>,
  /// The ComponentGroup element ids you want to reference from the fragments.
  #[serde(default)]
  pub component_group_refs: Vec<String>,
  /// The Component element ids you want to reference from the fragments.
  #[serde(default)]
  pub component_refs: Vec<String>,
  /// The FeatureGroup element ids you want to reference from the fragments.
  #[serde(default)]
  pub feature_group_refs: Vec<String>,
  /// The Feature element ids you want to reference from the fragments.
  #[serde(default)]
  pub feature_refs: Vec<String>,
  /// The Merge element ids you want to reference from the fragments.
  #[serde(default)]
  pub merge_refs: Vec<String>,
  /// Disables the Webview2 runtime installation after app install.
  ///
  /// Will be removed in v2, prefer the [`WindowsConfig::webview_install_mode`] option.
  #[serde(default)]
  pub skip_webview_install: bool,
  /// The path to the license file to render on the installer.
  ///
  /// Must be an RTF file, so if a different extension is provided, we convert it to the RTF format.
  pub license: Option<PathBuf>,
  /// Create an elevated update task within Windows Task Scheduler.
  #[serde(default)]
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

/// Install modes for the Webview2 runtime.
/// Note that for the updater bundle [`Self::DownloadBootstrapper`] is used.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum WebviewInstallMode {
  /// Do not install the Webview2 as part of the Windows Installer.
  Skip,
  /// Download the bootstrapper and run it.
  /// Requires internet connection.
  /// Results in a smaller installer size, but is not recommended on Windows 7.
  DownloadBootstrapper {
    /// Instructs the installer to run the bootstrapper in silent mode. Defaults to `true`.
    #[serde(default = "default_webview_install_silent")]
    silent: bool,
  },
  /// Embed the bootstrapper and run it.
  /// Requires internet connection.
  /// Increases the installer size by around 1.8MB, but offers better support on Windows 7.
  EmbedBootstrapper {
    /// Instructs the installer to run the bootstrapper in silent mode. Defaults to `true`.
    #[serde(default = "default_webview_install_silent")]
    silent: bool,
  },
  /// Embed the offline installer and run it.
  /// Does not require internet connection.
  /// Increases the installer size by around 127MB.
  OfflineInstaller {
    /// Instructs the installer to run the installer in silent mode. Defaults to `true`.
    #[serde(default = "default_webview_install_silent")]
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

fn default_webview_install_silent() -> bool {
  true
}

impl Default for WebviewInstallMode {
  fn default() -> Self {
    Self::DownloadBootstrapper {
      silent: default_webview_install_silent(),
    }
  }
}

/// Windows bundler configuration.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowsConfig {
  /// Specifies the file digest algorithm to use for creating file signatures.
  /// Required for code signing. SHA-256 is recommended.
  pub digest_algorithm: Option<String>,
  /// Specifies the SHA1 hash of the signing certificate.
  pub certificate_thumbprint: Option<String>,
  /// Server to use during timestamping.
  pub timestamp_url: Option<String>,
  /// Whether to use Time-Stamp Protocol (TSP, a.k.a. RFC 3161) for the timestamp server. Your code signing provider may
  /// use a TSP timestamp server, like e.g. SSL.com does. If so, enable TSP by setting to true.
  #[serde(default)]
  pub tsp: bool,
  /// The installation mode for the Webview2 runtime.
  #[serde(default)]
  pub webview_install_mode: WebviewInstallMode,
  /// Path to the webview fixed runtime to use. Overwrites [`Self::webview_install_mode`] if set.
  ///
  /// Will be removed in v2, prefer the [`Self::webview_install_mode`] option.
  ///
  /// The fixed version can be downloaded [on the official website](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).
  /// The `.cab` file must be extracted to a folder and this folder path must be defined on this field.
  pub webview_fixed_runtime_path: Option<PathBuf>,
  /// Validates a second app installation, blocking the user from installing an older version if set to `false`.
  ///
  /// For instance, if `1.2.1` is installed, the user won't be able to install app version `1.2.0` or `1.1.5`.
  ///
  /// The default value of this flag is `true`.
  #[serde(default = "default_allow_downgrades")]
  pub allow_downgrades: bool,
  /// Configuration for the MSI generated with WiX.
  pub wix: Option<WixConfig>,
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
      allow_downgrades: default_allow_downgrades(),
      wix: None,
    }
  }
}

fn default_allow_downgrades() -> bool {
  true
}

/// Configuration for tauri-bundler.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BundleConfig {
  /// Whether Tauri should bundle your application or just output the executable.
  #[serde(default)]
  pub active: bool,
  /// The bundle targets, currently supports ["deb", "appimage", "msi", "app", "dmg", "updater"] or "all".
  #[serde(default)]
  pub targets: BundleTarget,
  /// The application identifier in reverse domain name notation (e.g. `com.tauri.example`).
  /// This string must be unique across applications since it is used in system configurations like
  /// the bundle ID and path to the webview data directory.
  /// This string must contain only alphanumeric characters (A–Z, a–z, and 0–9), hyphens (-),
  /// and periods (.).
  pub identifier: String,
  /// The app's icons
  #[serde(default)]
  pub icon: Vec<String>,
  /// App resources to bundle.
  /// Each resource is a path to a file or directory.
  /// Glob patterns are supported.
  pub resources: Option<Vec<String>>,
  /// A copyright string associated with your application.
  pub copyright: Option<String>,
  /// The application kind.
  ///
  /// Should be one of the following:
  /// Business, DeveloperTool, Education, Entertainment, Finance, Game, ActionGame, AdventureGame, ArcadeGame, BoardGame, CardGame, CasinoGame, DiceGame, EducationalGame, FamilyGame, KidsGame, MusicGame, PuzzleGame, RacingGame, RolePlayingGame, SimulationGame, SportsGame, StrategyGame, TriviaGame, WordGame, GraphicsAndDesign, HealthcareAndFitness, Lifestyle, Medical, Music, News, Photography, Productivity, Reference, SocialNetworking, Sports, Travel, Utility, Video, Weather.
  pub category: Option<String>,
  /// A short description of your application.
  pub short_description: Option<String>,
  /// A longer, multi-line description of the application.
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
  pub external_bin: Option<Vec<String>>,
  /// Configuration for the Windows bundle.
  #[serde(default)]
  pub windows: WindowsConfig,
}

/// A CLI argument definition.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  pub long_description: Option<String>,
  /// Specifies that the argument takes a value at run time.
  ///
  /// NOTE: values for arguments may be specified in any of the following methods
  /// - Using a space such as -o value or --option value
  /// - Using an equals and no space such as -o=value or --option=value
  /// - Use a short and no space such as -ovalue
  #[serde(default)]
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
  #[serde(default)]
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
  pub number_of_values: Option<usize>,
  /// Specifies a list of possible values for this argument.
  /// At runtime, the CLI verifies that only one of the specified values was used, or fails with an error message.
  pub possible_values: Option<Vec<String>>,
  /// Specifies the minimum number of values for this argument.
  /// For example, if you had a -f `<file>` argument where you wanted at least 2 'files',
  /// you would set `minValues: 2`, and this argument would be satisfied if the user provided, 2 or more values.
  pub min_values: Option<usize>,
  /// Specifies the maximum number of values are for this argument.
  /// For example, if you had a -f `<file>` argument where you wanted up to 3 'files',
  /// you would set .max_values(3), and this argument would be satisfied if the user provided, 1, 2, or 3 values.
  pub max_values: Option<usize>,
  /// Sets whether or not the argument is required by default.
  ///
  /// - Required by default means it is required, when no other conflicting rules have been evaluated
  /// - Conflicting rules take precedence over being required.
  #[serde(default)]
  pub required: bool,
  /// Sets an arg that override this arg's required setting
  /// i.e. this arg will be required unless this other argument is present.
  pub required_unless_present: Option<String>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless all these other arguments are present.
  pub required_unless_present_all: Option<Vec<String>>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless at least one of these other arguments are present.
  pub required_unless_present_any: Option<Vec<String>>,
  /// Sets a conflicting argument by name
  /// i.e. when using this argument, the following argument can't be present and vice versa.
  pub conflicts_with: Option<String>,
  /// The same as conflictsWith but allows specifying multiple two-way conflicts per argument.
  pub conflicts_with_all: Option<Vec<String>>,
  /// Tets an argument by name that is required when this one is present
  /// i.e. when using this argument, the following argument must be present.
  pub requires: Option<String>,
  /// Sts multiple arguments by names that are required when this one is present
  /// i.e. when using this argument, the following arguments must be present.
  pub requires_all: Option<Vec<String>>,
  /// Allows a conditional requirement with the signature [arg, value]
  /// the requirement will only become valid if `arg`'s value equals `${value}`.
  pub requires_if: Option<Vec<String>>,
  /// Allows specifying that an argument is required conditionally with the signature [arg, value]
  /// the requirement will only become valid if the `arg`'s value equals `${value}`.
  pub required_if_eq: Option<Vec<String>>,
  /// Requires that options use the --option=val syntax
  /// i.e. an equals between the option and associated value.
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
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliConfig {
  /// Command description which will be shown on the help information.
  pub description: Option<String>,
  /// Command long description which will be shown on the help information.
  pub long_description: Option<String>,
  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed before the auto-generated help information.
  /// This is often used for header information.
  pub before_help: Option<String>,
  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed after the auto-generated help information.
  /// This is often used to describe how to use the arguments, or caveats to be noted.
  pub after_help: Option<String>,
  /// List of arguments for the command
  pub args: Option<Vec<CliArg>>,
  /// List of subcommands of this command
  pub subcommands: Option<HashMap<String, CliConfig>>,
}

impl CliConfig {
  /// List of arguments for the command
  pub fn args(&self) -> Option<&Vec<CliArg>> {
    self.args.as_ref()
  }

  /// List of subcommands of this command
  pub fn subcommands(&self) -> Option<&HashMap<String, CliConfig>> {
    self.subcommands.as_ref()
  }

  /// Command description which will be shown on the help information.
  pub fn description(&self) -> Option<&String> {
    self.description.as_ref()
  }

  /// Command long description which will be shown on the help information.
  pub fn long_description(&self) -> Option<&String> {
    self.description.as_ref()
  }

  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed before the auto-generated help information.
  /// This is often used for header information.
  pub fn before_help(&self) -> Option<&String> {
    self.before_help.as_ref()
  }

  /// Adds additional help information to be displayed in addition to auto-generated help.
  /// This information is displayed after the auto-generated help information.
  /// This is often used to describe how to use the arguments, or caveats to be noted.
  pub fn after_help(&self) -> Option<&String> {
    self.after_help.as_ref()
  }
}

/// The window configuration object.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowConfig {
  /// The window identifier. It must be alphanumeric.
  #[serde(default = "default_window_label")]
  pub label: String,
  /// The window webview URL.
  #[serde(default)]
  pub url: WindowUrl,
  /// Whether the file drop is enabled or not on the webview. By default it is enabled.
  ///
  /// Disabling it is required to use drag and drop on the frontend on Windows.
  #[serde(default = "default_file_drop_enabled")]
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
  pub min_width: Option<f64>,
  /// The min window height.
  pub min_height: Option<f64>,
  /// The max window width.
  pub max_width: Option<f64>,
  /// The max window height.
  pub max_height: Option<f64>,
  /// Whether the window is resizable or not.
  #[serde(default = "default_resizable")]
  pub resizable: bool,
  /// The window title.
  #[serde(default = "default_title")]
  pub title: String,
  /// Whether the window starts as fullscreen or not.
  #[serde(default)]
  pub fullscreen: bool,
  /// Whether the window will be initially hidden or focused.
  #[serde(default = "default_focus")]
  pub focus: bool,
  /// Whether the window is transparent or not.
  ///
  /// Note that on `macOS` this requires the `macos-private-api` feature flag, enabled under `tauri.conf.json > tauri > macOSPrivateApi`.
  /// WARNING: Using private APIs on `macOS` prevents your application from being accepted to the `App Store`.
  #[serde(default)]
  pub transparent: bool,
  /// Whether the window is maximized or not.
  #[serde(default)]
  pub maximized: bool,
  /// Whether the window is visible or not.
  #[serde(default = "default_visible")]
  pub visible: bool,
  /// Whether the window should have borders and bars.
  #[serde(default = "default_decorations")]
  pub decorations: bool,
  /// Whether the window should always be on top of other windows.
  #[serde(default)]
  pub always_on_top: bool,
  /// Whether or not the window icon should be added to the taskbar.
  #[serde(default)]
  pub skip_taskbar: bool,
  /// The initial window theme. Defaults to the system theme. Only implemented on Windows and macOS 10.14+.
  pub theme: Option<crate::Theme>,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      label: default_window_label(),
      url: WindowUrl::default(),
      file_drop_enabled: default_file_drop_enabled(),
      center: false,
      x: None,
      y: None,
      width: default_width(),
      height: default_height(),
      min_width: None,
      min_height: None,
      max_width: None,
      max_height: None,
      resizable: default_resizable(),
      title: default_title(),
      fullscreen: false,
      focus: false,
      transparent: false,
      maximized: false,
      visible: default_visible(),
      decorations: default_decorations(),
      always_on_top: false,
      skip_taskbar: false,
      theme: None,
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

fn default_resizable() -> bool {
  true
}

fn default_title() -> String {
  "Tauri App".to_string()
}

fn default_focus() -> bool {
  true
}

fn default_visible() -> bool {
  true
}

fn default_decorations() -> bool {
  true
}

fn default_file_drop_enabled() -> bool {
  true
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
      Self::Inline(s) => s.contains(&format!("{} ", source)) || s.contains(&format!(" {}", source)),
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
      Self::Policy(s) => write!(f, "{}", s),
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

/// Security configuration.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
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
  pub dev_csp: Option<Csp>,
  /// Freeze the `Object.prototype` when using the custom protocol.
  #[serde(default)]
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
  #[serde(default)]
  pub dangerous_disable_asset_csp_modification: DisabledCspModificationKind,
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
/// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum FsAllowlistScope {
  /// A list of paths that are allowed by this scope.
  AllowedPaths(Vec<PathBuf>),
  /// A complete scope configuration.
  Scope {
    /// A list of paths that are allowed by this scope.
    #[serde(default)]
    allow: Vec<PathBuf>,
    /// A list of paths that are not allowed by this scope.
    /// This gets precedence over the [`Self::Scope::allow`] list.
    #[serde(default)]
    deny: Vec<PathBuf>,
  },
}

impl Default for FsAllowlistScope {
  fn default() -> Self {
    Self::AllowedPaths(Vec::new())
  }
}

impl FsAllowlistScope {
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

/// Allowlist for the file system APIs.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsAllowlistConfig {
  /// The access scope for the filesystem APIs.
  #[serde(default)]
  pub scope: FsAllowlistScope,
  /// Use this flag to enable all file system API features.
  #[serde(default)]
  pub all: bool,
  /// Read file from local filesystem.
  #[serde(default)]
  pub read_file: bool,
  /// Write file to local filesystem.
  #[serde(default)]
  pub write_file: bool,
  /// Read directory from local filesystem.
  #[serde(default)]
  pub read_dir: bool,
  /// Copy file from local filesystem.
  #[serde(default)]
  pub copy_file: bool,
  /// Create directory from local filesystem.
  #[serde(default)]
  pub create_dir: bool,
  /// Remove directory from local filesystem.
  #[serde(default)]
  pub remove_dir: bool,
  /// Remove file from local filesystem.
  #[serde(default)]
  pub remove_file: bool,
  /// Rename file from local filesystem.
  #[serde(default)]
  pub rename_file: bool,
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
      features
    }
  }
}

/// Allowlist for the window APIs.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  #[serde(default)]
  pub request_user_attention: bool,
  /// Allows setting the resizable flag of the window.
  #[serde(default)]
  pub set_resizable: bool,
  /// Allows changing the window title.
  #[serde(default)]
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
  #[serde(default)]
  pub set_decorations: bool,
  /// Allows setting the always_on_top flag of the window.
  #[serde(default)]
  pub set_always_on_top: bool,
  /// Allows setting the window size.
  #[serde(default)]
  pub set_size: bool,
  /// Allows setting the window minimum size.
  #[serde(default)]
  pub set_min_size: bool,
  /// Allows setting the window maximum size.
  #[serde(default)]
  pub set_max_size: bool,
  /// Allows changing the position of the window.
  #[serde(default)]
  pub set_position: bool,
  /// Allows setting the fullscreen flag of the window.
  #[serde(default)]
  pub set_fullscreen: bool,
  /// Allows focusing the window.
  #[serde(default)]
  pub set_focus: bool,
  /// Allows changing the window icon.
  #[serde(default)]
  pub set_icon: bool,
  /// Allows setting the skip_taskbar flag of the window.
  #[serde(default)]
  pub set_skip_taskbar: bool,
  /// Allows start dragging on the window.
  #[serde(default)]
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
      set_size: true,
      set_min_size: true,
      set_max_size: true,
      set_position: true,
      set_fullscreen: true,
      set_focus: true,
      set_icon: true,
      set_skip_taskbar: true,
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
      check_feature!(self, features, set_size, "window-set-size");
      check_feature!(self, features, set_min_size, "window-set-min-size");
      check_feature!(self, features, set_max_size, "window-set-max-size");
      check_feature!(self, features, set_position, "window-set-position");
      check_feature!(self, features, set_fullscreen, "window-set-fullscreen");
      check_feature!(self, features, set_focus, "window-set-focus");
      check_feature!(self, features, set_icon, "window-set-icon");
      check_feature!(self, features, set_skip_taskbar, "window-set-skip-taskbar");
      check_feature!(self, features, start_dragging, "window-start-dragging");
      check_feature!(self, features, print, "window-print");
      features
    }
  }
}

/// A command allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`.
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ShellAllowlistScope(pub Vec<ShellAllowedCommand>);

/// Defines the `shell > open` api scope.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowlistOpen {
  /// If the shell open API should be enabled.
  ///
  /// If enabled, the default validation regex (`^https?://`) is used.
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
/// - "https://**": allows all HTTPS urls
/// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
/// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
#[allow(rustdoc::bare_urls)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct HttpAllowlistScope(pub Vec<Url>);

/// Allowlist for the HTTP APIs.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProtocolAllowlistConfig {
  /// The access scope for the asset protocol.
  #[serde(default)]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  #[serde(default)]
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
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipboardAllowlistConfig {
  /// Use this flag to enable all clipboard APIs.
  #[serde(default)]
  pub all: bool,
  /// Enables the clipboard's `writeText` API.
  #[serde(default)]
  pub write_text: bool,
  /// Enables the clipboard's `readText` API.
  #[serde(default)]
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

/// Allowlist configuration.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  #[serde(default)]
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
      features
    }
  }
}

/// The application pattern.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "use", content = "options")]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PatternKind {
  /// Brownfield pattern.
  Brownfield,
  /// Isolation pattern. Recommended for security purposes.
  #[cfg(feature = "isolation")]
  Isolation {
    /// The dir containing the index.html file that contains the secure isolation application.
    dir: PathBuf,
  },
}

impl Default for PatternKind {
  fn default() -> Self {
    Self::Brownfield
  }
}

/// The Tauri configuration object.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  pub system_tray: Option<SystemTrayConfig>,
  /// MacOS private API configuration. Enables the transparent background API and sets the `fullScreenEnabled` preference to `true`.
  #[serde(rename = "macOSPrivateApi", default)]
  pub macos_private_api: bool,
}

impl TauriConfig {
  /// Returns all Cargo features.
  #[allow(dead_code)]
  pub fn all_features() -> Vec<&'static str> {
    let mut features = AllowlistConfig::all_features();
    features.extend(vec![
      "cli",
      "updater",
      "system-tray",
      "macos-private-api",
      "isolation",
    ]);
    features
  }

  /// Returns the enabled Cargo features.
  #[allow(dead_code)]
  pub fn features(&self) -> Vec<&str> {
    let mut features = self.allowlist.to_features();
    if self.cli.is_some() {
      features.push("cli");
    }
    if self.updater.active {
      features.push("updater");
    }
    if self.system_tray.is_some() {
      features.push("system-tray");
    }
    if self.macos_private_api {
      features.push("macos-private-api");
    }
    #[cfg(feature = "isolation")]
    if let PatternKind::Isolation { .. } = self.pattern {
      features.push("isolation");
    }
    features.sort_unstable();
    features
  }
}

/// A URL to an updater server.
///
/// The URL must use the `https` scheme on production.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename_all = "camelCase"))]
pub enum WindowsUpdateInstallMode {
  /// Specifies there's a basic UI during the installation process, including a final dialog box at the end.
  BasicUi,
  /// The quiet mode means there's no user interaction required.
  /// Requires admin privileges if the installer does.
  Quiet,
  /// Specifies unattended mode, which means the installation only shows a progress bar.
  Passive,
}

impl WindowsUpdateInstallMode {
  /// Returns the associated `msiexec.exe` arguments.
  pub fn msiexec_args(&self) -> &'static [&'static str] {
    match self {
      Self::BasicUi => &["/qb+"],
      Self::Quiet => &["/quiet"],
      Self::Passive => &["/passive"],
    }
  }
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

impl Default for WindowsUpdateInstallMode {
  fn default() -> Self {
    Self::Passive
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
        "unknown update install mode '{}'",
        s
      ))),
    }
  }
}

/// The updater configuration for Windows.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterWindowsConfig {
  /// The installation mode for the update on Windows. Defaults to `passive`.
  #[serde(default)]
  pub install_mode: WindowsUpdateInstallMode,
}

/// The Updater configuration object.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterConfig {
  /// Whether the updater is active or not.
  #[serde(default)]
  pub active: bool,
  /// Display built-in dialog or use event system if disabled.
  #[serde(default = "default_dialog")]
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
      #[serde(default = "default_dialog")]
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
      dialog: default_dialog(),
      endpoints: None,
      pubkey: "".into(),
      windows: Default::default(),
    }
  }
}

/// Configuration for application system tray icon.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemTrayConfig {
  /// Path to the icon to use on the system tray.
  ///
  /// It is forced to be a `.png` file on Linux and macOS, and a `.ico` file on Windows.
  pub icon_path: PathBuf,
  /// A Boolean value that determines whether the image represents a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image on macOS.
  #[serde(default)]
  pub icon_as_template: bool,
}

// We enable the unnecessary_wraps because we need
// to use an Option for dialog otherwise the CLI schema will mark
// the dialog as a required field which is not as we default it to true.
fn default_dialog() -> bool {
  true
}

/// Defines the URL or assets to embed in the application.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
      Self::Url(url) => write!(f, "{}", url),
      Self::Files(files) => write!(f, "{}", serde_json::to_string(files).unwrap()),
    }
  }
}

/// The Build configuration object.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
  #[serde(default = "default_dev_path")]
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
  #[serde(default = "default_dist_dir")]
  pub dist_dir: AppUrl,
  /// A shell command to run before `tauri dev` kicks in.
  ///
  /// The TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG environment variables are set if you perform conditional compilation.
  pub before_dev_command: Option<String>,
  /// A shell command to run before `tauri build` kicks in.
  ///
  /// The TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG environment variables are set if you perform conditional compilation.
  pub before_build_command: Option<String>,
  /// Features passed to `cargo` commands.
  pub features: Option<Vec<String>>,
  /// Whether we should inject the Tauri API on `window.__TAURI__` or not.
  #[serde(default)]
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
            .map_err(|e| DeError::custom(format!("failed to read version JSON file: {}", e)))?;
          let package_json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| DeError::custom(format!("failed to read version JSON file: {}", e)))?;
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageConfig {
  /// App name.
  pub product_name: Option<String>,
  /// App version. It is a semver version number or a path to a `package.json` file contaning the `version` field.
  #[serde(deserialize_with = "version_deserializer", default)]
  pub version: Option<String>,
}

fn version_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<PackageVersion>::deserialize(deserializer).map(|v| v.map(|v| v.0))
}

impl PackageConfig {
  /// The binary name.
  #[allow(dead_code)]
  pub fn binary_name(&self) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
      self.product_name.as_ref().map(|n| n.to_kebab_case())
    }
    #[cfg(not(target_os = "linux"))]
    {
      self.product_name.clone()
    }
  }
}

/// The tauri.conf.json is a file generated by the
/// [`tauri init`](https://tauri.app/v1/api/cli#init) command that lives in
/// your Tauri application source directory (src-tauri).
///
/// Once generated, you may modify it at will to customize your Tauri application.
///
/// ## Platform-Specific Configuration
///
/// In addition to the JSON defined on the `tauri.conf.json` file, Tauri can
/// read a platform-specific configuration from `tauri.linux.conf.json`,
/// `tauri.windows.conf.json`, and `tauri.macos.conf.json` and merges it with
/// the main `tauri.conf.json` configuration.
///
/// ## Configuration Structure
///
/// `tauri.conf.json` is composed of the following objects:
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
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct PluginConfig(pub HashMap<String, JsonValue>);

fn default_build() -> BuildConfig {
  BuildConfig {
    runner: None,
    dev_path: default_dev_path(),
    dist_dir: default_dist_dir(),
    before_dev_command: None,
    before_build_command: None,
    features: None,
    with_global_tauri: false,
  }
}

/// Implement `ToTokens` for all config structs, allowing a literal `Config` to be built.
///
/// This allows for a build script to output the values in a `Config` to a `TokenStream`, which can
/// then be consumed by another crate. Useful for passing a config to both the build script and the
/// application using tauri while only parsing it once (in the build script).
#[cfg(feature = "build")]
mod build {
  use std::{convert::identity, path::Path};

  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  use super::*;

  use serde_json::Value as JsonValue;

  /// Create a `String` constructor `TokenStream`.
  ///
  /// e.g. `"Hello World" -> String::from("Hello World").
  /// This takes a `&String` to reduce casting all the `&String` -> `&str` manually.
  fn str_lit(s: impl AsRef<str>) -> TokenStream {
    let s = s.as_ref();
    quote! { #s.into() }
  }

  /// Create an `Option` constructor `TokenStream`.
  fn opt_lit(item: Option<&impl ToTokens>) -> TokenStream {
    match item {
      None => quote! { ::core::option::Option::None },
      Some(item) => quote! { ::core::option::Option::Some(#item) },
    }
  }

  /// Helper function to combine an `opt_lit` with `str_lit`.
  fn opt_str_lit(item: Option<impl AsRef<str>>) -> TokenStream {
    opt_lit(item.map(str_lit).as_ref())
  }

  /// Helper function to combine an `opt_lit` with a list of `str_lit`
  fn opt_vec_str_lit(item: Option<impl IntoIterator<Item = impl AsRef<str>>>) -> TokenStream {
    opt_lit(item.map(|list| vec_lit(list, str_lit)).as_ref())
  }

  /// Create a `Vec` constructor, mapping items with a function that spits out `TokenStream`s.
  fn vec_lit<Raw, Tokens>(
    list: impl IntoIterator<Item = Raw>,
    map: impl Fn(Raw) -> Tokens,
  ) -> TokenStream
  where
    Tokens: ToTokens,
  {
    let items = list.into_iter().map(map);
    quote! { vec![#(#items),*] }
  }

  /// Create a `PathBuf` constructor `TokenStream`.
  ///
  /// e.g. `"Hello World" -> String::from("Hello World").
  fn path_buf_lit(s: impl AsRef<Path>) -> TokenStream {
    let s = s.as_ref().to_string_lossy().into_owned();
    quote! { ::std::path::PathBuf::from(#s) }
  }

  /// Creates a `Url` constructor `TokenStream`.
  fn url_lit(url: &Url) -> TokenStream {
    let url = url.as_str();
    quote! { #url.parse().unwrap() }
  }

  /// Create a map constructor, mapping keys and values with other `TokenStream`s.
  ///
  /// This function is pretty generic because the types of keys AND values get transformed.
  fn map_lit<Map, Key, Value, TokenStreamKey, TokenStreamValue, FuncKey, FuncValue>(
    map_type: TokenStream,
    map: Map,
    map_key: FuncKey,
    map_value: FuncValue,
  ) -> TokenStream
  where
    <Map as IntoIterator>::IntoIter: ExactSizeIterator,
    Map: IntoIterator<Item = (Key, Value)>,
    TokenStreamKey: ToTokens,
    TokenStreamValue: ToTokens,
    FuncKey: Fn(Key) -> TokenStreamKey,
    FuncValue: Fn(Value) -> TokenStreamValue,
  {
    let ident = quote::format_ident!("map");
    let map = map.into_iter();

    if map.len() > 0 {
      let items = map.map(|(key, value)| {
        let key = map_key(key);
        let value = map_value(value);
        quote! { #ident.insert(#key, #value); }
      });

      quote! {{
        let mut #ident = #map_type::new();
        #(#items)*
        #ident
      }}
    } else {
      quote! { #map_type::new() }
    }
  }

  /// Create a `serde_json::Value` variant `TokenStream` for a number
  fn json_value_number_lit(num: &serde_json::Number) -> TokenStream {
    // See https://docs.rs/serde_json/1/serde_json/struct.Number.html for guarantees
    let prefix = quote! { ::serde_json::Value };
    if num.is_u64() {
      // guaranteed u64
      let num = num.as_u64().unwrap();
      quote! { #prefix::Number(#num.into()) }
    } else if num.is_i64() {
      // guaranteed i64
      let num = num.as_i64().unwrap();
      quote! { #prefix::Number(#num.into()) }
    } else if num.is_f64() {
      // guaranteed f64
      let num = num.as_f64().unwrap();
      quote! { #prefix::Number(#num.into()) }
    } else {
      // invalid number
      quote! { #prefix::Null }
    }
  }

  /// Create a `serde_json::Value` constructor `TokenStream`
  fn json_value_lit(jv: &JsonValue) -> TokenStream {
    let prefix = quote! { ::serde_json::Value };

    match jv {
      JsonValue::Null => quote! { #prefix::Null },
      JsonValue::Bool(bool) => quote! { #prefix::Bool(#bool) },
      JsonValue::Number(number) => json_value_number_lit(number),
      JsonValue::String(str) => {
        let s = str_lit(str);
        quote! { #prefix::String(#s) }
      }
      JsonValue::Array(vec) => {
        let items = vec.iter().map(json_value_lit);
        quote! { #prefix::Array(vec![#(#items),*]) }
      }
      JsonValue::Object(map) => {
        let map = map_lit(quote! { ::serde_json::Map }, map, str_lit, json_value_lit);
        quote! { #prefix::Object(#map) }
      }
    }
  }

  /// Write a `TokenStream` of the `$struct`'s fields to the `$tokens`.
  ///
  /// All fields must represent a binding of the same name that implements `ToTokens`.
  macro_rules! literal_struct {
    ($tokens:ident, $struct:ident, $($field:ident),+) => {
      $tokens.append_all(quote! {
        ::tauri::utils::config::$struct {
          $($field: #$field),+
        }
      });
    };
  }

  impl ToTokens for WindowUrl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::WindowUrl };

      tokens.append_all(match self {
        Self::App(path) => {
          let path = path_buf_lit(&path);
          quote! { #prefix::App(#path) }
        }
        Self::External(url) => {
          let url = url_lit(url);
          quote! { #prefix::External(#url) }
        }
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

  impl ToTokens for WindowConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let label = str_lit(&self.label);
      let url = &self.url;
      let file_drop_enabled = self.file_drop_enabled;
      let center = self.center;
      let x = opt_lit(self.x.as_ref());
      let y = opt_lit(self.y.as_ref());
      let width = self.width;
      let height = self.height;
      let min_width = opt_lit(self.min_width.as_ref());
      let min_height = opt_lit(self.min_height.as_ref());
      let max_width = opt_lit(self.max_width.as_ref());
      let max_height = opt_lit(self.max_height.as_ref());
      let resizable = self.resizable;
      let title = str_lit(&self.title);
      let fullscreen = self.fullscreen;
      let focus = self.focus;
      let transparent = self.transparent;
      let maximized = self.maximized;
      let visible = self.visible;
      let decorations = self.decorations;
      let always_on_top = self.always_on_top;
      let skip_taskbar = self.skip_taskbar;
      let theme = opt_lit(self.theme.as_ref());

      literal_struct!(
        tokens,
        WindowConfig,
        label,
        url,
        file_drop_enabled,
        center,
        x,
        y,
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        resizable,
        title,
        fullscreen,
        focus,
        transparent,
        maximized,
        visible,
        decorations,
        always_on_top,
        skip_taskbar,
        theme
      );
    }
  }

  impl ToTokens for CliArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let short = opt_lit(self.short.as_ref());
      let name = str_lit(&self.name);
      let description = opt_str_lit(self.description.as_ref());
      let long_description = opt_str_lit(self.long_description.as_ref());
      let takes_value = self.takes_value;
      let multiple = self.multiple;
      let multiple_occurrences = self.multiple_occurrences;
      let number_of_values = opt_lit(self.number_of_values.as_ref());
      let possible_values = opt_vec_str_lit(self.possible_values.as_ref());
      let min_values = opt_lit(self.min_values.as_ref());
      let max_values = opt_lit(self.max_values.as_ref());
      let required = self.required;
      let required_unless_present = opt_str_lit(self.required_unless_present.as_ref());
      let required_unless_present_all = opt_vec_str_lit(self.required_unless_present_all.as_ref());
      let required_unless_present_any = opt_vec_str_lit(self.required_unless_present_any.as_ref());
      let conflicts_with = opt_str_lit(self.conflicts_with.as_ref());
      let conflicts_with_all = opt_vec_str_lit(self.conflicts_with_all.as_ref());
      let requires = opt_str_lit(self.requires.as_ref());
      let requires_all = opt_vec_str_lit(self.requires_all.as_ref());
      let requires_if = opt_vec_str_lit(self.requires_if.as_ref());
      let required_if_eq = opt_vec_str_lit(self.required_if_eq.as_ref());
      let require_equals = opt_lit(self.require_equals.as_ref());
      let index = opt_lit(self.index.as_ref());

      literal_struct!(
        tokens,
        CliArg,
        short,
        name,
        description,
        long_description,
        takes_value,
        multiple,
        multiple_occurrences,
        number_of_values,
        possible_values,
        min_values,
        max_values,
        required,
        required_unless_present,
        required_unless_present_all,
        required_unless_present_any,
        conflicts_with,
        conflicts_with_all,
        requires,
        requires_all,
        requires_if,
        required_if_eq,
        require_equals,
        index
      );
    }
  }

  impl ToTokens for CliConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let description = opt_str_lit(self.description.as_ref());
      let long_description = opt_str_lit(self.long_description.as_ref());
      let before_help = opt_str_lit(self.before_help.as_ref());
      let after_help = opt_str_lit(self.after_help.as_ref());
      let args = {
        let args = self.args.as_ref().map(|args| {
          let arg = args.iter().map(|a| quote! { #a });
          quote! { vec![#(#arg),*] }
        });
        opt_lit(args.as_ref())
      };
      let subcommands = opt_lit(
        self
          .subcommands
          .as_ref()
          .map(|map| {
            map_lit(
              quote! { ::std::collections::HashMap },
              map,
              str_lit,
              identity,
            )
          })
          .as_ref(),
      );

      literal_struct!(
        tokens,
        CliConfig,
        description,
        long_description,
        before_help,
        after_help,
        args,
        subcommands
      );
    }
  }

  impl ToTokens for PatternKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::PatternKind };

      tokens.append_all(match self {
        Self::Brownfield => quote! { #prefix::Brownfield },
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
          let path = path_buf_lit(&path);
          quote! { #prefix::FixedRuntime { path: #path } }
        }
      })
    }
  }

  impl ToTokens for WindowsConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let webview_install_mode = if let Some(fixed_runtime_path) = &self.webview_fixed_runtime_path
      {
        WebviewInstallMode::FixedRuntime {
          path: fixed_runtime_path.clone(),
        }
      } else {
        self.webview_install_mode.clone()
      };
      tokens.append_all(quote! { ::tauri::utils::config::WindowsConfig {
        webview_install_mode: #webview_install_mode,
        ..Default::default()
      }})
    }
  }

  impl ToTokens for BundleConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let identifier = str_lit(&self.identifier);
      let icon = vec_lit(&self.icon, str_lit);
      let active = self.active;
      let targets = quote!(Default::default());
      let resources = quote!(None);
      let copyright = quote!(None);
      let category = quote!(None);
      let short_description = quote!(None);
      let long_description = quote!(None);
      let appimage = quote!(Default::default());
      let deb = quote!(Default::default());
      let macos = quote!(Default::default());
      let external_bin = opt_vec_str_lit(self.external_bin.as_ref());
      let windows = &self.windows;

      literal_struct!(
        tokens,
        BundleConfig,
        active,
        identifier,
        icon,
        targets,
        resources,
        copyright,
        category,
        short_description,
        long_description,
        appimage,
        deb,
        macos,
        external_bin,
        windows
      );
    }
  }

  impl ToTokens for AppUrl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::AppUrl };

      tokens.append_all(match self {
        Self::Url(url) => {
          quote! { #prefix::Url(#url) }
        }
        Self::Files(files) => {
          let files = vec_lit(files, path_buf_lit);
          quote! { #prefix::Files(#files) }
        }
      })
    }
  }

  impl ToTokens for BuildConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let dev_path = &self.dev_path;
      let dist_dir = &self.dist_dir;
      let with_global_tauri = self.with_global_tauri;
      let runner = quote!(None);
      let before_dev_command = quote!(None);
      let before_build_command = quote!(None);
      let features = quote!(None);

      literal_struct!(
        tokens,
        BuildConfig,
        runner,
        dev_path,
        dist_dir,
        with_global_tauri,
        before_dev_command,
        before_build_command,
        features
      );
    }
  }

  impl ToTokens for WindowsUpdateInstallMode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::WindowsUpdateInstallMode };

      tokens.append_all(match self {
        Self::BasicUi => quote! { #prefix::BasicUi },
        Self::Quiet => quote! { #prefix::Quiet },
        Self::Passive => quote! { #prefix::Passive },
      })
    }
  }

  impl ToTokens for UpdaterWindowsConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let install_mode = &self.install_mode;
      literal_struct!(tokens, UpdaterWindowsConfig, install_mode);
    }
  }

  impl ToTokens for UpdaterConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let active = self.active;
      let dialog = self.dialog;
      let pubkey = str_lit(&self.pubkey);
      let endpoints = opt_lit(
        self
          .endpoints
          .as_ref()
          .map(|list| {
            vec_lit(list, |url| {
              let url = url.0.as_str();
              quote! { ::tauri::utils::config::UpdaterEndpoint(#url.parse().unwrap()) }
            })
          })
          .as_ref(),
      );
      let windows = &self.windows;

      literal_struct!(
        tokens,
        UpdaterConfig,
        active,
        dialog,
        pubkey,
        endpoints,
        windows
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

  impl ToTokens for SecurityConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let csp = opt_lit(self.csp.as_ref());
      let dev_csp = opt_lit(self.dev_csp.as_ref());
      let freeze_prototype = self.freeze_prototype;
      let dangerous_disable_asset_csp_modification = &self.dangerous_disable_asset_csp_modification;

      literal_struct!(
        tokens,
        SecurityConfig,
        csp,
        dev_csp,
        freeze_prototype,
        dangerous_disable_asset_csp_modification
      );
    }
  }

  impl ToTokens for SystemTrayConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let icon_as_template = self.icon_as_template;
      let icon_path = path_buf_lit(&self.icon_path);
      literal_struct!(tokens, SystemTrayConfig, icon_path, icon_as_template);
    }
  }

  impl ToTokens for FsAllowlistScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::FsAllowlistScope };

      tokens.append_all(match self {
        Self::AllowedPaths(allow) => {
          let allowed_paths = vec_lit(allow, path_buf_lit);
          quote! { #prefix::AllowedPaths(#allowed_paths) }
        }
        Self::Scope { allow, deny } => {
          let allow = vec_lit(allow, path_buf_lit);
          let deny = vec_lit(deny, path_buf_lit);
          quote! { #prefix::Scope { allow: #allow, deny: #deny } }
        }
      });
    }
  }

  impl ToTokens for FsAllowlistConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let scope = &self.scope;
      tokens.append_all(quote! { ::tauri::utils::config::FsAllowlistConfig { scope: #scope, ..Default::default() } })
    }
  }

  impl ToTokens for ProtocolAllowlistConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let asset_scope = &self.asset_scope;
      tokens.append_all(quote! { ::tauri::utils::config::ProtocolAllowlistConfig { asset_scope: #asset_scope, ..Default::default() } })
    }
  }

  impl ToTokens for HttpAllowlistScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allowed_urls = vec_lit(&self.0, url_lit);
      tokens.append_all(quote! { ::tauri::utils::config::HttpAllowlistScope(#allowed_urls) })
    }
  }

  impl ToTokens for HttpAllowlistConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let scope = &self.scope;
      tokens.append_all(quote! { ::tauri::utils::config::HttpAllowlistConfig { scope: #scope, ..Default::default() } })
    }
  }

  impl ToTokens for ShellAllowedCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let name = str_lit(&self.name);
      let command = path_buf_lit(&self.command);
      let args = &self.args;
      let sidecar = &self.sidecar;

      literal_struct!(tokens, ShellAllowedCommand, name, command, args, sidecar);
    }
  }

  impl ToTokens for ShellAllowedArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::ShellAllowedArgs };

      tokens.append_all(match self {
        Self::Flag(flag) => quote!(#prefix::Flag(#flag)),
        Self::List(list) => {
          let list = vec_lit(list, identity);
          quote!(#prefix::List(#list))
        }
      })
    }
  }

  impl ToTokens for ShellAllowedArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::ShellAllowedArg };

      tokens.append_all(match self {
        Self::Fixed(fixed) => {
          let fixed = str_lit(fixed);
          quote!(#prefix::Fixed(#fixed))
        }
        Self::Var { validator } => {
          let validator = str_lit(validator);
          quote!(#prefix::Var { validator: #validator })
        }
      })
    }
  }

  impl ToTokens for ShellAllowlistOpen {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::utils::config::ShellAllowlistOpen };

      tokens.append_all(match self {
        Self::Flag(flag) => quote!(#prefix::Flag(#flag)),
        Self::Validate(regex) => quote!(#prefix::Validate(#regex)),
      })
    }
  }

  impl ToTokens for ShellAllowlistScope {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let allowed_commands = vec_lit(&self.0, identity);
      tokens.append_all(quote! { ::tauri::utils::config::ShellAllowlistScope(#allowed_commands) })
    }
  }

  impl ToTokens for ShellAllowlistConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let scope = &self.scope;
      tokens.append_all(quote! { ::tauri::utils::config::ShellAllowlistConfig { scope: #scope, ..Default::default() } })
    }
  }

  impl ToTokens for AllowlistConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let fs = &self.fs;
      let protocol = &self.protocol;
      let http = &self.http;
      let shell = &self.shell;
      tokens.append_all(
        quote! { ::tauri::utils::config::AllowlistConfig { fs: #fs, protocol: #protocol, http: #http, shell: #shell, ..Default::default() } },
      )
    }
  }

  impl ToTokens for TauriConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let pattern = &self.pattern;
      let windows = vec_lit(&self.windows, identity);
      let cli = opt_lit(self.cli.as_ref());
      let bundle = &self.bundle;
      let updater = &self.updater;
      let security = &self.security;
      let system_tray = opt_lit(self.system_tray.as_ref());
      let allowlist = &self.allowlist;
      let macos_private_api = self.macos_private_api;

      literal_struct!(
        tokens,
        TauriConfig,
        pattern,
        windows,
        cli,
        bundle,
        updater,
        security,
        system_tray,
        allowlist,
        macos_private_api
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

  impl ToTokens for PackageConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let product_name = opt_str_lit(self.product_name.as_ref());
      let version = opt_str_lit(self.version.as_ref());

      literal_struct!(tokens, PackageConfig, product_name, version);
    }
  }

  impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let schema = quote!(None);
      let package = &self.package;
      let tauri = &self.tauri;
      let build = &self.build;
      let plugins = &self.plugins;

      literal_struct!(tokens, Config, schema, package, tauri, build, plugins);
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
