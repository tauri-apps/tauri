#![cfg(path_api)]
use tauri_api::path;
use tauri_api::path::BaseDirectory;
use webview_official::Webview;

pub fn resolve_path(
  webview: &mut Webview<'_>,
  path: String,
  directory: Option<BaseDirectory>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || path::resolve_path(path, directory),
    callback,
    error,
  )
}
