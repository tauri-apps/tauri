use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReadDirOptions {
  #[serde(default)]
  pub recursive: bool
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
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "read-binary-file"))]
  ReadBinaryFile {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "write-file"))]
  WriteFile {
    file: String,
    contents: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "read-dir"))]
  ReadDir {
    path: String,
    options: Option<ReadDirOptions>,
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
  #[cfg(not(any(feature = "dev-server", feature = "embedded-server")))]
  LoadAsset {
    asset: String,
    asset_type: String,
    callback: String,
    error: String,
  },
}
