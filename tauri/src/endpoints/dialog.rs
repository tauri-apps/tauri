use super::cmd::{OpenDialogOptions, SaveDialogOptions};
use crate::api::dialog::{
  ask as ask_dialog, message as message_dialog, pick_folder, save_file, select, select_multiple,
  DialogSelection, Response,
};
use crate::ApplicationDispatcherExt;
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
pub fn open<D: ApplicationDispatcherExt + 'static>(
  dispatcher: &mut D,
  options: OpenDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    dispatcher,
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
  dispatcher: &mut D,
  options: SaveDialogOptions,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    dispatcher,
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

/// Shows a message in a dialog.
pub fn message(title: String, message: String) {
  message_dialog(message, title);
}

/// Shows a dialog with a yes/no question.
pub fn ask<D: ApplicationDispatcherExt + 'static>(
  dispatcher: &mut D,
  title: String,
  message: String,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    dispatcher,
    move || match ask_dialog(message, title) {
      DialogSelection::Yes => crate::Result::Ok(true),
      _ => crate::Result::Ok(false),
    },
    callback,
    error,
  );
  Ok(())
}
