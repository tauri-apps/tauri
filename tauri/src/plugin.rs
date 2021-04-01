use crate::{
  api::config::PluginConfig, ApplicationExt, InvokeMessage, PageLoadPayload, WebviewManager,
};

use std::sync::{Arc, Mutex};

/// The plugin interface.
pub trait Plugin<A: ApplicationExt + 'static>: Send {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initialize the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, config: String) -> crate::Result<()> {
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
  fn created(&mut self, webview_manager: WebviewManager<A>) {}

  /// Callback invoked when the webview performs a navigation.
  #[allow(unused_variables)]
  fn on_page_load(&mut self, webview_manager: WebviewManager<A>, payload: PageLoadPayload) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  fn extend_api(&mut self, webview_manager: WebviewManager<A>, message: InvokeMessage<A>) {}
}

/// Plugin collection type.
pub type PluginStore<A> = Arc<Mutex<Vec<Box<dyn Plugin<A> + Send>>>>;

/// Registers a plugin.
pub fn register<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  plugin: impl Plugin<A> + Send + 'static,
) {
  let mut plugins = store.lock().unwrap();
  plugins.push(Box::new(plugin));
}

pub(crate) fn initialize<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  plugins_config: PluginConfig,
) -> crate::Result<()> {
  let mut plugins = store.lock().unwrap();
  for plugin in plugins.iter_mut() {
    let plugin_config = plugins_config.get(plugin.name());
    plugin.initialize(plugin_config)?;
  }

  Ok(())
}

pub(crate) fn initialization_script<A: ApplicationExt + 'static>(store: &PluginStore<A>) -> String {
  let mut plugins = store.lock().unwrap();
  let mut initialization_script = String::new();
  for plugin in plugins.iter_mut() {
    if let Some(plugin_initialization_script) = plugin.initialization_script() {
      initialization_script.push_str(&format!(
        "(function () {{ {} }})();",
        plugin_initialization_script
      ));
    }
  }
  initialization_script
}

pub(crate) fn created<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
) {
  let mut plugins = store.lock().unwrap();
  for plugin in plugins.iter_mut() {
    plugin.created(webview_manager.clone());
  }
}

pub(crate) fn on_page_load<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
  payload: PageLoadPayload,
) {
  let mut plugins = store.lock().unwrap();
  for plugin in plugins.iter_mut() {
    plugin.on_page_load(webview_manager.clone(), payload.clone());
  }
}

pub(crate) fn extend_api<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
  command: String,
  message: InvokeMessage<A>,
) {
  let mut plugins = store.lock().unwrap();
  let target_plugin_name = command
    .replace("plugin:", "")
    .split('|')
    .next()
    .unwrap()
    .to_string();
  for plugin in plugins.iter_mut() {
    if plugin.name() == target_plugin_name {
      plugin.extend_api(webview_manager.clone(), message);
      break;
    }
  }
}
