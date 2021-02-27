use crate::{api::config::PluginConfig, async_runtime::Mutex, ApplicationExt, WebviewManager};

use futures::future::join_all;
use serde_json::Value as JsonValue;

use std::sync::Arc;

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

  /// Callback invoked when the webview is ready.
  #[allow(unused_variables)]
  async fn ready(&mut self, webview_manager: WebviewManager<A>) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  async fn extend_api(
    &mut self,
    webview_manager: WebviewManager<A>,
    payload: &JsonValue,
  ) -> crate::Result<JsonValue> {
    Err(crate::Error::UnknownApi(None))
  }
}

/// Plugin collection type.
pub type PluginStore<A> = Arc<Mutex<Vec<Box<dyn Plugin<A> + Sync + Send>>>>;

/// Registers a plugin.
pub async fn register<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  plugin: impl Plugin<A> + Sync + Send + 'static,
) {
  let mut plugins = store.lock().await;
  plugins.push(Box::new(plugin));
}

pub(crate) async fn initialize<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  plugins_config: PluginConfig,
) -> crate::Result<()> {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    let plugin_config = plugins_config.get(plugin.name());
    futures.push(plugin.initialize(plugin_config));
  }

  for res in join_all(futures).await {
    res?;
  }

  Ok(())
}

pub(crate) async fn initialization_script<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
) -> String {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.initialization_script());
  }

  let mut initialization_script = String::new();
  for res in join_all(futures).await {
    if let Some(plugin_initialization_script) = res {
      initialization_script.push_str(&format!(
        "(function () {{ {} }})();",
        plugin_initialization_script
      ));
    }
  }
  initialization_script
}

pub(crate) async fn created<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
) {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.created(webview_manager.clone()));
  }
  join_all(futures).await;
}

pub(crate) async fn ready<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
) {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.ready(webview_manager.clone()));
  }
  join_all(futures).await;
}

pub(crate) async fn extend_api<A: ApplicationExt + 'static>(
  store: &PluginStore<A>,
  webview_manager: &crate::WebviewManager<A>,
  arg: &JsonValue,
) -> crate::Result<Option<JsonValue>> {
  let mut plugins = store.lock().await;
  for ext in plugins.iter_mut() {
    match ext.extend_api(webview_manager.clone(), arg).await {
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
