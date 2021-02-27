use serde::{
  de::{Deserializer, Visitor},
  Deserialize,
};
use serde_json::Value as JsonValue;

use std::collections::HashMap;

/// The window webview URL options.
#[derive(PartialEq, Debug, Clone)]
pub enum WindowUrl {
  /// The app's index URL.
  App,
  /// A custom URL.
  Custom(String),
}

impl Default for WindowUrl {
  fn default() -> Self {
    Self::App
  }
}

impl<'de> Deserialize<'de> for WindowUrl {
  fn deserialize<D>(deserializer: D) -> Result<WindowUrl, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringVisitor;
    impl<'de> Visitor<'de> for StringVisitor {
      type Value = WindowUrl;
      fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string representing an url")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        if v.to_lowercase() == "app" {
          Ok(WindowUrl::App)
        } else {
          Ok(WindowUrl::Custom(v.to_string()))
        }
      }
    }
    deserializer.deserialize_str(StringVisitor)
  }
}

/// The window configuration object.
#[derive(PartialEq, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
  #[serde(default = "default_window_label")]
  /// The window identifier.
  pub label: String,
  /// The window webview URL.
  #[serde(default)]
  pub url: WindowUrl,
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

fn default_visible() -> bool {
  true
}

fn default_decorations() -> bool {
  true
}

fn default_title() -> String {
  "Tauri App".to_string()
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      label: default_window_label(),
      url: WindowUrl::App,
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
      transparent: false,
      maximized: false,
      visible: default_visible(),
      decorations: default_decorations(),
      always_on_top: false,
    }
  }
}

/// A CLI argument definition
#[derive(PartialEq, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
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
  ///
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

/// The CLI root command definition.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "cli", rename_all = "camelCase")]
pub struct CliConfig {
  description: Option<String>,
  long_description: Option<String>,
  before_help: Option<String>,
  after_help: Option<String>,
  args: Option<Vec<CliArg>>,
  subcommands: Option<HashMap<String, CliConfig>>,
}

impl CliConfig {
  /// List of args for the command
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

/// The bundler configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "bundle", rename_all = "camelCase")]
pub struct BundleConfig {
  /// The bundle identifier.
  pub identifier: String,
}

impl Default for BundleConfig {
  fn default() -> Self {
    Self {
      identifier: String::from(""),
    }
  }
}

fn default_window_config() -> Vec<WindowConfig> {
  vec![Default::default()]
}

/// The Tauri configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  /// The window configuration.
  #[serde(default = "default_window_config")]
  pub windows: Vec<WindowConfig>,
  /// The CLI configuration.
  #[serde(default)]
  pub cli: Option<CliConfig>,
  /// The bundler configuration.
  #[serde(default)]
  pub bundle: BundleConfig,
}

impl Default for TauriConfig {
  fn default() -> Self {
    Self {
      windows: default_window_config(),
      cli: None,
      bundle: BundleConfig::default(),
    }
  }
}

/// The Build configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  /// the devPath config.
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
  /// the dist config.
  #[serde(default = "default_dist_path")]
  pub dist_dir: String,
}

fn default_dev_path() -> String {
  "http://localhost:8080".to_string()
}

fn default_dist_path() -> String {
  "../dist".to_string()
}

impl Default for BuildConfig {
  fn default() -> Self {
    Self {
      dev_path: default_dev_path(),
      dist_dir: default_dist_path(),
    }
  }
}

/// The tauri.conf.json mapper.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  /// The Tauri configuration.
  #[serde(default)]
  pub tauri: TauriConfig,
  /// The build configuration.
  #[serde(default)]
  pub build: BuildConfig,
  /// The plugins config.
  #[serde(default)]
  pub plugins: PluginConfig,
}

/// The plugin configs holds a HashMap mapping a plugin name to its configuration object.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PluginConfig(HashMap<String, JsonValue>);

impl PluginConfig {
  /// Gets a plugin configuration.
  pub fn get<S: AsRef<str>>(&self, plugin_name: S) -> String {
    self
      .0
      .get(plugin_name.as_ref())
      .map(|config| config.to_string())
      .unwrap_or_else(|| "{}".to_string())
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
    let d_windows = default_window_config();
    // get default title
    let d_title = default_title();
    // get default bundle
    let d_bundle = BundleConfig::default();

    // create a tauri config.
    let tauri = TauriConfig {
      windows: vec![WindowConfig {
        label: "main".to_string(),
        url: WindowUrl::App,
        x: None,
        y: None,
        width: 800f64,
        height: 600f64,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
        resizable: true,
        title: String::from("Tauri App"),
        fullscreen: false,
        transparent: false,
        maximized: false,
        visible: true,
        decorations: true,
        always_on_top: false,
      }],
      bundle: BundleConfig {
        identifier: String::from(""),
      },
      cli: None,
    };

    // create a build config
    let build = BuildConfig {
      dev_path: String::from("http://localhost:8080"),
      dist_dir: String::from("../dist"),
    };

    // test the configs
    assert_eq!(t_config, tauri);
    assert_eq!(b_config, build);
    assert_eq!(d_bundle, tauri.bundle);
    assert_eq!(d_path, String::from("http://localhost:8080"));
    assert_eq!(d_title, tauri.windows[0].title);
    assert_eq!(d_windows, tauri.windows);
  }
}
