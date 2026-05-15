// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use tauri::{
  command,
  ipc::{Channel, CommandScope},
  Resource, ResourceId,
  Manager, Runtime, WebviewUrl, Emitter, Listener,
};

// A simple Counter resource that lives in Rust
struct Counter {
  value: AtomicU32,
}

impl Resource for Counter {
  fn name(&self) -> std::borrow::Cow<'_, str> {
    "Counter".into()
  }
}

#[command]
pub fn create_counter<R: Runtime>(app: tauri::AppHandle<R>) -> ResourceId {
  let counter = Counter {
    value: AtomicU32::new(0),
  };
  app.resources_table().add(counter)
}

#[command]
pub fn increment_counter<R: Runtime>(app: tauri::AppHandle<R>, rid: ResourceId) -> tauri::Result<u32> {
  let counter = app.resources_table().get::<Counter>(rid)?;
  let new_value = counter.value.fetch_add(1, Ordering::SeqCst) + 1;
  Ok(new_value)
}

#[command]
pub fn get_counter_value<R: Runtime>(app: tauri::AppHandle<R>, rid: ResourceId) -> tauri::Result<u32> {
  let counter = app.resources_table().get::<Counter>(rid)?;
  Ok(counter.value.load(Ordering::SeqCst))
}

// Event tracking for testing
#[derive(Default)]
pub struct EventTracker {
  pub window_events: Mutex<Vec<String>>,
  pub menu_events: Mutex<Vec<String>>,
}

#[command]
pub fn get_tracked_window_events<R: Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<Vec<String>> {
  let tracker = app.state::<EventTracker>();
  let events = tracker.window_events.lock().unwrap().clone();
  Ok(events)
}

#[command]
pub fn get_tracked_menu_events<R: Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<Vec<String>> {
  let tracker = app.state::<EventTracker>();
  let events = tracker.menu_events.lock().unwrap().clone();
  Ok(events)
}

#[command]
pub fn clear_tracked_events<R: Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  let tracker = app.state::<EventTracker>();
  tracker.window_events.lock().unwrap().clear();
  tracker.menu_events.lock().unwrap().clear();
  Ok(())
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[derive(Debug, Deserialize)]
pub struct LogScope {
  event: String,
}

#[command]
pub fn log_operation(
  event: String,
  payload: Option<String>,
  command_scope: CommandScope<LogScope>,
) -> Result<(), &'static str> {
  if command_scope.denies().iter().any(|s| s.event == event) {
    Err("denied")
  } else if !command_scope.allows().iter().any(|s| s.event == event) {
    Err("not allowed")
  } else {
    log::info!("{event} {payload:?}");
    Ok(())
  }
}

