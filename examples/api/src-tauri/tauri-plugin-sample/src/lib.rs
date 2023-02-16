use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;
pub mod models;

// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sample APIs.
pub trait SampleExt<R: Runtime> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<Result<PingResponse, String>>;
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sample")
    .setup(|_app, _api| {
      #[cfg(any(target_os = "android", target_os = "ios"))]
      mobile::init(_app, _api)?;

      Ok(())
    })
    .build()
}
