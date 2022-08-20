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

#[tauri::mobile_entry_point]
fn main() {
  super::AppBuilder::new()
    .setup(|app| {
      init_logging(&app.package_info().name);
      Ok(())
    })
    .run();
}
