mod cmd;
mod dialog;
mod file_system;
mod http;
mod salt;

#[cfg(any(feature = "embedded-server", feature = "no-server"))]
use std::path::PathBuf;
use tauri_api::config::Config;
use web_view::WebView;

#[cfg(windows)]
use std::path::MAIN_SEPARATOR;

#[allow(unused_variables)]
pub(crate) fn handle<T: 'static>(
  webview: &mut WebView<'_, T>,
  arg: &str,
  config: &Config,
) -> crate::Result<()> {
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
          ))?;
        }
        #[cfg(any(feature = "all-api", feature = "read-text-file"))]
        ReadTextFile {
          path,
          options,
          callback,
          error,
        } => {
          file_system::read_text_file(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "read-binary-file"))]
        ReadBinaryFile {
          path,
          options,
          callback,
          error,
        } => {
          file_system::read_binary_file(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "write-file"))]
        WriteFile {
          file,
          contents,
          options,
          callback,
          error,
        } => {
          file_system::write_file(webview, file, contents, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "read-dir"))]
        ReadDir {
          path,
          options,
          callback,
          error,
        } => {
          file_system::read_dir(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "copy-file"))]
        CopyFile {
          source,
          destination,
          options,
          callback,
          error,
        } => {
          file_system::copy_file(webview, source, destination, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "create-dir"))]
        CreateDir {
          path,
          options,
          callback,
          error,
        } => {
          file_system::create_dir(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "remove-dir"))]
        RemoveDir {
          path,
          options,
          callback,
          error,
        } => {
          file_system::remove_dir(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "remove-file"))]
        RemoveFile {
          path,
          options,
          callback,
          error,
        } => {
          file_system::remove_file(webview, path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "rename-file"))]
        RenameFile {
          old_path,
          new_path,
          options,
          callback,
          error,
        } => {
          file_system::rename_file(webview, old_path, new_path, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "set-title"))]
        SetTitle { title } => {
          webview.set_title(&title)?;
        }
        #[cfg(any(feature = "all-api", feature = "execute"))]
        Execute {
          command,
          args,
          callback,
          error,
        } => {
          crate::call(webview, command, args, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "open"))]
        Open { uri } => {
          open_fn(uri)?;
        }
        ValidateSalt {
          salt,
          callback,
          error,
        } => {
          salt::validate(webview, salt, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "event"))]
        Listen {
          event,
          handler,
          once,
        } => {
          let js_string = listen_fn(event, handler, once)?;
          webview.eval(&js_string)?;
        }
        #[cfg(any(feature = "all-api", feature = "event"))]
        Emit { event, payload } => {
          crate::event::on_event(event, payload);
        }
        #[cfg(any(feature = "all-api", feature = "open-dialog"))]
        OpenDialog {
          options,
          callback,
          error,
        } => {
          dialog::open(webview, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "save-dialog"))]
        SaveDialog {
          options,
          callback,
          error,
        } => {
          dialog::save(webview, options, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "http-request"))]
        HttpRequest {
          options,
          callback,
          error,
        } => {
          http::make_request(webview, *options, callback, error);
        }
        #[cfg(any(feature = "embedded-server", feature = "no-server"))]
        LoadAsset {
          asset,
          asset_type,
          callback,
          error,
        } => {
          load_asset(webview, asset, asset_type, callback, error)?;
        }
        #[cfg(feature = "cli")]
        CliMatches { callback, error } => crate::execute_promise(
          webview,
          move || match crate::cli::get_matches() {
            Some(matches) => Ok(serde_json::to_string(matches)?),
            None => Err(anyhow::anyhow!(r#""failed to get matches""#)),
          },
          callback,
          error,
        ),
        #[cfg(any(feature = "all-api", feature = "notification"))]
        Notification {
          options,
          callback,
          error,
        } => {
          notification(
            webview,
            options,
            callback,
            error,
            #[cfg(windows)]
            config,
          )?;
        }
        #[cfg(any(feature = "all-api", feature = "notification"))]
        IsNotificationPermissionGranted { callback, error } => {
          crate::execute_promise(
            webview,
            move || {
              let settings = crate::settings::read_settings()?;
              if let Some(allow_notification) = settings.allow_notification {
                Ok(allow_notification.to_string())
              } else {
                Ok("null".to_string())
              }
            },
            callback,
            error,
          );
        }
        #[cfg(any(feature = "all-api", feature = "notification"))]
        RequestNotificationPermission { callback, error } => crate::execute_promise_sync(
          webview,
          move || {
            let mut settings = crate::settings::read_settings()?;
            let granted = r#""granted""#.to_string();
            let denied = r#""denied""#.to_string();
            if let Some(allow_notification) = settings.allow_notification {
              return Ok(if allow_notification { granted } else { denied });
            }
            let answer = tauri_api::dialog::ask(
              "This app wants to show notifications. Do you allow?",
              "Permissions",
            );
            match answer {
              tauri_api::dialog::DialogSelection::Yes => {
                settings.allow_notification = Some(true);
                crate::settings::write_settings(settings)?;
                Ok(granted)
              }
              tauri_api::dialog::DialogSelection::No => Ok(denied),
              _ => Ok(r#""default""#.to_string()),
            }
          },
          callback,
          error,
        ),
      }
      Ok(())
    }
  }
}

