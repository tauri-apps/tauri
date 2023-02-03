use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

const PLUGIN_NAME: &str = "sample";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.test";

#[cfg(target_os = "ios")]
mod ios {
  use tauri::Runtime;

  extern "C" {
    fn init_plugin(webview: tauri::cocoa::base::id);
  }

  pub fn initialize_plugin<R: Runtime>(window: tauri::Window<R>) {
    window.with_webview(|w| {
      unsafe { init_plugin(w.inner()) };
    });
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  #[allow(unused_mut)]
  let mut builder = Builder::new(PLUGIN_NAME);

  #[cfg(target_os = "ios")]
  {
    builder = builder.on_webview_ready(|window| {
      ios::initialize_plugin(window);
    });
  }

  #[cfg(target_os = "android")]
  {
    builder = builder.setup(|app| {
      app.initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")?;
      Ok(())
    });
  }

  builder.build()
}
