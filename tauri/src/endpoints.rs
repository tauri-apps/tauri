mod cmd;

use web_view::WebView;

#[allow(unused_variables)]
pub(crate) fn handle<T: 'static>(webview: &mut WebView<'_, T>, arg: &str) -> crate::Result<bool> {
  use cmd::Cmd::*;
  match serde_json::from_str(arg) {
    Err(_) => Ok(false),
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
        #[cfg(any(feature = "all-api", feature = "readTextFile"))]
        ReadTextFile {
          path,
          callback,
          error,
        } => {
          crate::file_system::read_text_file(webview, path, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "readBinaryFile"))]
        ReadBinaryFile {
          path,
          callback,
          error,
        } => {
          crate::file_system::read_binary_file(webview, path, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "writeFile"))]
        WriteFile {
          file,
          contents,
          callback,
          error,
        } => {
          crate::file_system::write_file(webview, file, contents, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "listDirs"))]
        ListDirs {
          path,
          callback,
          error,
        } => {
          crate::file_system::list_dirs(webview, path, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "listFiles"))]
        ListFiles {
          path,
          callback,
          error,
        } => {
          crate::file_system::list(webview, path, callback, error);
        }
        #[cfg(any(feature = "all-api", feature = "setTitle"))]
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
          crate::salt::validate(webview, salt, callback, error);
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
        #[cfg(not(any(feature = "dev-server", feature = "embedded-server")))]
        LoadAsset {
          asset,
          asset_type,
          callback,
          error,
        } => {
          load_asset(webview, asset, asset_type, callback, error)?;
        }
      }
      Ok(true)
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

#[cfg(not(any(feature = "dev-server", feature = "embedded-server")))]
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
      let read_asset = crate::assets::ASSETS.get(&format!(
        "{}{}{}",
        env!("TAURI_DIST_DIR"),
        if asset.starts_with("/") { "" } else { "/" },
        asset
      ));
      if read_asset.is_err() {
        return Err(format!("Asset '{}' not found", asset).into());
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
          "`data:image/{};base64,{}`",
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
          .map_err(|err| crate::ErrorKind::Promise(format!("`{}`", err)).into())
          .map(|_| r#""Asset loaded successfully""#.to_string())
      }
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
