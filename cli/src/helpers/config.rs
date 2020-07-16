use once_cell::sync::OnceCell;
use serde::de::{Deserializer, Error as DeError, Visitor};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

static CONFIG: OnceCell<Config> = OnceCell::new();

/// The window configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "window", rename_all = "camelCase")]
pub struct WindowConfig {
  /// The window width.
  #[serde(default = "default_width")]
  pub width: i32,
  /// The window height.
  #[serde(default = "default_height")]
  pub height: i32,
  /// Whether the window is resizable or not.
  #[serde(default = "default_resizable")]
  pub resizable: bool,
  /// The window title.
  #[serde(default = "default_title")]
  pub title: String,
  /// Whether the window starts as fullscreen or not.
  #[serde(default)]
  pub fullscreen: bool,
}

fn default_width() -> i32 {
  800
}

fn default_height() -> i32 {
  600
}

fn default_resizable() -> bool {
  true
}

fn default_title() -> String {
  "Tauri App".to_string()
}

fn default_window() -> WindowConfig {
  WindowConfig {
    width: default_width(),
    height: default_height(),
    resizable: default_resizable(),
    title: default_title(),
    fullscreen: false,
  }
}

/// The embedded server port.
#[derive(PartialEq, Debug, Deserialize)]
pub enum Port {
  /// Port with a numeric value.
  Value(u16),
  /// Random port.
  Random,
}

/// The embeddedServer configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "embeddedServer", rename_all = "camelCase")]
pub struct EmbeddedServerConfig {
  /// Whether the embeddedServer is active or not
  pub active: bool,
  /// The embedded server host.
  #[serde(default = "default_host")]
  pub host: String,
  /// The embedded server port.
  /// If it's `random`, we'll generate one at runtime.
  #[serde(default = "default_port", deserialize_with = "port_deserializer")]
  pub port: Port,
}

fn default_host() -> String {
  "http://127.0.0.1".to_string()
}

fn port_deserializer<'de, D>(deserializer: D) -> Result<Port, D::Error>
where
  D: Deserializer<'de>,
{
  struct PortDeserializer;

  impl<'de> Visitor<'de> for PortDeserializer {
    type Value = Port;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      formatter.write_str("a port number or the 'random' string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: DeError,
    {
      if value != "random" {
        Err(DeError::custom(
          "expected a 'random' string or a port number",
        ))
      } else {
        Ok(Port::Random)
      }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
      E: DeError,
    {
      Ok(Port::Value(value as u16))
    }
  }

  deserializer.deserialize_any(PortDeserializer {})
}

fn default_port() -> Port {
  Port::Random
}

fn default_embedded_server() -> EmbeddedServerConfig {
  EmbeddedServerConfig {
    active: false,
    host: default_host(),
    port: default_port(),
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
  pub required_unless: Option<String>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless all these other arguments are present.
  pub required_unless_all: Option<Vec<String>>,
  /// Sets args that override this arg's required setting
  /// i.e. this arg will be required unless at least one of these other arguments are present.
  pub required_unless_one: Option<Vec<String>>,
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
  pub required_if: Option<Vec<String>>,
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

fn default_bundle() -> BundleConfig {
  BundleConfig {
    identifier: String::from(""),
  }
}

/// The Tauri configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  /// The window configuration.
  #[serde(default = "default_window")]
  pub window: WindowConfig,
  /// The embeddedServer configuration.
  #[serde(default = "default_embedded_server")]
  pub embedded_server: EmbeddedServerConfig,
  /// The CLI configuration.
  #[serde(default)]
  pub cli: Option<CliConfig>,
  /// The bundler configuration.
  #[serde(default = "default_bundle")]
  pub bundle: BundleConfig,
}

/// The Build configuration object.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  /// the devPath config.
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
  #[serde(default = "default_dist_dir")]
  pub dist_dir: String,
}

fn default_dev_path() -> String {
  "".to_string()
}

fn default_dist_dir() -> String {
  "../dist".to_string()
}

type JsonObject = HashMap<String, JsonValue>;

/// The tauri.conf.json mapper.
#[derive(PartialEq, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  /// The Tauri configuration.
  #[serde(default = "default_tauri")]
  pub tauri: TauriConfig,
  /// The build configuration.
  #[serde(default = "default_build")]
  pub build: BuildConfig,
  /// The plugins config.
  #[serde(default)]
  pub plugins: HashMap<String, JsonObject>,
}

fn default_tauri() -> TauriConfig {
  TauriConfig {
    window: default_window(),
    embedded_server: default_embedded_server(),
    cli: None,
    bundle: default_bundle(),
  }
}

fn default_build() -> BuildConfig {
  BuildConfig {
    dev_path: default_dev_path(),
    dist_dir: default_dist_dir(),
  }
}

/// Gets the static parsed config from `tauri.conf.json`.
pub fn get() -> crate::Result<&'static Config> {
  if let Some(config) = CONFIG.get() {
    return Ok(config);
  }

  let path = super::app_paths::tauri_dir().join("tauri.conf.json");
  let file = File::open(path)?;
  let buf = BufReader::new(file);
  let config = serde_json::from_reader(buf)?;

  CONFIG
    .set(config)
    .map_err(|_| anyhow::anyhow!("failed to set CONFIG"))?;

  let config = CONFIG.get().unwrap();
  Ok(config)
}
