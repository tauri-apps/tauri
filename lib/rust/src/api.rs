mod cmd;

use proton_ui::WebView;

#[allow(unused_variables)]
pub fn handler<T: 'static>(webview: &mut WebView<'_, T>, arg: &str) -> bool {
  #[cfg(feature = "api")]
  {
    use cmd::Cmd::*;
    match serde_json::from_str(arg) {
      Err(_) => false,
      Ok(command) => {
        match command {
          #[cfg(any(feature = "all-api", feature = "readTextFile"))]
          ReadTextFile {
            path,
            callback,
            error,
          } => {
            super::file_system::read_text_file(webview, path, callback, error);
          }
          #[cfg(any(feature = "all-api", feature = "readBinaryFile"))]
          ReadBinaryFile {
            path,
            callback,
            error,
          } => {
            super::file_system::read_binary_file(webview, path, callback, error);
          }
          #[cfg(any(feature = "all-api", feature = "writeFile"))]
          WriteFile {
            file,
            contents,
            callback,
            error,
          } => {
            super::file_system::write_file(webview, file, contents, callback, error);
          }
          #[cfg(any(feature = "all-api", feature = "listDirs"))]
          ListDirs {
            path,
            callback,
            error,
          } => {
            super::file_system::list_dirs(webview, path, callback, error);
          }
          #[cfg(any(feature = "all-api", feature = "listFiles"))]
          ListFiles {
            path,
            callback,
            error,
          } => {
            super::file_system::list(webview, path, callback, error);
          }
          #[cfg(any(feature = "all-api", feature = "setTitle"))]
          SetTitle { title } => {
            webview.set_title(&title).unwrap();
          }
          #[cfg(any(feature = "all-api", feature = "execute"))]
          Execute {
            command,
            args,
            callback,
            error,
          } => {
            super::command::call(webview, command, args, callback, error);
          }
        }
        true
      }
    }
  }
  #[cfg(not(feature = "api"))]
  false
}
