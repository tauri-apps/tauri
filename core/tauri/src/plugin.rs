// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Extend Tauri functionality.

use crate::{api::config::PluginConfig, runtime::Runtime, App, Invoke, PageLoadPayload, Window};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use tauri_macros::default_runtime;

/// The plugin result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The plugin interface.
pub trait Plugin<R: Runtime>: Send {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initialize the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, app: &App<R>, config: JsonValue) -> Result<()> {
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
  fn created(&mut self, window: Window<R>) {}

  /// Callback invoked when the webview performs a navigation.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  fn extend_api(&mut self, invoke: Invoke<R>) {}
}

/// Plugin collection type.
#[default_runtime(crate::Wry, wry)]
pub(crate) struct PluginStore<R: Runtime> {
  store: HashMap<&'static str, Box<dyn Plugin<R>>>,
}

impl<R: Runtime> Default for PluginStore<R> {
  fn default() -> Self {
    Self {
      store: HashMap::new(),
    }
  }
}

impl<R: Runtime> PluginStore<R> {
  /// Adds a plugin to the store.
  ///
  /// Returns `true` if a plugin with the same name is already in the store.
  pub fn register<P: Plugin<R> + 'static>(&mut self, plugin: P) -> bool {
    self.store.insert(plugin.name(), Box::new(plugin)).is_some()
  }

  /// Initializes all plugins in the store.
  pub(crate) fn initialize(&mut self, app: &App<R>, config: &PluginConfig) -> crate::Result<()> {
    self.store.values_mut().try_for_each(|plugin| {
      plugin
        .initialize(
          app,
          config.0.get(plugin.name()).cloned().unwrap_or_default(),
        )
        .map_err(|e| crate::Error::PluginInitialization(plugin.name().to_string(), e.to_string()))
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
  pub(crate) fn created(&mut self, window: Window<R>) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.created(window.clone()))
  }

  /// Runs the on_page_load hook for all plugins in the store.
  pub(crate) fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_page_load(window.clone(), payload.clone()))
  }

  pub(crate) fn extend_api(&mut self, mut invoke: Invoke<R>) {
    let command = invoke.message.command.replace("plugin:", "");
    let mut tokens = command.split('|');
    // safe to unwrap: split always has a least one item
    let target = tokens.next().unwrap();

    if let Some(plugin) = self.store.get_mut(target) {
      invoke.message.command = tokens
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(String::new);
      plugin.extend_api(invoke);
    } else {
      invoke
        .resolver
        .reject(format!("plugin {} not found", target));
    }
  }
}
