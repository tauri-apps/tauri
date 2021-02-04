use crate::async_runtime::Mutex;

use crate::WebviewMut;
use once_cell::sync::Lazy;

use std::sync::Arc;

/// The plugin interface.
#[async_trait::async_trait]
pub trait Plugin<W: WebviewMut + 'static>: Sync {
  /// The JS script to evaluate on init.
  async fn init_script(&self) -> Option<String> {
    None
  }
  /// Callback invoked when the webview is created.
  #[allow(unused_variables)]
  async fn created(&self, webview: W) {}

  /// Callback invoked when the webview is ready.
  #[allow(unused_variables)]
  async fn ready(&self, webview: W) {}

  /// Add invoke_handler API extension commands.
  #[allow(unused_variables)]
  async fn extend_api(&self, webview: W, payload: &str) -> Result<bool, String> {
    Err("unknown variant".to_string())
  }
}

/* type PluginStore<W: WebviewMut> = Arc<Mutex<Vec<Box<dyn Plugin<W> + Sync + Send>>>>;

fn plugins<W: WebviewMut>() -> &'static PluginStore<W> {
  static PLUGINS: Lazy<PluginStore<W>> = Lazy::new(Default::default);
  &PLUGINS
}

/// Registers a plugin.
pub async fn register<W: WebviewMut + 'static>(plugin: impl Plugin<W> + Sync + Send + 'static) {
  let mut plugins = plugins().lock().await;
  plugins.push(Box::new(plugin));
}

pub(crate) async fn init_script() -> String {
  let mut init = String::new();

  let plugins = plugins().lock().await;
  for plugin in plugins.iter() {
    if let Some(init_script) = plugin.init_script().await {
      init.push_str(&format!("(function () {{ {} }})();", init_script));
    }
  }

  init
}

pub(crate) async fn created<W: WebviewMut>(webview: &mut W) {
  let plugins = plugins().lock().await;
  for plugin in plugins.iter() {
    plugin.created(webview.clone()).await;
  }
}

pub(crate) async fn ready<W: WebviewMut>(webview: &mut W) {
  let plugins = plugins().lock().await;
  for plugin in plugins.iter() {
    plugin.ready(webview.clone()).await;
  }
}

pub(crate) async fn extend_api<W: WebviewMut>(webview: &mut W, arg: &str) -> Result<bool, String> {
  let plugins = plugins().lock().await;
  for ext in plugins.iter() {
    match ext.extend_api(webview.clone(), arg).await {
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
}*/
