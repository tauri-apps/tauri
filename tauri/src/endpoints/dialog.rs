use super::cmd::{OpenDialogOptions, SaveDialogOptions};
use crate::api::dialog::{
  ask as ask_dialog, message as message_dialog, pick_folder, save_file, select, select_multiple,
  DialogSelection, Response,
};
use crate::WebviewDispatcher;
use serde_json::Value as JsonValue;

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
pub fn open<W: WebviewDispatcher + 'static>(
  webview: &mut W,
  options: OpenDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview,
    move || {
      let response = if options.multiple {
        select_multiple(options.filter, options.default_path)
      } else if options.directory {
        pick_folder(options.default_path)
      } else {
        select(options.filter, options.default_path)
      };
      response.map(map_response)
    },
    callback,
    error,
  );
  Ok(())
}

/// Shows a save dialog.
#[cfg(save_dialog)]
pub fn save<W: WebviewDispatcher + 'static>(
  webview: &mut W,
  options: SaveDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview,
    move || save_file(options.filter, options.default_path).map(map_response),
    callback,
    error,
  );
  Ok(())
}

/// Shows a message in a dialog.
pub fn message(title: String, message: String) {
  message_dialog(message, title);
}

/// Shows a dialog with a yes/no question.
pub fn ask<W: WebviewDispatcher + 'static>(
  webview: &mut W,
  title: String,
  message: String,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview,
    move || match ask_dialog(message, title) {
      DialogSelection::Yes => Ok(true),
      _ => Ok(false),
    },
    callback,
    error,
  );
  Ok(())
}
