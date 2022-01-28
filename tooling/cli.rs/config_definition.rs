// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::field_reassign_with_default)]

#[cfg(target_os = "linux")]
use heck::ToKebabCase;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::skip_serializing_none;
use url::Url;

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum BundleTarget {
  All(Vec<String>),
  One(String),
}

impl BundleTarget {
  #[allow(dead_code)]
  pub fn to_vec(&self) -> Vec<String> {
    match self {
      Self::All(list) => list.clone(),
      Self::One(i) => vec![i.clone()],
    }
  }
}

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DebConfig {
  /// The list of deb dependencies your application relies on.
  pub depends: Option<Vec<String>>,
  /// Enable the boostrapper script.
  #[serde(default)]
  pub use_bootstrapper: bool,
  /// The files to include on the package.
  #[serde(default)]
  pub files: HashMap<PathBuf, PathBuf>,
}

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MacConfig {
  /// A list of strings indicating any macOS X frameworks that need to be bundled with the application.
  ///
  /// If a name is used, ".framework" must be omitted and it will look for standard install locations. You may also use a path to a specific framework.
  pub frameworks: Option<Vec<String>>,
  /// A version string indicating the minimum macOS X version that the bundled application supports.
  pub minimum_system_version: Option<String>,
  /// Allows your application to communicate with the outside world.
  /// It should be a lowercase, without port and protocol domain name.
  pub exception_domain: Option<String>,
  /// The path to the license file to add to the DMG bundle.
  pub license: Option<String>,
  /// Enable the boostrapper script.
  #[serde(default)]
  pub use_bootstrapper: bool,
  /// Identity to use for code signing.
  pub signing_identity: Option<String>,
  /// Provider short name for notarization.
  pub provider_short_name: Option<String>,
  /// Path to the entitlements file.
  pub entitlements: Option<String>,
}

