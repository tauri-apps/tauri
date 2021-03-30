use crate::{
  api::config::PluginConfig, runtime::Dispatch, InvokeMessage, PageLoadPayload, Tag, Window,
};
use serde_json::Value as JsonValue;
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

/// The plugin interface.
pub trait Plugin<E: Tag, L: Tag, D: Dispatch>: Send {
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
  fn created(&mut self, window: Window<E, L, D>) {}

  /// Callback invoked when the webview performs a navigation.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, window: Window<E, L, D>, payload: PageLoadPayload) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  fn extend_api(&mut self, message: InvokeMessage<E, L, D>) {}
}

/// Plugin collection type.
pub struct PluginStore<E: Tag, L: Tag, D: Dispatch> {
  store: Arc<Mutex<HashMap<&'static str, Box<dyn Plugin<E, L, D>>>>>,
}

impl<E: Tag, L: Tag, D: Dispatch> Clone for PluginStore<E, L, D> {
  fn clone(&self) -> Self {
    PluginStore {
      store: self.store.clone(),
    }
  }
}

impl<E: Tag, L: Tag, D: Dispatch> Default for PluginStore<E, L, D> {
  fn default() -> Self {
    Self {
      store: Arc::new(Mutex::default()),
    }
  }
}

impl<E: Tag, L: Tag, D: Dispatch> PluginStore<E, L, D> {
  /// Adds a plugin to the store.
  ///
  /// Returns `true` if a plugin with the same name is already in the store.
  pub fn register<P: Plugin<E, L, D> + 'static>(&self, plugin: P) -> bool {
    self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .insert(plugin.name(), Box::new(plugin))
      .is_some()
  }

  /// Initializes all plugins in the store.
  pub(crate) fn initialize(&self, config: &PluginConfig) -> crate::Result<()> {
    self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .values_mut()
      .try_for_each(|plugin| {
        plugin.initialize(config.0.get(plugin.name()).cloned().unwrap_or_default())
      })
  }

  /// Generates an initialization script from all plugins in the store.
  pub(crate) fn initialization_script(&self) -> String {
    self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .values()
      .filter_map(|p| p.initialization_script())
      .fold(String::new(), |acc, script| {
        format!("{}\n(function () {{ {} }})();", acc, script)
      })
  }

  /// Runs the created hook for all plugins in the store.
  pub(crate) fn created(&self, window: Window<E, L, D>) {
    self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .values_mut()
      .for_each(|plugin| plugin.created(window.clone()))
  }

  /// Runs the on_page_load hook for all plugins in the store.
  pub(crate) fn on_page_load(&self, window: Window<E, L, D>, payload: PageLoadPayload) {
    self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .values_mut()
      .for_each(|plugin| plugin.on_page_load(window.clone(), payload.clone()))
  }

  pub(crate) fn extend_api(&self, command: String, message: InvokeMessage<E, L, D>) {
    let target = command
      .replace("plugin:", "")
      .split('|')
      .next()
      .expect("target plugin name empty")
      .to_string();

    if let Some(plugin) = self
      .store
      .lock()
      .expect("poisoned plugin store mutex")
      .get_mut(target.as_str())
    {
      plugin.extend_api(message);
    }
  }
}
