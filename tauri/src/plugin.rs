use crate::api::config::PluginConfig;
use crate::async_runtime::Mutex;
use crate::ApplicationDispatcherExt;

use futures::future::join_all;

use std::sync::Arc;

/// The plugin interface.
#[async_trait::async_trait]
pub trait Plugin<D: ApplicationDispatcherExt + 'static>: Send + Sync {
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
  async fn created(&mut self, dispatcher: D) {}

  /// Callback invoked when the webview is ready.
  #[allow(unused_variables)]
  async fn ready(&mut self, dispatcher: D) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  async fn extend_api(&mut self, dispatcher: D, payload: &str) -> crate::Result<()> {
    Err(crate::Error::UnknownApi)
  }
}

/// Plugin collection type.
pub type PluginStore<D> = Arc<Mutex<Vec<Box<dyn Plugin<D> + Sync + Send>>>>;

/// Registers a plugin.
pub async fn register<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  plugin: impl Plugin<D> + Sync + Send + 'static,
) {
  let mut plugins = store.lock().await;
  plugins.push(Box::new(plugin));
}

pub(crate) async fn initialize<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
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

pub(crate) async fn initialization_script<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
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

pub(crate) async fn created<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
) {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.created(dispatcher.clone()));
  }
  join_all(futures).await;
}

pub(crate) async fn ready<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
) {
  let mut plugins = store.lock().await;
  let mut futures = Vec::new();
  for plugin in plugins.iter_mut() {
    futures.push(plugin.ready(dispatcher.clone()));
  }
  join_all(futures).await;
}

pub(crate) async fn extend_api<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
  arg: &str,
) -> crate::Result<bool> {
  let mut plugins = store.lock().await;
  for ext in plugins.iter_mut() {
    match ext.extend_api(dispatcher.clone(), arg).await {
      Ok(_) => {
        return Ok(true);
      }
      Err(e) => match e {
        crate::Error::UnknownApi => {}
        _ => return Err(e),
      },
    }
  }
  Ok(false)
}
