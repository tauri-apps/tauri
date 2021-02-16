use crate::api::dialog::{
  ask as ask_dialog, message as message_dialog, pick_folder, save_file, select, select_multiple,
  DialogSelection, Response,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use std::path::PathBuf;

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

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The open dialog API.
  OpenDialog {
    options: OpenDialogOptions,
  },
  /// The save dialog API.
  SaveDialog {
    options: SaveDialogOptions,
  },
  MessageDialog {
    message: String,
  },
  AskDialog {
    title: Option<String>,
    message: String,
  },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<JsonValue> {
    match self {
      Self::OpenDialog { options } => {
        #[cfg(open_dialog)]
        return open(options).and_then(super::to_value);
        #[cfg(not(open_dialog))]
        Err(crate::Error::ApiNotAllowlisted("title".to_string()));
      }
      Self::SaveDialog { options } => {
        #[cfg(save_dialog)]
        return save(options).and_then(super::to_value);
        #[cfg(not(save_dialog))]
        Err(crate::Error::ApiNotAllowlisted("saveDialog".to_string()));
      }
      Self::MessageDialog { message } => {
        let exe = std::env::current_exe()?;
        let app_name = exe
          .file_name()
          .expect("failed to get exe filename")
          .to_string_lossy()
          .to_string();
        message_dialog(app_name, message);
        Ok(JsonValue::Null)
      }
      Self::AskDialog { title, message } => {
        let exe = std::env::current_exe()?;
        let answer = ask(
          title.unwrap_or_else(|| {
            exe
              .file_name()
              .expect("failed to get exe filename")
              .to_string_lossy()
              .to_string()
          }),
          message,
        )?;
        Ok(JsonValue::Bool(answer))
      }
    }
  }
}

/// maps a dialog response to a JS value to eval
#[cfg(any(open_dialog, save_dialog))]
fn map_response(response: Response) -> JsonValue {
  match response {
    Response::Okay(path) => path.into(),
    Response::OkayMultiple(paths) => paths.into(),
    Response::Cancel => JsonValue::Null,
  }
}

/// Shows an open dialog.
#[cfg(open_dialog)]
pub fn open(options: OpenDialogOptions) -> crate::Result<JsonValue> {
  let response = if options.multiple {
    select_multiple(options.filter, options.default_path)
  } else if options.directory {
    pick_folder(options.default_path)
  } else {
    select(options.filter, options.default_path)
  };
  let res = response.map(map_response)?;
  Ok(res)
}

/// Shows a save dialog.
#[cfg(save_dialog)]
pub fn save(options: SaveDialogOptions) -> crate::Result<JsonValue> {
  save_file(options.filter, options.default_path)
    .map(map_response)
    .map_err(Into::into)
}

/// Shows a dialog with a yes/no question.
pub fn ask(title: String, message: String) -> crate::Result<bool> {
  match ask_dialog(message, title) {
    DialogSelection::Yes => Ok(true),
    _ => Ok(false),
  }
}
