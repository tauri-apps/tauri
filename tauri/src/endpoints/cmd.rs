use crate::api::path::BaseDirectory;
use serde::Deserialize;
use tauri_api::http::HttpRequestOptions;

#[derive(Deserialize)]
pub struct DirOperationOptions {
  #[serde(default)]
  pub recursive: bool,
  pub dir: Option<BaseDirectory>,
}

#[derive(Deserialize)]
pub struct FileOperationOptions {
  pub dir: Option<BaseDirectory>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
  pub filter: Option<String>,
  #[serde(default)]
  pub multiple: bool,
  #[serde(default)]
  pub directory: bool,
  pub default_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogOptions {
  pub filter: Option<String>,
  pub default_path: Option<String>,
}

#[derive(Deserialize)]
pub struct NotificationOptions {
  pub title: Option<String>,
  pub body: String,
  pub icon: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  Init {},
  ReadTextFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  ReadBinaryFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  WriteFile {
    file: String,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  ReadDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  CopyFile {
    source: String,
    destination: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  CreateDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  RemoveDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  RemoveFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[serde(rename_all = "camelCase")]
  RenameFile {
    old_path: String,
    new_path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  SetTitle {
    title: String,
  },
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
  Open {
    uri: String,
  },
  ValidateSalt {
    salt: String,
    callback: String,
    error: String,
  },
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  Emit {
    event: String,
    payload: Option<String>,
  },
  OpenDialog {
    options: OpenDialogOptions,
    callback: String,
    error: String,
  },
  SaveDialog {
    options: SaveDialogOptions,
    callback: String,
    error: String,
  },
  HttpRequest {
    options: Box<HttpRequestOptions>,
    callback: String,
    error: String,
  },
  #[serde(rename_all = "camelCase")]
  #[cfg(any(feature = "embedded-server", feature = "no-server"))]
  LoadAsset {
    asset: String,
    asset_type: String,
    callback: String,
    error: String,
  },
  CliMatches {
    callback: String,
    error: String,
  },
  Notification {
    options: NotificationOptions,
    callback: String,
    error: String,
  },
  RequestNotificationPermission {
    callback: String,
    error: String,
  },
  IsNotificationPermissionGranted {
    callback: String,
    error: String,
  },
}