#[derive(Serialize)]
pub struct ApiResponse {
  message: String,
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> ApiResponse {
  println!("{endpoint} {body:?}");
  ApiResponse {
    message: "message response".into(),
  }
}

#[command]
pub fn echo(request: tauri::ipc::Request<'_>) -> tauri::ipc::Response {
  tauri::ipc::Response::new(request.body().clone())
}

#[command]
pub fn spam(channel: Channel<i32>) -> tauri::Result<()> {
  for i in 1..=1_000 {
    channel.send(i)?;
  }
  Ok(())
}

#[command]
pub fn write_test_report<R: Runtime>(
  #[allow(unused_variables)] app: tauri::AppHandle<R>,
  report: String,
) -> Result<(), String> {
  #[cfg(target_env = "ohos")]
  let dir = std::path::PathBuf::from("/data/storage/el2/base/cache");
  #[cfg(not(target_env = "ohos"))]
  let dir = {
    use tauri::Manager;
    app.path().app_cache_dir().map_err(|e| e.to_string())?
  };

  std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  let path = dir.join("test-report.json");
  std::fs::write(&path, &report).map_err(|e| e.to_string())?;
  Ok(())
}

static CONSOLE_LOG_BUFFER: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

#[command]
pub fn console_log<R: Runtime>(
  #[allow(unused_variables)] app: tauri::AppHandle<R>,
  level: String,
  message: String,
) -> Result<(), String> {
  let ts = chrono::Local::now().format("%H:%M:%S%.3f");
  let entry = format!("[{}] {} {}", ts, level, message);

  let mut buffer = CONSOLE_LOG_BUFFER.lock().map_err(|e| e.to_string())?;
  buffer.push(entry);

  if buffer.len() > 1000 {
    buffer.remove(0);
  }
  Ok(())
}

#[command]
pub fn flush_console_log<R: Runtime>(
  #[allow(unused_variables)] app: tauri::AppHandle<R>,
) -> Result<String, String> {
  #[cfg(target_env = "ohos")]
  let dir = std::path::PathBuf::from("/data/storage/el2/base/cache");
  #[cfg(not(target_env = "ohos"))]
  let dir = {
    use tauri::Manager;
    app.path().app_cache_dir().map_err(|e| e.to_string())?
  };

  std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  let path = dir.join("console-log.txt");

  let mut buffer = CONSOLE_LOG_BUFFER.lock().map_err(|e| e.to_string())?;
  if buffer.is_empty() {
    return Ok(path.to_string_lossy().to_string());
  }
  let new_content = buffer.join("\n");
  buffer.clear();

  let existing = if path.exists() {
    std::fs::read_to_string(&path).unwrap_or_default()
  } else {
    String::new()
  };

  let full_content = if existing.is_empty() {
    new_content
  } else {
    format!("{}\n{}", existing, new_content)
  };

  std::fs::write(&path, &full_content).map_err(|e| e.to_string())?;

  Ok(path.to_string_lossy().to_string())
}

#[command]
pub fn clear_console_log<R: Runtime>(
  #[allow(unused_variables)] app: tauri::AppHandle<R>,
) -> Result<String, String> {
  #[cfg(target_env = "ohos")]
  let dir = std::path::PathBuf::from("/data/storage/el2/base/cache");
  #[cfg(not(target_env = "ohos"))]
  let dir = {
    use tauri::Manager;
    app.path().app_cache_dir().map_err(|e| e.to_string())?
  };

  let mut buffer = CONSOLE_LOG_BUFFER.lock().map_err(|e| e.to_string())?;
  buffer.clear();

  std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  let path = dir.join("console-log.txt");
  std::fs::write(&path, "").map_err(|e| e.to_string())?;

  Ok(path.to_string_lossy().to_string())
}

#[command]
pub fn test_eval<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  log::info!("test_eval called");

  if let Some(window) = app.get_webview_window("main") {
    window.eval(r#"document.title = "✅ Eval Success! (From Rust)""#)?;
    window.eval_with_callback(r#"new Date().toLocaleString()"#, move |time_str| {
      log::info!("Current time from JS: {}", time_str);
    })?;
    window.eval(r#"
      const div = document.createElement('div');
      div.style.cssText = 'position:fixed;top:50px;right:20px;background:green;color:white;padding:15px;border-radius:5px;z-index:9999;';
      div.textContent = '✅ Eval from Rust!';
      document.body.appendChild(div);
      setTimeout(() => div.remove(), 3000);
    "#)?;
  }

  Ok(())
}

#[command]
pub fn test_navigate<R: tauri::Runtime>(
  window: tauri::WebviewWindow<R>,
  url: String,
) -> tauri::Result<()> {
  log::info!("test_navigate called with url: {}", url);
  match url.parse() {
    Ok(parsed_url) => {
      window.navigate(parsed_url)?;
    }
    Err(e) => {
      log::error!("Failed to parse URL: {}", e);
    }
  }
  Ok(())
}

#[command]
pub fn test_reload<R: tauri::Runtime>(window: tauri::WebviewWindow<R>) -> tauri::Result<()> {
  log::info!("test_reload called");
  window.reload()?;
  Ok(())
}

#[command]
pub fn create_isolated_window<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  window_id: String,
  data_suffix: String,
) -> tauri::Result<()> {
  log::info!("Creating isolated window: {}", window_id);

  let mut data_dir = app.path().app_data_dir()?;
  data_dir.push(format!("webview_data_{}", data_suffix));

  log::info!("Data directory: {:?}", data_dir);

  tauri::WebviewWindowBuilder::new(&app, window_id, WebviewUrl::default())
    .title(format!("Isolated Window: {}", data_suffix))
    .data_directory(data_dir)
    .inner_size(800.0, 600.0)
    .build()?;

  Ok(())
}

#[command]
pub fn dummy_command() -> tauri::Result<()> {
  Ok(())
}

