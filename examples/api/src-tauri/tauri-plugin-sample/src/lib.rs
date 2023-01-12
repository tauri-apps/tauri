use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sample").build()
}