fn default_language() -> String {
  "en-US".into()
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WixConfig {
  /// The installer language. See https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables.
  #[serde(default = "default_language")]
  pub language: String,
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
  #[serde(default)]
  pub skip_webview_install: bool,
  /// The path to the license file to render on the installer.
  ///
  /// Must be an RTF file, so if a different extension is provided, we convert it to the RTF format.
  pub license: Option<PathBuf>,
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

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowsConfig {
  /// Specifies the file digest algorithm to use for creating file signatures.
  /// Required for code signing. SHA-256 is recommended.
  pub digest_algorithm: Option<String>,
  /// Specifies the SHA1 hash of the signing certificate.
  pub certificate_thumbprint: Option<String>,
  /// Server to use during timestamping.
  pub timestamp_url: Option<String>,
  /// Configuration for the MSI generated with WiX.
  pub wix: Option<WixConfig>,
}

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageConfig {
  /// Application name. Automatically converted to kebab-case on Linux.
  pub product_name: Option<String>,
  /// Application version.
  pub version: Option<String>,
}

impl PackageConfig {
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

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BundleConfig {
  /// Whether we should build your app with tauri-bundler or plain `cargo build`
  #[serde(default)]
  pub active: bool,
  /// The bundle targets, currently supports ["deb", "app", "msi", "appimage", "dmg"] or "all"
  pub targets: Option<BundleTarget>,
  /// The app's identifier
  pub identifier: Option<String>,
  /// The app's icons
  pub icon: Option<Vec<String>>,
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

/// A CLI argument definition
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliArg {
  /// The short version of the argument, without the preceding -.
  ///
  /// NOTE: Any leading - characters will be stripped, and only the first non - character will be used as the short version.
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
  pub takes_value: Option<bool>,
  /// Specifies that the argument may appear more than once.
  ///
  /// - For flags, this results in the number of occurrences of the flag being recorded.
  /// For example -ddd or -d -d -d would count as three occurrences.
  /// - For options there is a distinct difference in multiple occurrences vs multiple values.
  /// For example, --opt val1 val2 is one occurrence, but two values. Whereas --opt val1 --opt val2 is two occurrences.
  pub multiple: Option<bool>,
  /// specifies that the argument may appear more than once.
  pub multiple_occurrences: Option<bool>,
  ///
  pub number_of_values: Option<u64>,
  /// Specifies a list of possible values for this argument.
  /// At runtime, the CLI verifies that only one of the specified values was used, or fails with an error message.
  pub possible_values: Option<Vec<String>>,
  /// Specifies the minimum number of values for this argument.
  /// For example, if you had a -f <file> argument where you wanted at least 2 'files',
  /// you would set `minValues: 2`, and this argument would be satisfied if the user provided, 2 or more values.
  pub min_values: Option<u64>,
  /// Specifies the maximum number of values are for this argument.
  /// For example, if you had a -f <file> argument where you wanted up to 3 'files',
  /// you would set .max_values(3), and this argument would be satisfied if the user provided, 1, 2, or 3 values.
  pub max_values: Option<u64>,
  /// Sets whether or not the argument is required by default.
  ///
  /// - Required by default means it is required, when no other conflicting rules have been evaluated
  /// - Conflicting rules take precedence over being required.
  pub required: Option<bool>,
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
  pub index: Option<u64>,
}

/// describes a CLI configuration
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CliConfig {
  /// command description which will be shown on the help information
  description: Option<String>,
  /// command long description which will be shown on the help information
  long_description: Option<String>,
  /// adds additional help information to be displayed in addition to auto-generated help
  /// this information is displayed before the auto-generated help information.
  /// this is often used for header information
  before_help: Option<String>,
  /// adds additional help information to be displayed in addition to auto-generated help
  /// this information is displayed after the auto-generated help information
  /// this is often used to describe how to use the arguments, or caveats to be noted.
  after_help: Option<String>,
  /// list of args for the command
  args: Option<Vec<CliArg>>,
  /// list of subcommands of this command.
  ///
  /// subcommands are effectively sub-apps, because they can contain their own arguments, subcommands, usage, etc.
  /// they also function just like the app command, in that they get their own auto generated help and usage
  subcommands: Option<HashMap<String, CliConfig>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Port {
  /// Port with a numeric value.
  Value(u16),
  /// Random port.
  Random,
}

/// The window configuration object.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowConfig {
  /// The window identifier.
  pub label: Option<String>,
  /// The window webview URL.
  pub url: Option<String>,
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
  pub width: Option<f64>,
  /// The window height.
  pub height: Option<f64>,
  /// The min window width.
  pub min_width: Option<f64>,
  /// The min window height.
  pub min_height: Option<f64>,
  /// The max window width.
  pub max_width: Option<f64>,
  /// The max window height.
  pub max_height: Option<f64>,
  /// Whether the window is resizable or not.
  #[serde(default)]
  pub resizable: bool,
  /// The window title.
  pub title: Option<String>,
  /// Whether the window starts as fullscreen or not.
  #[serde(default)]
  pub fullscreen: bool,
  /// Whether the window will be initially hidden or focused.
  #[serde(default = "default_focus")]
  pub focus: bool,
  /// Whether the window is transparent or not.
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

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityConfig {
  /// The Content Security Policy that will be injected on all HTML files.
  ///
  /// This is a really important part of the configuration since it helps you ensure your WebView is secured.
  /// See https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP.
  pub csp: Option<String>,
}

pub trait Allowlist {
  fn to_features(self) -> Vec<&'static str>;
}

macro_rules! check_feature {
  ($self:ident, $features:ident, $flag:ident, $feature_name: expr) => {
    if $self.$flag {
      $features.push($feature_name)
    }
  };
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsAllowlistConfig {
  /// Use this flag to enable all file system API features.
  #[serde(default)]
  pub all: bool,
  /// Read text file from local filesystem.
  #[serde(default)]
  pub read_text_file: bool,
  /// Read binary file from local filesystem.
  #[serde(default)]
  pub read_binary_file: bool,
  /// Write text file to local filesystem.
  #[serde(default)]
  pub write_file: bool,
  /// Write binary file to local filesystem.
  #[serde(default)]
  pub write_binary_file: bool,
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
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["fs-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, read_text_file, "fs-read-text-file");
      check_feature!(self, features, read_binary_file, "fs-read-binary-file");
      check_feature!(self, features, write_file, "fs-write-file");
      check_feature!(self, features, write_binary_file, "fs-write-binary-file");
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

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowAllowlistConfig {
  /// Use this flag to enable all window API features.
  #[serde(default)]
  pub all: bool,
  /// Allows dynamic window creation.
  #[serde(default)]
  pub create: bool,
}

impl Allowlist for WindowAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["window-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, create, "window-create");
      features
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellAllowlistConfig {
  /// Use this flag to enable all shell API features.
  #[serde(default)]
  pub all: bool,
  /// Enable binary execution.
  #[serde(default)]
  pub execute: bool,
  /// Open URL with the user's default application.
  #[serde(default)]
  pub open: bool,
}

impl Allowlist for ShellAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["shell-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, execute, "shell-execute");
      check_feature!(self, features, open, "shell-open");
      features
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DialogAllowlistConfig {
  /// Use this flag to enable all dialog API features.
  #[serde(default)]
  pub all: bool,
  /// Open dialog window to pick files.
  #[serde(default)]
  pub open: bool,
  /// Open dialog window to pick where to save files.
  #[serde(default)]
  pub save: bool,
}

impl Allowlist for DialogAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["dialog-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, open, "dialog-open");
      check_feature!(self, features, save, "dialog-save");
      features
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpAllowlistConfig {
  /// Use this flag to enable all HTTP API features.
  #[serde(default)]
  pub all: bool,
  /// Allows making HTTP requests.
  #[serde(default)]
  pub request: bool,
}

impl Allowlist for HttpAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["http-all"]
    } else {
      let mut features = Vec::new();
      check_feature!(self, features, request, "http-request");
      features
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationAllowlistConfig {
  /// Use this flag to enable all notification API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for NotificationAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["notification-all"]
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlobalShortcutAllowlistConfig {
  /// Use this flag to enable all global shortcut API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for GlobalShortcutAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["global-shortcut-all"]
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OsAllowlistConfig {
  /// Use this flag to enable all OS API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for OsAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["os-all"]
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathAllowlistConfig {
  /// Use this flag to enable all path API features.
  #[serde(default)]
  pub all: bool,
}

impl Allowlist for PathAllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
    if self.all {
      vec!["path-all"]
    } else {
      vec![]
    }
  }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
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
}

impl Allowlist for AllowlistConfig {
  fn to_features(self) -> Vec<&'static str> {
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
      features
    }
  }
}

/// The Tauri configuration object.
#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TauriConfig {
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
  allowlist: AllowlistConfig,
  /// Security configuration.
  pub security: Option<SecurityConfig>,
  /// The updater configuration.
  #[serde(default = "default_updater")]
  pub updater: UpdaterConfig,
  /// Configuration for app system tray.
  pub system_tray: Option<SystemTrayConfig>,
}

impl TauriConfig {
  #[allow(dead_code)]
  pub fn features(&self) -> Vec<&str> {
    self.allowlist.clone().to_features()
  }
}

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterConfig {
  /// Whether the updater is active or not.
  #[serde(default)]
  pub active: bool,
  /// Display built-in dialog or use event system if disabled.
  #[serde(default = "default_dialog")]
  pub dialog: Option<bool>,
  /// The updater endpoints.
  pub endpoints: Option<Vec<String>>,
  /// Optional pubkey.
  pub pubkey: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
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
#[allow(clippy::unnecessary_wraps)]
fn default_dialog() -> Option<bool> {
  Some(true)
}

/// The window webview URL options.
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
#[non_exhaustive]
pub enum WindowUrl {
  /// An external URL.
  External(Url),
  /// An app URL.
  App(PathBuf),
}

impl Default for WindowUrl {
  fn default() -> Self {
    Self::App("index.html".into())
  }
}

impl std::fmt::Display for WindowUrl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::External(url) => write!(f, "{}", url),
      Self::App(path) => write!(f, "{}", path.display()),
    }
  }
}

/// The `dev_path` and `dist_dir` options.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
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
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildConfig {
  /// The binary used to build and run the application.
  pub runner: Option<String>,
  /// The path or URL to use on development.
  #[serde(default = "default_dev_path")]
  pub dev_path: AppUrl,
  /// The path to the app's dist dir. This path must contain your index.html file.
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

fn default_dev_path() -> AppUrl {
  AppUrl::Url(WindowUrl::External(
    Url::parse("http://localhost:8080").unwrap(),
  ))
}

fn default_dist_dir() -> AppUrl {
  AppUrl::Url(WindowUrl::App(PathBuf::from("../dist")))
}

/// The tauri.conf.json mapper.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
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
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, JsonSchema)]
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

fn default_updater() -> UpdaterConfig {
  UpdaterConfig {
    active: false,
    dialog: Some(true),
    endpoints: None,
    pubkey: None,
  }
}
