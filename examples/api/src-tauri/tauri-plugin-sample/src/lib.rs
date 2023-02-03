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

  pub fn initialize_plugin<R: Runtime>(window: Option<tauri::Window<R>>) {
    if let Some(window) = window {
      window.with_webview(|w| {
        unsafe { init_plugin(w.inner()) };
      });
    } else {
      unsafe { init_plugin(tauri::cocoa::base::nil) };
    }
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new(PLUGIN_NAME)
    .setup(|app| {
      #[cfg(target_os = "android")]
      app.initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")?;
      #[cfg(target_os = "ios")]
      ios::initialize_plugin(Option::<tauri::Window<R>>::None);
      Ok(())
    })
    .build()
}
