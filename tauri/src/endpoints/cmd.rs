use std::path::PathBuf;

use crate::api::path::BaseDirectory;
use serde::Deserialize;
use tauri_api::http::HttpRequestOptions;

/// The options for the directory functions on the file system API.
#[derive(Deserialize)]
pub struct DirOperationOptions {
  /// Whether the API should recursively perform the operation on the directory.
  #[serde(default)]
  pub recursive: bool,
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined directory path.
  pub dir: Option<BaseDirectory>,
}

/// The options for the file functions on the file system API.
#[derive(Deserialize)]
pub struct FileOperationOptions {
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined file path.
  pub dir: Option<BaseDirectory>,
}

/// The options for the open dialog API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
  /// The initial path of the dialog.
  pub filter: Option<String>,
  /// Whether the dialog allows multiple selection or not.
  #[serde(default)]
  pub multiple: bool,
  /// Whether the dialog is a directory selection (`true` value) or file selection (`false` value).
  #[serde(default)]
  pub directory: bool,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
}

/// The options for the save dialog API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogOptions {
  /// The initial path of the dialog.
  pub filter: Option<String>,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
}

/// The options for the notification API.
#[derive(Deserialize)]
pub struct NotificationOptions {
  /// The notification title.
  pub title: String,
  /// The notification body.
  pub body: Option<String>,
  /// The notification icon.
  pub icon: Option<String>,
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The read text file API.
  ReadTextFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The read binary file API.
  ReadBinaryFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The write file API.
  WriteFile {
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The write binary file API.
  WriteBinaryFile {
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The read dir API.
  ReadDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The copy file API.
  CopyFile {
    source: PathBuf,
    destination: PathBuf,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The create dir API.
  CreateDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The remove dir API.
  RemoveDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The remove file API.
  RemoveFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The rename file API.
  #[serde(rename_all = "camelCase")]
  RenameFile {
    old_path: PathBuf,
    new_path: PathBuf,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The Directories API
  GetDirectory {
    directory: BaseDirectory,
    callback: String,
    error: String,
  },
  /// The resolve path API
  ResolvePath {
    path: String,
    directory: Option<BaseDirectory>,
    callback: String,
    error: String,
  },
  /// The set webview title API.
  SetTitle {
    title: String,
  },
  /// The execute script API.
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
  /// The open URL in browser API
  Open {
    uri: String,
  },
  ValidateSalt {
    salt: String,
    callback: String,
    error: String,
  },
  /// The event listen API.
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  /// The event emit API.
  Emit {
    event: String,
    payload: Option<String>,
  },
  /// The open dialog API.
  OpenDialog {
    options: OpenDialogOptions,
    callback: String,
    error: String,
  },
  /// The save dialog API.
  SaveDialog {
    options: SaveDialogOptions,
    callback: String,
    error: String,
  },
  MessageDialog {
    message: String,
  },
  AskDialog {
    title: Option<String>,
    message: String,
    callback: String,
    error: String,
  },
  /// The HTTP request API.
  HttpRequest {
    options: Box<HttpRequestOptions>,
    callback: String,
    error: String,
  },
  /// The load asset into webview API.
  #[serde(rename_all = "camelCase")]
  #[cfg(assets)]
  LoadAsset {
    asset: String,
    asset_type: String,
    callback: String,
    error: String,
  },
  /// The get CLI matches API.
  CliMatches {
    callback: String,
    error: String,
  },
  /// The show notification API.
  Notification {
    options: NotificationOptions,
    callback: String,
    error: String,
  },
  /// The request notification permission API.
  RequestNotificationPermission {
    callback: String,
    error: String,
  },
  /// The notification permission check API.
  IsNotificationPermissionGranted {
    callback: String,
    error: String,
  },
}
