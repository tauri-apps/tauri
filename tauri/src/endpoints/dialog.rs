use crate::{
  api::dialog::{
    ask as ask_dialog, message as message_dialog, pick_folder, save_file, select, select_multiple,
    DialogSelection, Response,
  },
  app::{ApplicationDispatcherExt, Event},
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
}

impl Cmd {
  pub async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<()> {
    match self {
      Self::OpenDialog {
        options,
        callback,
        error,
      } => {
        #[cfg(open_dialog)]
        open(webview_manager, options, callback, error)?;
        #[cfg(not(open_dialog))]
        allowlist_error(webview_manager, error, "title");
      }
      Self::SaveDialog {
        options,
        callback,
        error,
      } => {
        #[cfg(save_dialog)]
        save(webview_manager, options, callback, error)?;
        #[cfg(not(save_dialog))]
        throw_allowlist_error(webview_manager, "saveDialog");
      }
      Self::MessageDialog { message } => {
        let exe = std::env::current_exe()?;
        let app_name = exe
          .file_name()
          .expect("failed to get exe filename")
          .to_string_lossy()
          .to_string();
        webview_manager
          .current_webview()?
          .send_event(Event::Run(Box::new(move || {
            message_dialog(app_name, message);
          })));
      }
      Self::AskDialog {
        title,
        message,
        callback,
        error,
      } => {
        let exe = std::env::current_exe()?;
        ask(
          webview_manager,
          title.unwrap_or_else(|| {
            exe
              .file_name()
              .expect("failed to get exe filename")
              .to_string_lossy()
              .to_string()
          }),
          message,
          callback,
          error,
        )?;
      }
    }
    Ok(())
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
pub fn open<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  options: OpenDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview_manager,
    move || {
      let response = if options.multiple {
        select_multiple(options.filter, options.default_path)
      } else if options.directory {
        pick_folder(options.default_path)
      } else {
        select(options.filter, options.default_path)
      };
      let res = response.map(map_response)?;
      Ok(res)
    },
    callback,
    error,
  );
  Ok(())
}

/// Shows a save dialog.
#[cfg(save_dialog)]
pub fn save<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  options: SaveDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview_manager,
    move || {
      save_file(options.filter, options.default_path)
        .map(map_response)
        .map_err(|e| e.into())
    },
    callback,
    error,
  );
  Ok(())
}

/// Shows a dialog with a yes/no question.
pub fn ask<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  title: String,
  message: String,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview_manager,
    move || match ask_dialog(message, title) {
      DialogSelection::Yes => crate::Result::Ok(true),
      _ => crate::Result::Ok(false),
    },
    callback,
    error,
  );
  Ok(())
}
