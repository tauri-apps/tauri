mod cmd;
#[allow(unused_imports)]
mod file_system;
mod path;
mod salt;

#[cfg(open)]
mod browser;
mod dialog;
#[cfg(event)]
mod event;
#[cfg(http_request)]
mod http;
#[cfg(notification)]
mod notification;

use crate::{app::Context, webview::Event, ApplicationDispatcherExt};

#[allow(unused_variables)]
pub(crate) async fn handle<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  arg: &str,
  context: &Context,
) -> crate::Result<()> {
  use cmd::Cmd::*;
  match serde_json::from_str(arg) {
    Err(e) => Err(e.into()),
    Ok(command) => {
      match command {
        ReadTextFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_text_file)]
          file_system::read_text_file(webview_manager, path, options, callback, error).await;
          #[cfg(not(read_text_file))]
          allowlist_error(webview_manager, error, "readTextFile");
        }
        ReadBinaryFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_binary_file)]
          file_system::read_binary_file(webview_manager, path, options, callback, error).await;
          #[cfg(not(read_binary_file))]
          allowlist_error(webview_manager, error, "readBinaryFile");
        }
        WriteFile {
          path,
          contents,
          options,
          callback,
          error,
        } => {
          #[cfg(write_file)]
          file_system::write_file(webview_manager, path, contents, options, callback, error).await;
          #[cfg(not(write_file))]
          allowlist_error(webview_manager, error, "writeFile");
        }
        WriteBinaryFile {
          path,
          contents,
          options,
          callback,
          error,
        } => {
          #[cfg(write_binary_file)]
          file_system::write_binary_file(webview_manager, path, contents, options, callback, error)
            .await;
          #[cfg(not(write_binary_file))]
          allowlist_error(webview_manager, error, "writeBinaryFile");
        }
        ReadDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_dir)]
          file_system::read_dir(webview_manager, path, options, callback, error).await;
          #[cfg(not(read_dir))]
          allowlist_error(webview_manager, error, "readDir");
        }
        CopyFile {
          source,
          destination,
          options,
          callback,
          error,
        } => {
          #[cfg(copy_file)]
          file_system::copy_file(
            webview_manager,
            source,
            destination,
            options,
            callback,
            error,
          )
          .await;
          #[cfg(not(copy_file))]
          allowlist_error(webview_manager, error, "copyFile");
        }
        CreateDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(create_dir)]
          file_system::create_dir(webview_manager, path, options, callback, error).await;
          #[cfg(not(create_dir))]
          allowlist_error(webview_manager, error, "createDir");
        }
        RemoveDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(remove_dir)]
          file_system::remove_dir(webview_manager, path, options, callback, error).await;
          #[cfg(not(remove_dir))]
          allowlist_error(webview_manager, error, "removeDir");
        }
        RemoveFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(remove_file)]
          file_system::remove_file(webview_manager, path, options, callback, error).await;
          #[cfg(not(remove_file))]
          allowlist_error(webview_manager, error, "removeFile");
        }
        RenameFile {
          old_path,
          new_path,
          options,
          callback,
          error,
        } => {
          #[cfg(rename_file)]
          file_system::rename_file(
            webview_manager,
            old_path,
            new_path,
            options,
            callback,
            error,
          )
          .await;
          #[cfg(not(rename_file))]
          allowlist_error(webview_manager, error, "renameFile");
        }
        ResolvePath {
          path,
          directory,
          callback,
          error,
        } => {
          #[cfg(path_api)]
          path::resolve_path(webview_manager, path, directory, callback, error).await;
          #[cfg(not(path_api))]
          allowlist_error(webview_manager, error, "pathApi");
        }
        SetTitle { title } => {
          webview_manager.current_webview()?.set_title(&title);
          #[cfg(not(set_title))]
          throw_allowlist_error(webview_manager, "title");
        }
        Execute {
          command,
          args,
          callback,
          error,
        } => {
          #[cfg(execute)]
          crate::call(webview_manager, command, args, callback, error).await;
          #[cfg(not(execute))]
          throw_allowlist_error(webview_manager, "execute");
        }
        Open { uri } => {
          #[cfg(open)]
          browser::open(uri);
          #[cfg(not(open))]
          throw_allowlist_error(webview_manager, "open");
        }
        ValidateSalt {
          salt,
          callback,
          error,
        } => {
          salt::validate(webview_manager, salt, callback, error)?;
        }
        Listen {
          event,
          handler,
          once,
        } => {
          #[cfg(event)]
          {
            let js_string = event::listen_fn(event, handler, once)?;
            webview_manager.current_webview()?.eval(&js_string);
          }
          #[cfg(not(event))]
          throw_allowlist_error(webview_manager, "event");
        }
        Emit { event, payload } => {
          #[cfg(event)]
          crate::event::on_event(event, payload);
          #[cfg(not(event))]
          throw_allowlist_error(webview_manager, "event");
        }
        OpenDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(open_dialog)]
          dialog::open(webview_manager, options, callback, error)?;
          #[cfg(not(open_dialog))]
          allowlist_error(webview_manager, error, "title");
        }
        SaveDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(save_dialog)]
          dialog::save(webview_manager, options, callback, error)?;
          #[cfg(not(save_dialog))]
          throw_allowlist_error(webview_manager, "saveDialog");
        }
        MessageDialog { message } => {
          let exe = std::env::current_exe()?;
          let exe_dir = exe.parent().expect("failed to get exe directory");
          let app_name = exe
            .file_name()
            .expect("failed to get exe filename")
            .to_string_lossy()
            .to_string();
          webview_manager
            .current_webview()?
            .send_event(Event::Run(Box::new(move || {
              dialog::message(app_name, message);
            })));
        }
        AskDialog {
          title,
          message,
          callback,
          error,
        } => {
          let exe = std::env::current_exe()?;
          dialog::ask(
            webview_manager,
            title.unwrap_or_else(|| {
              let exe_dir = exe.parent().expect("failed to get exe directory");
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
        HttpRequest {
          options,
          callback,
          error,
        } => {
          #[cfg(http_request)]
          http::make_request(webview_manager, *options, callback, error).await;
          #[cfg(not(http_request))]
          allowlist_error(webview_manager, error, "httpRequest");
        }
        CliMatches { callback, error } => {
          #[cfg(cli)]
          {
            let matches = tauri_api::cli::get_matches(&context.config).map_err(|e| e.into());
            crate::execute_promise(webview_manager, async move { matches }, callback, error).await;
          }
          #[cfg(not(cli))]
          api_error(
            webview_manager,
            error,
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri.cli)",
          );
        }
        Notification {
          options,
          callback,
          error,
        } => {
          #[cfg(notification)]
          notification::send(webview_manager, options, callback, error, &context.config).await;
          #[cfg(not(notification))]
          allowlist_error(webview_manager, error, "notification");
        }
        IsNotificationPermissionGranted { callback, error } => {
          #[cfg(notification)]
          notification::is_permission_granted(webview_manager, callback, error).await;
          #[cfg(not(notification))]
          allowlist_error(webview_manager, error, "notification");
        }
        RequestNotificationPermission { callback, error } => {
          #[cfg(notification)]
          notification::request_permission(webview_manager, callback, error)?;
          #[cfg(not(notification))]
          allowlist_error(webview_manager, error, "notification");
        }
      }
      Ok(())
    }
  }
}