fn init() -> crate::Result<String> {
  #[cfg(not(any(feature = "all-api", feature = "event")))]
  return Ok(String::from(""));
  #[cfg(any(feature = "all-api", feature = "event"))]
  return Ok(format!(
            "
              window['{queue}'] = [];
              window['{fn}'] = function (payload, salt, ignoreQueue) {{
              const listeners = (window['{listeners}'] && window['{listeners}'][payload.type]) || []
              if (!ignoreQueue && listeners.length === 0) {{
                window['{queue}'].push({{
                  payload: payload,
                  salt: salt
                }})
              }}

              if (listeners.length > 0) {{
                window.tauri.promisified({{
                  cmd: 'validateSalt',
                  salt: salt
                }}).then(function () {{
                  for (let i = listeners.length - 1; i >= 0; i--) {{
                    const listener = listeners[i]
                    if (listener.once)
                      listeners.splice(i, 1)
                    listener.handler(payload)
                  }}
                }})
              }}
            }}
            ",
            fn = crate::event::emit_function_name(),
            queue = crate::event::event_queue_object_name(),
            listeners = crate::event::event_listeners_object_name()
  ));
}

#[cfg(any(feature = "all-api", feature = "open"))]
fn open_fn(uri: String) -> crate::Result<()> {
  crate::spawn(move || {
    #[cfg(test)]
    assert!(uri.contains("http://"));

    #[cfg(not(test))]
    webbrowser::open(&uri).expect("Failed to open webbrowser with uri");
  });

  Ok(())
}

#[cfg(any(feature = "all-api", feature = "event"))]
fn listen_fn(event: String, handler: String, once: bool) -> crate::Result<String> {
  Ok(format!(
    "if (window['{listeners}'] === void 0) {{
      window['{listeners}'] = {{}}
      }}
    if (window['{listeners}']['{evt}'] === void 0) {{
      window['{listeners}']['{evt}'] = []
    }}
    window['{listeners}']['{evt}'].push({{
      handler: window['{handler}'],
      once: {once_flag}
    }});

    for (let i = 0; i < (window['{queue}'] || []).length; i++) {{
      const e = window['{queue}'][i];
      window['{emit}'](e.payload, e.salt, true)
    }}
  ",
    listeners = crate::event::event_listeners_object_name(),
    queue = crate::event::event_queue_object_name(),
    emit = crate::event::emit_function_name(),
    evt = event,
    handler = handler,
    once_flag = if once { "true" } else { "false" }
  ))
}

