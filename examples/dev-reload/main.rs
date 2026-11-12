// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(debug_assertions)]
#[tauri::command]
async fn reload_call(
  host: tauri::State<'_, std::sync::Arc<std::sync::Mutex<tauri::dev_reload::ReloadHost>>>,
  command: String,
) -> Result<serde_json::Value, String> {
  let host = host.inner().clone();
  tauri::async_runtime::spawn_blocking(move || {
    host
      .lock()
      .map_err(|e| e.to_string())?
      .call(&command, serde_json::Value::Null)
  })
  .await
  .map_err(|e| e.to_string())?
}

#[cfg(debug_assertions)]
#[tauri::command]
fn smoke_enabled() -> bool {
  std::env::var_os("TAURI_RELOAD_SMOKE").is_some()
}

#[cfg(debug_assertions)]
#[tauri::command]
fn smoke_report(app: tauri::AppHandle, phase: String) {
  if std::env::var_os("TAURI_RELOAD_SMOKE").is_some() {
    println!("RELOAD_SMOKE:{phase}");
    if phase == "passed" {
      app.exit(0);
    }
    if phase == "failed" {
      app.exit(1);
    }
  }
}

#[cfg(debug_assertions)]
fn main() {
  let manifest = std::env::var_os("TAURI_RELOAD_MANIFEST")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| {
      std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/dev-reload/backend/.reload/current")
    });
  tauri::Builder::default()
    .manage(std::sync::Arc::new(std::sync::Mutex::new(
      tauri::dev_reload::ReloadHost::new(
        manifest,
        serde_json::json!({"count": 0}),
        std::time::Duration::from_secs(3),
      ),
    )))
    .invoke_handler(tauri::generate_handler![
      reload_call,
      smoke_report,
      smoke_enabled
    ])
    .run(tauri::generate_context!(
      "../../examples/dev-reload/tauri.conf.json"
    ))
    .expect("failed to run reload example");
}

#[cfg(not(debug_assertions))]
fn main() {
  eprintln!("This example requires a debug build.");
}