#[allow(dead_code)]
fn api_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  error_fn: String,
  message: &str,
) {
  let reject_code = tauri_api::rpc::format_callback(error_fn, message);
  if let Ok(dispatcher) = webview_manager.current_webview() {
    dispatcher.eval(&reject_code);
  }
}

#[allow(dead_code)]
fn allowlist_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  error_fn: String,
  allowlist_key: &str,
) {
  api_error(
    webview_manager,
    error_fn,
    &format!(
      "{}' not on the allowlist (https://tauri.studio/docs/api/config#tauri.allowlist)",
      allowlist_key
    ),
  )
}

#[allow(dead_code)]
fn throw_allowlist_error<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  allowlist_key: &str,
) {
  let reject_code = format!(
    r#"throw new Error("'{}' not on the allowlist")"#,
    allowlist_key
  );
  if let Ok(dispatcher) = webview_manager.current_webview() {
    dispatcher.eval(&reject_code);
  }
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  // check the listen_fn for various usecases.
  proptest! {
    #[cfg(event)]
    #[test]
    fn check_listen_fn(event in "", handler in "", once in proptest::bool::ANY) {
      super::event::listen_fn(event, handler, once).expect("listen_fn failed");
    }
  }

  // Test the open func to see if proper uris can be opened by the browser.
  proptest! {
    #[cfg(open)]
    #[test]
    fn check_open(uri in r"(http://)([\\w\\d\\.]+([\\w]{2,6})?)") {
      super::browser::open(uri);
  }
  }
}
