use crate::api::dialog::{select, select_multiple, save_file, pick_folder, Response};
use super::cmd::{OpenDialogOptions, SaveDialogOptions};
use web_view::WebView;

fn map_response(response: Response) -> String {
  match response {
    Response::Okay(path) => format!(r#""{}""#, path).replace("\\", "\\\\"),
    Response::OkayMultiple(paths) => format!("{:?}", paths),
    Response::Cancel => panic!("unexpected response type")
  }
}

pub fn open<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: OpenDialogOptions,
  callback: String,
  error: String,
) {
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
      response
        .map(map_response)
        .map_err(|e| crate::ErrorKind::Dialog(e.to_string()).into())
    },
    callback,
    error,
  );
}

pub fn save<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: SaveDialogOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise_sync(
    webview,
    move || {
      save_file(options.filter, options.default_path)
        .map(map_response)
        .map_err(|e| crate::ErrorKind::Dialog(e.to_string()).into())
    },
    callback,
    error,
  );
}