#[command]
pub fn create_window_with_custom_ua<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  window_id: String,
  user_agent: String,
) -> tauri::Result<()> {
  log::info!("Creating window with custom User-Agent: {}", user_agent);

  let window = tauri::WebviewWindowBuilder::new(&app, window_id, WebviewUrl::default())
    .title("Window with Custom User-Agent")
    .user_agent(&user_agent)
    .inner_size(800.0, 600.0)
    .build()?;

  window.eval_with_callback("navigator.userAgent", move |ua| {
    log::info!("Window User-Agent (from Rust): {}", ua);
  })?;

  Ok(())
}

#[command]
pub fn create_window_no_throttle<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  window_id: String,
) -> tauri::Result<()> {
  log::info!("Creating window with background throttling disabled");

  use tauri::utils::config::BackgroundThrottlingPolicy;

  let _window = tauri::WebviewWindowBuilder::new(&app, window_id, WebviewUrl::default())
    .title("Window with No Background Throttling")
    .background_throttling(BackgroundThrottlingPolicy::Disabled)
    .inner_size(800.0, 600.0)
    .initialization_script(
      r#"
        document.addEventListener('DOMContentLoaded', () => {
          const div = document.createElement('div');
          div.style.padding = '20px';
          div.innerHTML = '<h2>No Background Throttling Test</h2><p>Background timers should continue running even when window is hidden/minimized.</p><p><strong>Note:</strong> Only supported on macOS 14.0+ and iOS 17.0+</p>';
          document.body.appendChild(div);

          let count = 0;
          const counterDiv = document.createElement('div');
          counterDiv.style.padding = '20px';
          counterDiv.style.background = '#f0f0f0';
          counterDiv.style.marginTop = '20px';
          counterDiv.innerHTML = '<p>Timer (updates every second): <strong id="counter">0</strong></p>';
          document.body.appendChild(counterDiv);

          setInterval(() => {
            count++;
            document.getElementById('counter').textContent = count;
          }, 1000);
        });
      "#
    )
    .build()?;

  Ok(())
}

#[command]
pub fn create_transparent_window<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  window_id: String,
) -> tauri::Result<()> {
  log::info!("Creating transparent borderless window: {}", window_id);

  let _window = tauri::WebviewWindowBuilder::new(&app, window_id, WebviewUrl::default())
    .title("Transparent Window")
    .transparent(true)
    .inner_size(600.0, 400.0)
    .initialization_script(
      r#"
        document.addEventListener('DOMContentLoaded', () => {
          document.body.style.background = 'transparent';
          document.body.style.margin = '0';
          document.body.style.padding = '20px';
          document.body.style.display = 'flex';
          document.body.style.flexDirection = 'column';
          document.body.style.alignItems = 'center';
          document.body.style.justifyContent = 'center';
          document.body.style.fontFamily = 'system-ui, sans-serif';

          const div = document.createElement('div');
          div.style.background = 'rgba(0, 0, 0, 0.7)';
          div.style.color = 'white';
          div.style.padding = '30px';
          div.style.borderRadius = '15px';
          div.style.backdropFilter = 'blur(10px)';
          div.style.textAlign = 'center';
          div.innerHTML = '<h2>🪟 Transparent Borderless Window</h2><p>This window has transparent background and no title bar.</p><p style="font-size: 12px; opacity: 0.7; margin-top: 20px;">Close this window by pressing Ctrl+W or Cmd+W</p>';
          document.body.appendChild(div);
        });
      "#
    )
    .build()?;

  Ok(())
}

/// Test command for app_handle.emit
#[command]
pub fn emit_test_event<R: Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  app.emit("test-emit-event", "hello from rust")
}

/// Test command for app_handle.listen
#[command]
pub fn setup_app_listener<R: Runtime + 'static>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  let app_clone = app.clone();
  app.listen("app-listen-test", move |_event| {
    log::info!("Received app-listen-test via app.listen");
    let _ = app_clone.emit("app-listen-response", "heard you");
  });
  Ok(())
}

/// Test command for tauri::async_runtime::spawn
#[command]
pub fn test_async_spawn<R: Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  tauri::async_runtime::spawn(async move {
    // Simulate some async work
    let _ = app.emit("spawn-completed", "async done");
  });
  Ok(())
}
