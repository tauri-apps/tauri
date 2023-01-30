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

  #[repr(C)]
  pub struct TauriPlugin {
    pub null: bool,
  }

  extern "C" {
    fn init_plugin(webview: tauri::cocoa::base::id) -> tauri::swift::SRObject<TauriPlugin>;
  }

  pub fn initialize_plugin<R: Runtime>(window: tauri::Window<R>) {
    std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_secs(2));
      log::info!("With webview...");
      window.with_webview(|w| {
        log::info!("Initializing plugin...");
        let _plugin = unsafe { init_plugin(w.inner()) };
        log::info!("Initialized plugin!");
      });
    });
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  #[allow(unused_mut)]
  let mut builder = Builder::new(PLUGIN_NAME);

  #[cfg(any(target_os = "android", target_os = "ios"))]
  {
    builder = builder.on_webview_ready(|window| {
      #[cfg(target_os = "ios")]
      ios::initialize_plugin(window);

      #[cfg(target_os = "android")]
      {
        use tauri::Manager;

        window
          .app_handle()
          .initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")
          .unwrap();
      }
    });
  }
  builder.build()
}
