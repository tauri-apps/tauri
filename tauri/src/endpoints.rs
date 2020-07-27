mod cmd;
#[allow(unused_imports)]
mod file_system;
mod salt;

#[cfg(assets)]
mod asset;
#[cfg(open)]
mod browser;
mod dialog;
#[cfg(event)]
mod event;
#[cfg(http_request)]
mod http;
#[cfg(notification)]
mod notification;

use crate::app::AppContext;
use webview_official::Webview;

#[allow(unused_variables)]
pub(crate) fn handle(webview: &mut Webview<'_>, arg: &str, ctx: &AppContext) -> crate::Result<()> {
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
          file_system::read_text_file(webview, path, options, callback, error);
          #[cfg(not(read_text_file))]
          allowlist_error(webview, error, "readTextFile");
        }
        ReadBinaryFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_binary_file)]
          file_system::read_binary_file(webview, path, options, callback, error);
          #[cfg(not(read_binary_file))]
          allowlist_error(webview, error, "readBinaryFile");
        }
        WriteFile {
          path,
          contents,
          options,
          callback,
          error,
        } => {
          #[cfg(write_file)]
          file_system::write_file(webview, path, contents, options, callback, error);
          #[cfg(not(write_file))]
          allowlist_error(webview, error, "writeFile");
        }
        WriteBinaryFile {
          path,
          contents,
          options,
          callback,
          error,
        } => {
          #[cfg(write_binary_file)]
          file_system::write_binary_file(webview, path, contents, options, callback, error);
          #[cfg(not(write_binary_file))]
          allowlist_error(webview, error, "writeBinaryFile");
        }
        ReadDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_dir)]
          file_system::read_dir(webview, path, options, callback, error);
          #[cfg(not(read_dir))]
          allowlist_error(webview, error, "readDir");
        }
        CopyFile {
          source,
          destination,
          options,
          callback,
          error,
        } => {
          #[cfg(copy_file)]
          file_system::copy_file(webview, source, destination, options, callback, error);
          #[cfg(not(copy_file))]
          allowlist_error(webview, error, "copyFile");
        }
        CreateDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(create_dir)]
          file_system::create_dir(webview, path, options, callback, error);
          #[cfg(not(create_dir))]
          allowlist_error(webview, error, "createDir");
        }
        RemoveDir {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(remove_dir)]
          file_system::remove_dir(webview, path, options, callback, error);
          #[cfg(not(remove_dir))]
          allowlist_error(webview, error, "removeDir");
        }
        RemoveFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(remove_file)]
          file_system::remove_file(webview, path, options, callback, error);
          #[cfg(not(remove_file))]
          allowlist_error(webview, error, "removeFile");
        }
        RenameFile {
          old_path,
          new_path,
          options,
          callback,
          error,
        } => {
          #[cfg(rename_file)]
          file_system::rename_file(webview, old_path, new_path, options, callback, error);
          #[cfg(not(rename_file))]
          allowlist_error(webview, error, "renameFile");
        }
        SetTitle { title } => {
          #[cfg(set_title)]
          webview.set_title(&title);
          #[cfg(not(set_title))]
          throw_allowlist_error(webview, "title");
        }
        Execute {
          command,
          args,
          callback,
          error,
        } => {
          #[cfg(execute)]
          crate::call(webview, command, args, callback, error);
          #[cfg(not(execute))]
          throw_allowlist_error(webview, "execute");
        }
        Open { uri } => {
          #[cfg(open)]
          browser::open(uri);
          #[cfg(not(open))]
          throw_allowlist_error(webview, "open");
        }
        ValidateSalt {
          salt,
          callback,
          error,
        } => {
          salt::validate(webview, salt, callback, error)?;
        }
        Listen {
          event,
          handler,
          once,
        } => {
          #[cfg(event)]
          {
            let js_string = event::listen_fn(event, handler, once)?;
            webview.eval(&js_string);
          }
          #[cfg(not(event))]
          throw_allowlist_error(webview, "event");
        }
        Emit { event, payload } => {
          #[cfg(event)]
          crate::event::on_event(event, payload);
          #[cfg(not(event))]
          throw_allowlist_error(webview, "event");
        }
        OpenDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(open_dialog)]
          dialog::open(webview, options, callback, error)?;
          #[cfg(not(open_dialog))]
          allowlist_error(webview, error, "title");
        }
        SaveDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(save_dialog)]
          dialog::save(webview, options, callback, error)?;
          #[cfg(not(save_dialog))]
          throw_allowlist_error(webview, "saveDialog");
        }
        MessageDialog { message } => {
          let exe = std::env::current_exe()?;
          let exe_dir = exe.parent().expect("failed to get exe directory");
          let app_name = exe
            .file_name()
            .expect("failed to get exe filename")
            .to_string_lossy();
          dialog::message(app_name.to_string(), message);
        }
        AskDialog {
          title,
          message,
          callback,
          error,
        } => {
          let exe = std::env::current_exe()?;
          dialog::ask(
            webview,
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
          http::make_request(webview, *options, callback, error);
          #[cfg(not(http_request))]
          allowlist_error(webview, error, "httpRequest");
        }
        #[cfg(assets)]
        LoadAsset {
          asset,
          asset_type,
          callback,
          error,
        } => {
          asset::load(webview, asset, asset_type, callback, error, &ctx);
        }
        CliMatches { callback, error } => {
          #[cfg(cli)]
          {
            // TODO: memoize this?  previous used a static but that's not possible anymore
            let matches = tauri_api::cli::get_matches(&ctx.config);
            crate::execute_promise(webview, move || matches, callback, error);
          }
          #[cfg(not(cli))]
          api_error(
            webview,
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
          notification::send(webview, options, callback, error, &ctx.config);
          #[cfg(not(notification))]
          allowlist_error(webview, error, "notification");
        }
        IsNotificationPermissionGranted { callback, error } => {
          #[cfg(notification)]
          notification::is_permission_granted(webview, callback, error);
          #[cfg(not(notification))]
          allowlist_error(webview, error, "notification");
        }
        RequestNotificationPermission { callback, error } => {
          #[cfg(notification)]
          notification::request_permission(webview, callback, error)?;
          #[cfg(not(notification))]
          allowlist_error(webview, error, "notification");
        }
      }
      Ok(())
    }
  }
}

#[allow(dead_code)]
fn api_error(webview: &mut Webview<'_>, error_fn: String, message: &str) {
  let reject_code = tauri_api::rpc::format_callback(error_fn, message);
  webview.eval(&reject_code)
}

#[allow(dead_code)]
fn allowlist_error(webview: &mut Webview<'_>, error_fn: String, allowlist_key: &str) {
  api_error(
    webview,
    error_fn,
    &format!(
      "{}' not on the allowlist (https://tauri.studio/docs/api/config#tauri.allowlist)",
      allowlist_key
    ),
  )
}

#[allow(dead_code)]
fn throw_allowlist_error(webview: &mut Webview<'_>, allowlist_key: &str) {
  let reject_code = format!(
    r#"throw new Error("'{}' not on the allowlist")"#,
    allowlist_key
  );
  webview.eval(&reject_code)
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
