// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
  api::config::PluginConfig,
  hooks::{InvokeMessage, PageLoadPayload},
  runtime::{window::Window, Params},
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// The plugin interface.
pub trait Plugin<M: Params>: Send {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initialize the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, config: JsonValue) -> crate::Result<()> {
    Ok(())
  }

  /// The JS script to evaluate on webview initialization.
  /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
  /// so global variables must be assigned to `window` instead of implicity declared.
  ///
  /// It's guaranteed that this script is executed before the page is loaded.
  fn initialization_script(&self) -> Option<String> {
    None
  }

  /// Callback invoked when the webview is created.
  #[allow(unused_variables)]
  fn created(&mut self, window: Window<M>) {}

  /// Callback invoked when the webview performs a navigation.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, window: Window<M>, payload: PageLoadPayload) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  fn extend_api(&mut self, message: InvokeMessage<M>) {}
}

/// Plugin collection type.
pub struct PluginStore<M: Params> {
  store: HashMap<&'static str, Box<dyn Plugin<M>>>,
}

impl<M: Params> Default for PluginStore<M> {
  fn default() -> Self {
    Self {
      store: HashMap::new(),
    }
  }
}

impl<M: Params> PluginStore<M> {
  /// Adds a plugin to the store.
  ///
  /// Returns `true` if a plugin with the same name is already in the store.
  pub fn register<P: Plugin<M> + 'static>(&mut self, plugin: P) -> bool {
    self.store.insert(plugin.name(), Box::new(plugin)).is_some()
  }

  /// Initializes all plugins in the store.
  pub(crate) fn initialize(&mut self, config: &PluginConfig) -> crate::Result<()> {
    self.store.values_mut().try_for_each(|plugin| {
      plugin.initialize(config.0.get(plugin.name()).cloned().unwrap_or_default())
    })
  }

  /// Generates an initialization script from all plugins in the store.
  pub(crate) fn initialization_script(&self) -> String {
    self
      .store
      .values()
      .filter_map(|p| p.initialization_script())
      .fold(String::new(), |acc, script| {
        format!("{}\n(function () {{ {} }})();", acc, script)
      })
  }

  /// Runs the created hook for all plugins in the store.
  pub(crate) fn created(&mut self, window: Window<M>) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.created(window.clone()))
  }

  /// Runs the on_page_load hook for all plugins in the store.
  pub(crate) fn on_page_load(&mut self, window: Window<M>, payload: PageLoadPayload) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_page_load(window.clone(), payload.clone()))
  }

  pub(crate) fn extend_api(&mut self, command: String, message: InvokeMessage<M>) {
    let target = command
      .replace("plugin:", "")
      .split('|')
      .next()
      .expect("target plugin name empty")
      .to_string();

    if let Some(plugin) = self.store.get_mut(target.as_str()) {
      plugin.extend_api(message);
    }
  }
}
