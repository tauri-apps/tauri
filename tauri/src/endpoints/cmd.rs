use serde::Deserialize;
use crate::api::path::BaseDirectory;

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
pub struct OpenDialogOptions {
  pub filter: Option<String>,
  #[serde(default)]
  pub multiple: bool,
  #[serde(default)]
  pub directory: bool,
  pub default_path: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveDialogOptions {
  pub filter: Option<String>,
  pub default_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  Init {},
  #[cfg(any(feature = "all-api", feature = "read-text-file"))]
  ReadTextFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "read-binary-file"))]
  ReadBinaryFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "write-file"))]
  WriteFile {
    file: String,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "write-binary-file"))]
  WriteBinaryFile {
    file: String,
    contents: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "read-dir"))]
  ReadDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "copy-file"))]
  CopyFile {
    source: String,
    destination: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "create-dir"))]
  CreateDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "remove-dir"))]
  RemoveDir {
    path: String,
    options: Option<DirOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "remove-file"))]
  RemoveFile {
    path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "rename-file"))]
  RenameFile {
    old_path: String,
    new_path: String,
    options: Option<FileOperationOptions>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "set-title"))]
  SetTitle {
    title: String,
  },
  #[cfg(any(feature = "all-api", feature = "execute"))]
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "open"))]
  Open {
    uri: String,
  },
  ValidateSalt {
    salt: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "event"))]
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  #[cfg(any(feature = "all-api", feature = "event"))]
  Emit {
    event: String,
    payload: Option<String>,
  },
  #[cfg(any(feature = "all-api", feature = "open-dialog"))]
  OpenDialog {
    options: OpenDialogOptions,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "save-dialog"))]
  SaveDialog {
    options: SaveDialogOptions,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "embedded-server", feature = "no-server"))]
  LoadAsset {
    asset: String,
    asset_type: String,
    callback: String,
    error: String,
  },
}
