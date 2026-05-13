// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::{
  command,
  ipc::{Channel, CommandScope},
  Runtime,
  Manager, WebviewUrl,
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
  log::info!("log_operation");

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
pub fn perform_request(_endpoint: String, _body: RequestBody) -> ApiResponse {
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

#[command]
pub fn test_eval<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> tauri::Result<()> {
  log::info!("test_eval called");

  if let Some(window) = app.get_webview_window("main") {

    // 执行一个最简单的脚本：修改页面标题
    window.eval(r#"document.title = "✅ Eval Success! (From Rust)""#)?;

    // 执行带回调的脚本，获取当前时间
    window.eval_with_callback(
      r#"new Date().toLocaleString()"#,
      move |time_str| {
        log::info!("Current time from JS: {}", time_str);
      }
    )?;

    // 在页面上显示一个提示
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
pub fn test_navigate<R: tauri::Runtime>(window: tauri::WebviewWindow<R>, url: String) -> tauri::Result<()> {
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

  // 创建独立的数据目录
  let mut data_dir = app.path().app_data_dir()?;
  data_dir.push(format!("webview_data_{}", data_suffix));

  log::info!("Data directory: {:?}", data_dir);

  // 创建带有独立数据目录的窗口
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

  // 通过 Rust 端的 eval 获取 User-Agent 并打印日志
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

          // Start a counter to test
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
    //.decorations(false) // only desktop
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
