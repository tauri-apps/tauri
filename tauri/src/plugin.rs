use crate::{api::config::PluginConfig, ApplicationExt, PageLoadPayload, WebviewManager};

use serde_json::Value as JsonValue;

use std::sync::{Arc, Mutex};

/// The plugin interface.
#[async_trait::async_trait]
pub trait Plugin<A: ApplicationExt + 'static>: Send + Sync {
  /// The plugin name. Used as key on the plugin config object.
  fn name(&self) -> &'static str;

  /// Initialize the plugin.
  #[allow(unused_variables)]
  async fn initialize(&mut self, config: String) -> crate::Result<()> {
    Ok(())
  }

  /// The JS script to evaluate on webview initialization.
  /// The script is wrapped into its own context with `(function () { /* your script here */ })();`,
  /// so global variables must be assigned to `window` instead of implicity declared.
  ///
  /// It's guaranteed that this script is executed before the page is loaded.
  async fn initialization_script(&self) -> Option<String> {
    None
  }

  /// Callback invoked when the webview is created.
  #[allow(unused_variables)]
  async fn created(&mut self, webview_manager: WebviewManager<A>) {}

  /// Callback invoked when the webview performs a navigation.
  #[allow(unused_variables)]
  async fn on_page_load(&mut self, webview_manager: WebviewManager<A>, payload: PageLoadPayload) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  async fn extend_api(
    &mut self,
    webview_manager: WebviewManager<A>,
    command: String,
    payload: &JsonValue,
  ) -> crate::Result<JsonValue> {
    Err(crate::Error::UnknownApi(None))
  }
}

/// Plugin collection type.
pub type PluginStore<A> = Arc<Mutex<Vec<Box<dyn Plugin<A> + Sync + Send>>>>;

/// Registers a plugin.
pub fn register<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  plugin: impl Plugin<A> + Sync + Send + 'static,
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
    crate::async_runtime::block_on(plugin.initialize(plugin_config))?;
  }

  Ok(())
}

pub(crate) fn initialization_script<A: ApplicationExt + 'static>(store: &PluginStore<A>) -> String {
  let mut plugins = store.lock().unwrap();
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.initialization_script());
  }

  let mut initialization_script = String::new();
  for res in futures {
    if let Some(plugin_initialization_script) = crate::async_runtime::block_on(res) {
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
    crate::async_runtime::block_on(plugin.created(webview_manager.clone()));
  }
}

pub(crate) fn on_page_load<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
  payload: PageLoadPayload,
) {
  let mut plugins = store.lock().unwrap();
  for plugin in plugins.iter_mut() {
    crate::async_runtime::block_on(plugin.on_page_load(webview_manager.clone(), payload.clone()));
  }
}

pub(crate) fn extend_api<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
  command: String,
  arg: &JsonValue,
) -> crate::Result<Option<JsonValue>> {
  let mut plugins = store.lock().unwrap();
  for ext in plugins.iter_mut() {
    match crate::async_runtime::block_on(ext.extend_api(
      webview_manager.clone(),
      command.clone(),
      arg,
    )) {
      Ok(value) => {
        return Ok(Some(value));
      }
      Err(e) => match e {
        crate::Error::UnknownApi(_) => {}
        _ => return Err(e),
      },
    }
  }
  Ok(None)
}
