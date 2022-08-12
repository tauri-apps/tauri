#[cfg(target_os = "android")]
use tauri_runtime_wry::wry::application::{android_fn, platform::android::ndk_glue::*};

#[cfg(target_os = "android")]
fn init_logging(app_name: &str) {
  android_logger::init_once(
    android_logger::Config::default()
      .with_min_level(log::Level::Trace)
      .with_tag(app_name),
  );
}

#[cfg(not(target_os = "android"))]
fn init_logging(_app_name: &str) {
  env_logger::init();
}

fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
    Ok(t) => t,
    Err(err) => {
      eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
      std::process::abort()
    }
  }
}

fn _start_app() {
  stop_unwind(main);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn start_app() {
  #[cfg(target_os = "android")]
  android_fn!(studio_tauri, api);
  _start_app()
}

fn main() {
  super::AppBuilder::new()
    .setup(|app| {
      init_logging(&app.package_info().name);
      Ok(())
    })
    .run();
}
