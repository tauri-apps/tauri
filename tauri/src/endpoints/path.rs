#![cfg(path_api)]
use tauri_api::path;
use tauri_api::path::BaseDirectory;
use webview_official::Webview;

pub fn get_directory(
  webview: &mut Webview<'_>,
  directory: BaseDirectory,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      Result::Ok(match directory {
        BaseDirectory::Audio => path::audio_dir(),
        BaseDirectory::Cache => path::cache_dir(),
        BaseDirectory::Config => path::config_dir(),
        BaseDirectory::Data => path::data_dir(),
        BaseDirectory::LocalData => path::local_data_dir(),
        BaseDirectory::Desktop => path::desktop_dir(),
        BaseDirectory::Document => path::document_dir(),
        BaseDirectory::Download => path::download_dir(),
        BaseDirectory::Executable => path::executable_dir(),
        BaseDirectory::Font => path::font_dir(),
        BaseDirectory::Home => path::home_dir(),
        BaseDirectory::Picture => path::picture_dir(),
        BaseDirectory::Public => path::public_dir(),
        BaseDirectory::Runtime => path::runtime_dir(),
        BaseDirectory::Template => path::template_dir(),
        BaseDirectory::Video => path::video_dir(),
        BaseDirectory::Resource => path::resource_dir(),
        BaseDirectory::App => path::app_dir(),
      })
    },
    callback,
    error,
  )
}

pub fn resolve_path(
  webview: &mut Webview<'_>,
  path: String,
  directory: Option<BaseDirectory>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || path::resolve_path(path, directory),
    callback,
    error,
  )
}
