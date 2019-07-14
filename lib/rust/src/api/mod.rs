mod cmd;

use proton_ui::WebView;

pub fn handler<T: 'static>(webview: &mut WebView<T>, arg: &str) -> bool
{
    use cmd::Cmd::*;
    match serde_json::from_str(arg) {
        Err(_) => {
            false
        },
        Ok(command) => {
            match command {
                Init => (),
                ReadAsString { path, callback, error } => {
                    super::file_system::read_text_file(webview, path, callback, error);
                }
                ReadAsBinary { path, callback, error } => {
                    super::file_system::read_binary_file(webview, path, callback, error);
                }
                Write { file, contents, callback, error } => {
                    super::file_system::write_file(webview, file, contents, callback, error);
                }
                ListDirs { path, callback, error } => {
                    super::file_system::list_dirs(webview, path, callback, error);
                }
                List { path, callback, error } => {
                    super::file_system::list(webview, path, callback, error);
                }
                SetTitle { title } => {
                    webview.set_title(&title).unwrap();
                }
                Call { command, args, callback, error } => {
                    super::command::call(webview, command, args, callback, error);
                }
            }
            true
        }
    }
}
