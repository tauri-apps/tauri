#![cfg(path_api)]
use crate::WebviewMut;
use tauri_api::path;
use tauri_api::path::BaseDirectory;

pub async fn resolve_path<W: WebviewMut>(
  webview: &mut W,
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
