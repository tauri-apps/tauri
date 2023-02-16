use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::*;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sample")
    .setup(|_app, _api| {
      #[cfg(any(target_os = "android", target_os = "ios"))]
      mobile::init(_app, _api)?;

      Ok(())
    })
    .build()
}
