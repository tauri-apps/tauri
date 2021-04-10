// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;
use serde_json::Value as JsonValue;
use url::Url;

/// The window webview URL options.
#[derive(PartialEq, Debug, Clone, Deserialize)]
#[serde(untagged)]
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
      url: WindowUrl::default(),
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

/// The Updater configuration object.
#[derive(PartialEq, Deserialize, Debug, Clone)]
#[serde(tag = "updater", rename_all = "camelCase")]
pub struct UpdaterConfig {
  /// Whether the updater is active or not.
  #[serde(default)]
  pub active: bool,
  /// Display built-in dialog or use event system if disabled.
  #[serde(default = "default_updater_dialog")]
  pub dialog: bool,
  /// The updater endpoints.
  #[serde(default)]
  pub endpoints: Option<Vec<String>>,
  /// Optional pubkey.
  #[serde(default)]
  pub pubkey: Option<String>,
}

fn default_updater_dialog() -> bool {
  true
}

impl Default for UpdaterConfig {
  fn default() -> Self {
    Self {
      active: false,
      dialog: true,
      endpoints: None,
      pubkey: None,
    }
  }
}

/// A CLI argument definition
#[derive(PartialEq, Deserialize, Debug, Default, Clone)]
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
#[derive(PartialEq, Deserialize, Debug, Clone)]
#[serde(tag = "cli", rename_all = "camelCase")]
#[allow(missing_docs)] // TODO
pub struct CliConfig {
  pub description: Option<String>,
  pub long_description: Option<String>,
  pub before_help: Option<String>,
  pub after_help: Option<String>,
  pub args: Option<Vec<CliArg>>,
  pub subcommands: Option<HashMap<String, CliConfig>>,
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
  /// The updater configuration.
  #[serde(default)]
  pub updater: UpdaterConfig,
}

impl Default for TauriConfig {
  fn default() -> Self {
    Self {
      windows: default_window_config(),
      cli: None,
      bundle: BundleConfig::default(),
      updater: UpdaterConfig::default(),
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
  /// Whether we should inject the Tauri API on `window.__TAURI__` or not.
  #[serde(default)]
  pub with_global_tauri: bool,
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
      with_global_tauri: false,
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
pub struct PluginConfig(pub HashMap<String, JsonValue>);

/// Implement `ToTokens` for all config structs, allowing a literal `Config` to be built.
///
/// This allows for a build script to output the values in a `Config` to a `TokenStream`, which can
/// then be consumed by another crate. Useful for passing a config to both the build script and the
/// application using tauri while only parsing it once (in the build script).
#[cfg(feature = "build")]
mod build {
  use std::convert::identity;

  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  use super::*;

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
        ::tauri::api::config::$struct {
          $($field: #$field),+
        }
      });
    };
  }

  impl ToTokens for WindowUrl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let prefix = quote! { ::tauri::api::config::WindowUrl };

      tokens.append_all(match self {
        Self::App(path) => {
          let path = path.to_string_lossy().to_string();
          quote! { #prefix::App(::std::path::PathBuf::from(#path)) }
        }
        Self::External(url) => {
          let url = url.as_str();
          quote! { #prefix::External(#url.parse().unwrap()) }
        }
      })
    }
  }

  impl ToTokens for WindowConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let label = str_lit(&self.label);
      let url = &self.url;
      let x = opt_lit(self.x.as_ref());
      let y = opt_lit(self.y.as_ref());
      let width = self.width;
      let height = self.height;
      let min_width = opt_lit(self.min_width.as_ref());
      let min_height = opt_lit(self.min_height.as_ref());
      let max_width = opt_lit(self.max_width.as_ref());
      let max_height = opt_lit(self.min_height.as_ref());
      let resizable = self.resizable;
      let title = str_lit(&self.title);
      let fullscreen = self.fullscreen;
      let transparent = self.transparent;
      let maximized = self.maximized;
      let visible = self.visible;
      let decorations = self.decorations;
      let always_on_top = self.always_on_top;

      literal_struct!(
        tokens,
        WindowConfig,
        label,
        url,
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
        transparent,
        maximized,
        visible,
        decorations,
        always_on_top
      );
    }
  }

  impl ToTokens for CliArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let short = opt_lit(self.short.as_ref());
      let name = str_lit(&self.name);
      let description = opt_str_lit(self.description.as_ref());
      let long_description = opt_str_lit(self.long_description.as_ref());
      let takes_value = opt_lit(self.takes_value.as_ref());
      let multiple = opt_lit(self.multiple.as_ref());
      let multiple_occurrences = opt_lit(self.multiple_occurrences.as_ref());
      let number_of_values = opt_lit(self.number_of_values.as_ref());
      let possible_values = opt_vec_str_lit(self.possible_values.as_ref());
      let min_values = opt_lit(self.min_values.as_ref());
      let max_values = opt_lit(self.max_values.as_ref());
      let required = opt_lit(self.required.as_ref());
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

  impl ToTokens for BundleConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let identifier = str_lit(&self.identifier);

      literal_struct!(tokens, BundleConfig, identifier);
    }
  }

  impl ToTokens for BuildConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let dev_path = str_lit(&self.dev_path);
      let dist_dir = str_lit(&self.dist_dir);
      let with_global_tauri = self.with_global_tauri;

      literal_struct!(tokens, BuildConfig, dev_path, dist_dir, with_global_tauri);
    }
  }

  impl ToTokens for UpdaterConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let active = self.active;
      let dialog = self.dialog;
      let pubkey = opt_str_lit(self.pubkey.as_ref());
      let endpoints = opt_vec_str_lit(self.endpoints.as_ref());

      literal_struct!(tokens, UpdaterConfig, active, dialog, pubkey, endpoints);
    }
  }

  impl ToTokens for TauriConfig {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let windows = vec_lit(&self.windows, identity);
      let cli = opt_lit(self.cli.as_ref());
      let bundle = &self.bundle;
      let updater = &self.updater;

      literal_struct!(tokens, TauriConfig, windows, cli, bundle, updater);
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
      tokens.append_all(quote! { ::tauri::api::config::PluginConfig(#config) })
    }
  }

  impl ToTokens for Config {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let tauri = &self.tauri;
      let build = &self.build;
      let plugins = &self.plugins;

      literal_struct!(tokens, Config, tauri, build, plugins);
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
    let d_windows = default_window_config();
    // get default title
    let d_title = default_title();
    // get default bundle
    let d_bundle = BundleConfig::default();
    // get default updater
    let d_updater = UpdaterConfig::default();

    // create a tauri config.
    let tauri = TauriConfig {
      windows: vec![WindowConfig {
        label: "main".to_string(),
        url: WindowUrl::default(),
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
      updater: UpdaterConfig {
        active: false,
        dialog: true,
        pubkey: None,
        endpoints: None,
      },
    };

    // create a build config
    let build = BuildConfig {
      dev_path: String::from("http://localhost:8080"),
      dist_dir: String::from("../dist"),
      with_global_tauri: false,
    };

    // test the configs
    assert_eq!(t_config, tauri);
    assert_eq!(b_config, build);
    assert_eq!(d_bundle, tauri.bundle);
    assert_eq!(d_updater, tauri.updater);
    assert_eq!(d_path, String::from("http://localhost:8080"));
    assert_eq!(d_title, tauri.windows[0].title);
    assert_eq!(d_windows, tauri.windows);
  }
}
