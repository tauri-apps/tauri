#![cfg(path_api)]
use tauri_api::path;
use tauri_api::path::BaseDirectory;
use webview_official::WebviewMut;

pub async fn resolve_path(
  webview: &mut WebviewMut,
  path: String,
  directory: Option<BaseDirectory>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    async move { path::resolve_path(path, directory) },
    callback,
    error,
  )
  .await
}
