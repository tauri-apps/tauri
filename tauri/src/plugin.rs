use crate::async_runtime::Mutex;

use crate::ApplicationDispatcherExt;

use std::sync::Arc;

/// The plugin interface.
#[async_trait::async_trait]
pub trait Plugin<D: ApplicationDispatcherExt + 'static>: Sync {
  /// The JS script to evaluate on init.
  async fn init_script(&self) -> Option<String> {
    None
  }
  /// Callback invoked when the webview is created.
  #[allow(unused_variables)]
  async fn created(&self, dispatcher: D) {}

  /// Callback invoked when the webview is ready.
  #[allow(unused_variables)]
  async fn ready(&self, dispatcher: D) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  async fn extend_api(&self, dispatcher: D, payload: &str) -> Result<bool, String> {
    Err("unknown variant".to_string())
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

pub(crate) async fn init_script<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
) -> String {
  let mut init = String::new();

  let plugins = store.lock().await;
  for plugin in plugins.iter() {
    if let Some(init_script) = plugin.init_script().await {
      init.push_str(&format!("(function () {{ {} }})();", init_script));
    }
  }

  init
}

pub(crate) async fn created<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
) {
  let plugins = store.lock().await;
  for plugin in plugins.iter() {
    plugin.created(dispatcher.clone()).await;
  }
}

pub(crate) async fn ready<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
) {
  let plugins = store.lock().await;
  for plugin in plugins.iter() {
    plugin.ready(dispatcher.clone()).await;
  }
}

pub(crate) async fn extend_api<D: ApplicationDispatcherExt + 'static>(
  store: &PluginStore<D>,
  dispatcher: &mut D,
  arg: &str,
) -> Result<bool, String> {
  let plugins = store.lock().await;
  for ext in plugins.iter() {
    match ext.extend_api(dispatcher.clone(), arg).await {
      Ok(handled) => {
        if handled {
          return Ok(true);
        }
      }
      Err(e) => {
        if !e.contains("unknown variant") {
          return Err(e);
        }
      }
    }
  }
  Ok(false)
}
