// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri plugin extension to expand Tauri functionality.

use crate::{
  runtime::Runtime, utils::config::PluginConfig, AppHandle, Invoke, InvokeHandler, OnPageLoad,
  PageLoadPayload, RunEvent, Window,
};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use tauri_macros::default_runtime;

use std::{collections::HashMap, fmt};

/// The result type of Tauri plugin module.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The plugin interface.
pub trait Plugin<R: Runtime>: Send {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initializes the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, app: &AppHandle<R>, config: JsonValue) -> Result<()> {
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

  /// Callback invoked when the webview performs a navigation to a page.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {}

  /// Callback invoked when the event loop receives a new event.
  #[allow(unused_variables)]
  fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {}

  /// Extend commands to [`crate::Builder::invoke_handler`].
  #[allow(unused_variables)]
  fn extend_api(&mut self, invoke: Invoke<R>) {}
}

type SetupHook<R> = dyn Fn(&AppHandle<R>) -> Result<()> + Send + Sync;
type SetupWithConfigHook<R, T> = dyn Fn(&AppHandle<R>, T) -> Result<()> + Send + Sync;
type OnWebviewReady<R> = dyn Fn(Window<R>) + Send + Sync;
type OnEvent<R> = dyn Fn(&AppHandle<R>, &RunEvent) + Send + Sync;

/// Builds a [`TauriPlugin`].
pub struct Builder<R: Runtime, C: DeserializeOwned = ()> {
  name: &'static str,
  invoke_handler: Box<InvokeHandler<R>>,
  setup: Box<SetupHook<R>>,
  setup_with_config: Option<Box<SetupWithConfigHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
}

impl<R: Runtime, C: DeserializeOwned> Builder<R, C> {
  /// Creates a new Plugin builder.
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      setup: Box::new(|_| Ok(())),
      setup_with_config: None,
      js_init_script: None,
      invoke_handler: Box::new(|_| ()),
      on_page_load: Box::new(|_, _| ()),
      on_webview_ready: Box::new(|_| ()),
      on_event: Box::new(|_, _| ()),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(Invoke<R>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// The JS script to evaluate on webview initialization.
  /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
  /// so global variables must be assigned to `window` instead of implicity declared.
  ///
  /// It's guaranteed that this script is executed before the page is loaded.
  pub fn js_init_script(mut self, js_init_script: String) -> Self {
    self.js_init_script = Some(js_init_script);
    self
  }

  /// Define a closure that can setup plugin specific state.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&AppHandle<R>) -> Result<()> + Send + Sync + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Define a closure that can setup the plugin accepting a configuration object set on `tauri.conf.json > plugins > yourPluginName`.
  pub fn setup_with_config<F>(mut self, setup_with_config: F) -> Self
  where
    F: Fn(&AppHandle<R>, C) -> Result<()> + Send + Sync + 'static,
  {
    self.setup_with_config.replace(Box::new(setup_with_config));
    self
  }

  /// Callback invoked when the webview performs a navigation to a page.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<R>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Callback invoked when the webview is created.
  pub fn on_webview_ready<F>(mut self, on_webview_ready: F) -> Self
  where
    F: Fn(Window<R>) + Send + Sync + 'static,
  {
    self.on_webview_ready = Box::new(on_webview_ready);
    self
  }

  /// Callback invoked when the event loop receives a new event.
  pub fn on_event<F>(mut self, on_event: F) -> Self
  where
    F: Fn(&AppHandle<R>, &RunEvent) + Send + Sync + 'static,
  {
    self.on_event = Box::new(on_event);
    self
  }

  /// Builds the [TauriPlugin].
  pub fn build(self) -> TauriPlugin<R, C> {
    TauriPlugin {
      name: self.name,
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      setup_with_config: self.setup_with_config,
      js_init_script: self.js_init_script,
      on_page_load: self.on_page_load,
      on_webview_ready: self.on_webview_ready,
      on_event: self.on_event,
    }
  }
}

/// Plugin struct that is returned by the [`PluginBuilder`]. Should only be constructed through the builder.
pub struct TauriPlugin<R: Runtime, C: DeserializeOwned = ()> {
  name: &'static str,
  invoke_handler: Box<InvokeHandler<R>>,
  setup: Box<SetupHook<R>>,
  setup_with_config: Option<Box<SetupWithConfigHook<R, C>>>,
  js_init_script: Option<String>,
  on_page_load: Box<OnPageLoad<R>>,
  on_webview_ready: Box<OnWebviewReady<R>>,
  on_event: Box<OnEvent<R>>,
}

impl<R: Runtime, C: DeserializeOwned> Plugin<R> for TauriPlugin<R, C> {
  fn name(&self) -> &'static str {
    self.name
  }

  fn initialize(&mut self, app: &AppHandle<R>, config: JsonValue) -> Result<()> {
    (self.setup)(app)?;
    if let Some(s) = &self.setup_with_config {
      (s)(app, serde_json::from_value(config)?)?;
    }
    Ok(())
  }

  fn initialization_script(&self) -> Option<String> {
    self.js_init_script.clone()
  }

  fn created(&mut self, window: Window<R>) {
    (self.on_webview_ready)(window)
  }

  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {
    (self.on_page_load)(window, payload)
  }

  fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    (self.on_event)(app, event)
  }

  fn extend_api(&mut self, invoke: Invoke<R>) {
    (self.invoke_handler)(invoke)
  }
}

/// Plugin collection type.
#[default_runtime(crate::Wry, wry)]
pub(crate) struct PluginStore<R: Runtime> {
  store: HashMap<&'static str, Box<dyn Plugin<R>>>,
}

impl<R: Runtime> fmt::Debug for PluginStore<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("PluginStore")
      .field("plugins", &self.store.keys())
      .finish()
  }
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
  pub(crate) fn initialize(
    &mut self,
    app: &AppHandle<R>,
    config: &PluginConfig,
  ) -> crate::Result<()> {
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

  /// Runs the on_event hook for all plugins in the store.
  pub(crate) fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    self
      .store
      .values_mut()
      .for_each(|plugin| plugin.on_event(app, event))
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