#[cfg(any(feature = "embedded-server", feature = "no-server"))]
fn load_asset<T: 'static>(
  webview: &mut WebView<'_, T>,
  asset: String,
  asset_type: String,
  callback: String,
  error: String,
) -> crate::Result<()> {
  let handle = webview.handle();
  crate::execute_promise(
    webview,
    move || {
      let mut path = PathBuf::from(if asset.starts_with('/') {
        asset.replacen("/", "", 1)
      } else {
        asset.clone()
      });
      let mut read_asset;
      loop {
        read_asset = crate::assets::ASSETS.get(&format!(
          "{}/{}",
          env!("TAURI_DIST_DIR"),
          path.to_string_lossy()
        ));
        if read_asset.is_err() {
          match path.iter().next() {
            Some(component) => {
              let first_component = component.to_str().expect("failed to read path component");
              path = PathBuf::from(path.to_string_lossy().replacen(
                format!("{}/", first_component).as_str(),
                "",
                1,
              ));
            }
            None => {
              return Err(anyhow::anyhow!("Asset '{}' not found", asset));
            }
          }
        } else {
          break;
        }
      }

      if asset_type == "image" {
        let ext = if asset.ends_with("gif") {
          "gif"
        } else if asset.ends_with("png") {
          "png"
        } else {
          "jpeg"
        };
        Ok(format!(
          r#""data:image/{};base64,{}""#,
          ext,
          base64::encode(&read_asset.expect("Failed to read asset type").into_owned())
        ))
      } else {
        handle
          .dispatch(move |_webview| {
            let asset_bytes = &read_asset.expect("Failed to read asset type").into_owned();
            let asset_str =
              &std::str::from_utf8(asset_bytes).expect("failed to convert asset bytes to u8 slice");
            if asset_type == "stylesheet" {
              _webview.inject_css(asset_str)
            } else {
              _webview.eval(asset_str)
            }
          })
          .map_err(|err| err.into())
          .map(|_| r#""Asset loaded successfully""#.to_string())
      }
    },
    callback,
    error,
  );

  Ok(())
}

#[cfg(any(feature = "all-api", feature = "notification"))]
fn notification<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: cmd::NotificationOptions,
  callback: String,
  error: String,
  #[cfg(windows)] config: &Config,
) -> crate::Result<()> {
  #[cfg(windows)]
  let identifier = config.tauri.bundle.identifier.clone();
  crate::execute_promise(
    webview,
    move || {
      let mut notification = notify_rust::Notification::new();
      notification.body(&options.body);
      if let Some(title) = options.title {
        notification.summary(&title);
      }
      if let Some(icon) = options.icon {
        notification.icon(&icon);
      }
      #[cfg(windows)]
      {
        let exe = std::env::current_exe()?;
        let exe_dir = exe.parent().expect("failed to get exe directory");
        let curr_dir = exe_dir.display().to_string();
        // set the notification's System.AppUserModel.ID only when running the installed app
        if !(curr_dir.ends_with(format!("{S}target{S}debug", S = MAIN_SEPARATOR).as_str())
          || curr_dir.ends_with(format!("{S}target{S}release", S = MAIN_SEPARATOR).as_str()))
        {
          notification.app_id(&identifier);
        }
      }
      notification
        .show()
        .map_err(|e| anyhow::anyhow!(r#""{}""#, e.to_string()))?;
      Ok("".to_string())
    },
    callback,
    error,
  );
  Ok(())
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  #[test]
  // test to see if check init produces a string or not.
  fn check_init() {
    if cfg!(not(any(feature = "all-api", feature = "event"))) {
      let res = super::init();
      match res {
        Ok(s) => assert_eq!(s, ""),
        Err(_) => assert!(false),
      }
    } else if cfg!(any(feature = "all-api", feature = "event")) {
      let res = super::init();
      match res {
        Ok(s) => assert!(s.contains("window.tauri.promisified")),
        Err(_) => assert!(false),
      }
    }
  }

  // check the listen_fn for various usecases.
  proptest! {
    #[cfg(any(feature = "all-api", feature = "event"))]
    #[test]
    fn check_listen_fn(event in "", handler in "", once in proptest::bool::ANY) {
      let res = super::listen_fn(event, handler, once);
      match res {
        Ok(_) => assert!(true),
        Err(_) => assert!(false)
      }
    }
  }

  // Test the open func to see if proper uris can be opened by the browser.
  proptest! {
    #[cfg(any(feature = "all-api", feature = "open"))]
    #[test]
    fn check_open(uri in r"(http://)([\\w\\d\\.]+([\\w]{2,6})?)") {
      let res = super::open_fn(uri);
      match res {
        Ok(_) => assert!(true),
        Err(_) => assert!(false),
    }
  }
  }
}
