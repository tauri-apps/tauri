use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

const PLUGIN_NAME: &str = "sample";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.test";

#[cfg(target_os = "ios")]
mod ios {
  #[repr(C)]
  pub struct TauriPlugin {
    pub null: bool,
  }

  extern "C" {
    fn init_plugin() -> tauri::swift::SRObject<TauriPlugin>;
  }

  pub fn initialize_plugin() {
    std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_secs(5));
      log::info!("Initializing plugin...");
      let _plugin = unsafe { init_plugin() };
      log::info!("Initialized plugin!");
    });
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  #[allow(unused_mut)]
  let mut builder = Builder::new(PLUGIN_NAME);

  #[cfg(target_os = "ios")]
  ios::initialize_plugin();

  #[cfg(target_os = "android")]
  {
    use tauri::Manager;

    builder = builder.on_webview_ready(|window| {
      window
        .app_handle()
        .initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")
        .unwrap();
    });
  }
  builder.build()
}
