use web_view::WebView;

use tauri_api::dir;
use tauri_api::file;

use std::fs::File;
use std::io::Write;

pub fn list<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      dir::walk_dir(path.to_string())
        .map_err(|e| crate::ErrorKind::Promise(e.to_string()).into())
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

pub fn list_dirs<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      dir::list_dir_contents(&path)
        .map_err(|e| crate::ErrorKind::Promise(e.to_string()).into())
        .and_then(|f| serde_json::to_string(&f).map_err(|err| err.into()))
    },
    callback,
    error,
  );
}

pub fn write_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  file: String,
  contents: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      File::create(file)
        .map_err(|e| crate::ErrorKind::Promise(e.to_string()).into())
        .and_then(|mut f| {
          f.write_all(contents.as_bytes())
            .map_err(|err| err.into())
            .map(|_| "".to_string())
        })
    },
    callback,
    error,
  );
}

pub fn read_text_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_string(path)
        .map_err(|e| crate::ErrorKind::Promise(e.to_string()).into())
        .and_then(|f| {
          serde_json::to_string(&f)
            .map_err(|err| err.into())
            .map(|s| s.to_string())
        })
    },
    callback,
    error,
  );
}

pub fn read_binary_file<T: 'static>(
  webview: &mut WebView<'_, T>,
  path: String,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      file::read_binary(path)
        .map_err(|e| crate::ErrorKind::Promise(e.to_string()).into())
        .and_then(|f| {
          serde_json::to_string(&f)
            .map_err(|err| err.into())
            .map(|s| s.to_string())
        })
    },
    callback,
    error,
  );
}
