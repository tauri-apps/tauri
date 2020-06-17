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
  pub default_path: Option<String>,
}

/// The options for the save dialog API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogOptions {
  /// The initial path of the dialog.
  pub filter: Option<String>,
  /// The initial path of the dialog.
  pub default_path: Option<String>,
}

/// The options for the notification API.
#[derive(Deserialize)]
pub struct NotificationOptions {
  /// The notification title.
  pub title: Option<String>,
  /// The notification body.
  pub body: String,
  /// The notification icon.
  pub icon: Option<String>,
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The init command
  Init {},
  /// The read text file API.
  #[cfg(any(feature = "all-api", feature = "read-text-file"))]
  ReadTextFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The read binary file API.
  #[cfg(any(feature = "all-api", feature = "read-binary-file"))]
  ReadBinaryFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The write file API.
  #[cfg(any(feature = "all-api", feature = "write-file"))]
  WriteFile {
    file: String,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The read dir API.
  #[cfg(any(feature = "all-api", feature = "read-dir"))]
  ReadDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The copy file API.
  #[cfg(any(feature = "all-api", feature = "copy-file"))]
  CopyFile {
    source: String,
    destination: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The create dir API.
  #[cfg(any(feature = "all-api", feature = "create-dir"))]
  CreateDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The remove dir API.
  #[cfg(any(feature = "all-api", feature = "remove-dir"))]
  RemoveDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  /// The remove file API.
  #[cfg(any(feature = "all-api", feature = "remove-file"))]
  RemoveFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The rename file API.
  #[serde(rename_all = "camelCase")]
  #[cfg(any(feature = "all-api", feature = "rename-file"))]
  RenameFile {
    old_path: String,
    new_path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  /// The set webview title API.
  #[cfg(any(feature = "all-api", feature = "set-title"))]
  SetTitle { title: String },
  /// The execute script API.
  #[cfg(any(feature = "all-api", feature = "execute"))]
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
  /// The open URL in browser API.
  #[cfg(any(feature = "all-api", feature = "open"))]
  Open { uri: String },
  ValidateSalt {
    salt: String,
    callback: String,
    error: String,
  },
  /// The event listen API.
  #[cfg(any(feature = "all-api", feature = "event"))]
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  /// The event emit API.
  #[cfg(any(feature = "all-api", feature = "event"))]
  Emit {
    event: String,
    payload: Option<String>,
  },
  /// The open dialog API.
  #[cfg(any(feature = "all-api", feature = "open-dialog"))]
  OpenDialog {
    options: OpenDialogOptions,
    callback: String,
    error: String,
  },
  /// The save dialog API.
  #[cfg(any(feature = "all-api", feature = "save-dialog"))]
  SaveDialog {
    options: SaveDialogOptions,
    callback: String,
    error: String,
  },
  /// The HTTP request API.
  #[cfg(any(feature = "all-api", feature = "http-request"))]
  HttpRequest {
    options: Box<HttpRequestOptions>,
    callback: String,
    error: String,
  },
  /// The load asset into webview API.
  #[serde(rename_all = "camelCase")]
  #[cfg(any(feature = "embedded-server", feature = "no-server"))]
  LoadAsset {
    asset: String,
    asset_type: String,
    callback: String,
    error: String,
  },
  /// The get CLI matches API.
  #[cfg(feature = "cli")]
  CliMatches { callback: String, error: String },
  /// The show notification API.
  #[cfg(any(feature = "all-api", feature = "notification"))]
  Notification {
    options: NotificationOptions,
    callback: String,
    error: String,
  },
  /// The request notification permission API.
  #[cfg(any(feature = "all-api", feature = "notification"))]
  RequestNotificationPermission { callback: String, error: String },
  /// The notification permission check API.
  #[cfg(any(feature = "all-api", feature = "notification"))]
  IsNotificationPermissionGranted { callback: String, error: String },
}
