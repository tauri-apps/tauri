#![cfg(path_api)]
use crate::ApplicationDispatcherExt;
use tauri_api::{path, path::BaseDirectory};

pub async fn resolve_path<D: ApplicationDispatcherExt>(
  dispatcher: &mut D,
  path: String,
  directory: Option<BaseDirectory>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    dispatcher,
    async move { path::resolve_path(path, directory).map_err(|e| e.into()) },
    callback,
    error,
  )
  .await
}
