// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::{
  command,
  ipc::{Channel, CommandScope},
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

/// Opens a window hosting two webviews, to try the multiwebview API (requires the `unstable` feature).
///
/// On Android the window is a task of its own; the first webview lives in the window's activity and the
/// second one in `EmbeddedWebviewActivity`, started in the window's task, which activity embedding split rules
/// can show side by side. `packages/api-e2e/scripts/prepare-android.mjs` adds that activity to the Android project.
#[command]
pub async fn spawn_multiwebview_window<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
) -> Result<String, String> {
  #[cfg(all(feature = "unstable", any(desktop, target_os = "android")))]
  {
    use std::sync::atomic::{AtomicU32, Ordering};
    use tauri::{LogicalPosition, LogicalSize, WebviewUrl, webview::WebviewBuilder};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let (width, height) = (1000., 700.);
    // `main-*` so the webviews get the run-app capability
    let label = format!(
      "main-multiwebview-{}",
      COUNTER.fetch_add(1, Ordering::Relaxed)
    );

    let window = tauri::window::WindowBuilder::new(&app, &label)
      .title("Multiwebview")
      .inner_size(width, height)
      .build()
      .map_err(|e| e.to_string())?;

    let left = WebviewBuilder::new(format!("{label}-left"), WebviewUrl::default()).auto_resize();
    window
      .add_child(
        left,
        LogicalPosition::new(0., 0.),
        LogicalSize::new(width / 2., height),
      )
      .map_err(|e| e.to_string())?;

    let right = WebviewBuilder::new(format!("{label}-right"), WebviewUrl::default()).auto_resize();
    #[cfg(target_os = "android")]
    let right = right.activity_name("EmbeddedWebviewActivity");
    window
      .add_child(
        right,
        LogicalPosition::new(width / 2., 0.),
        LogicalSize::new(width / 2., height),
      )
      .map_err(|e| e.to_string())?;

    Ok(label)
  }

  #[cfg(not(all(feature = "unstable", any(desktop, target_os = "android"))))]
  {
    let _ = app;
    Err("the multiwebview API requires the `unstable` feature on desktop or Android".into())
  }
}
