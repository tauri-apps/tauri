use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("app-internals")
    .js_init_script("window.__APP__ = true".into())
    .js_init_script_on_all_frames("window.__FRAME__ = true".into())
    .build()
}
