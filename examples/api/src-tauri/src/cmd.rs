// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::{
  command,
  ipc::{Channel, CommandScope},
  Runtime,
};

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
