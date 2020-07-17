mod cmd;
#[allow(unused_imports)]
mod file_system;
mod init;
mod salt;

use init::init;

#[cfg(assets)]
mod asset;
#[cfg(open)]
mod browser;
#[cfg(any(open_dialog, save_dialog))]
mod dialog;
#[cfg(event)]
mod event;
#[cfg(http_request)]
mod http;
#[cfg(notification)]
mod notification;

use webview_rust_sys::Webview;

#[allow(unused_variables)]
pub(crate) fn handle(webview: &mut Webview, arg: &str) -> crate::Result<()> {
  use cmd::Cmd::*;
  match serde_json::from_str(arg) {
    Err(e) => Err(e.into()),
    Ok(command) => {
      match command {
        Init {} => {
          let event_init = init()?;
          webview.eval(&format!(
            r#"{event_init}
                window.external.invoke('{{"cmd":"__initialized"}}')
              "#,
            event_init = event_init
          ));
        }
        ReadTextFile {
          path,
          options,
          callback,
          error,
        } => {
          #[cfg(read_text_file)]
          file_system::read_text_file(webview, path, options, callback, error);
          #[cfg(not(read_text_file))]
          whitelist_error(webview, error, "readTextFile");
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
          whitelist_error(webview, error, "readBinaryFile");
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
          whitelist_error(webview, error, "writeFile");
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
          whitelist_error(webview, error, "writeBinaryFile");
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
          whitelist_error(webview, error, "readDir");
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
          whitelist_error(webview, error, "copyFile");
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
          whitelist_error(webview, error, "createDir");
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
          whitelist_error(webview, error, "removeDir");
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
          whitelist_error(webview, error, "removeFile");
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
          whitelist_error(webview, error, "renameFile");
        }
        SetTitle { title } => {
          #[cfg(set_title)]
          webview.set_title(&title);
          #[cfg(not(set_title))]
          throw_whitelist_error(webview, "title");
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
          throw_whitelist_error(webview, "execute");
        }
        Open { uri } => {
          #[cfg(open)]
          browser::open(uri);
          #[cfg(not(open))]
          throw_whitelist_error(webview, "open");
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
          throw_whitelist_error(webview, "event");
        }
        Emit { event, payload } => {
          #[cfg(event)]
          crate::event::on_event(event, payload);
          #[cfg(not(event))]
          throw_whitelist_error(webview, "event");
        }
        OpenDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(open_dialog)]
          dialog::open(webview, options, callback, error)?;
          #[cfg(not(open_dialog))]
          whitelist_error(webview, error, "title");
        }
        SaveDialog {
          options,
          callback,
          error,
        } => {
          #[cfg(save_dialog)]
          dialog::save(webview, options, callback, error)?;
          #[cfg(not(save_dialog))]
          throw_whitelist_error(webview, "saveDialog");
        }
        HttpRequest {
          options,
          callback,
          error,
        } => {
          #[cfg(http_request)]
          http::make_request(webview, *options, callback, error);
          #[cfg(not(http_request))]
          whitelist_error(webview, error, "httpRequest");
        }
        #[cfg(assets)]
        LoadAsset {
          asset,
          asset_type,
          callback,
          error,
        } => {
          asset::load(webview, asset, asset_type, callback, error);
        }
        CliMatches { callback, error } => {
          #[cfg(cli)]
          crate::execute_promise(
            webview,
            move || match crate::cli::get_matches() {
              Some(matches) => Ok(matches),
              None => Err(anyhow::anyhow!(r#""failed to get matches""#)),
            },
            callback,
            error,
          );
          #[cfg(not(cli))]
          api_error(
            webview,
            error,
            "CLI definition not set under tauri.conf.json > tauri > cli (https://tauri.studio/docs/api/config#tauri)",
          );
        }
        Notification {
          options,
          callback,
          error,
        } => {
          #[cfg(notification)]
          notification::send(webview, options, callback, error);
          #[cfg(not(notification))]
          whitelist_error(webview, error, "notification");
        }
        IsNotificationPermissionGranted { callback, error } => {
          #[cfg(notification)]
          notification::is_permission_granted(webview, callback, error);
          #[cfg(not(notification))]
          whitelist_error(webview, error, "notification");
        }
        RequestNotificationPermission { callback, error } => {
          #[cfg(notification)]
          notification::request_permission(webview, callback, error)?;
          #[cfg(not(notification))]
          whitelist_error(webview, error, "notification");
        }
      }
      Ok(())
    }
  }
}

#[allow(dead_code)]
fn api_error(webview: &mut Webview, error_fn: String, message: &str) {
  let reject_code = tauri_api::rpc::format_callback(error_fn, message);
  webview
    .eval(&reject_code)
}

#[allow(dead_code)]
fn whitelist_error(
  webview: &mut Webview,
  error_fn: String,
  whitelist_key: &str,
) {
  api_error(
    webview,
    error_fn,
    &format!(
      "{}' not whitelisted (https://tauri.studio/docs/api/config#tauri)",
      whitelist_key
    ),
  )
}

#[allow(dead_code)]
fn throw_whitelist_error(webview: &mut Webview, whitelist_key: &str) {
  let reject_code = format!(r#"throw new Error("'{}' not whitelisted")"#, whitelist_key);
  webview
    .eval(&reject_code)
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  #[test]
  // test to see if check init produces a string or not.
  fn check_init() {
    if cfg!(not(event)) {
      let res = super::init();
      match res {
        Ok(s) => assert_eq!(s, ""),
        Err(e) => panic!("init Err {:?}", e.to_string()),
      }
    } else if cfg!(event) {
      let res = super::init();
      match res {
        Ok(s) => assert!(s.contains("window.__TAURI__.promisified")),
        Err(e) => panic!("init Err {:?}", e.to_string()),
      }
    }
  }

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
