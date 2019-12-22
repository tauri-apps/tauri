mod cmd;

use web_view::WebView;

#[cfg(not(any(feature = "dev-server", feature = "embedded-server")))]
include!(concat!(env!("OUT_DIR"), "/data.rs"));

#[allow(unused_variables)]
pub fn handler<T: 'static>(webview: &mut WebView<'_, T>, arg: &str) -> bool {
  use cmd::Cmd::*;
  match serde_json::from_str(arg) {
    Err(_) => false,
    Ok(command) => {
      match command {
        Init {} => {
          webview
            .handle()
            .dispatch(move |_webview| {
              _webview
                .eval(&format!(
                  "window['{queue}'] = [];
                  window['{fn}'] = function (payload, salt, ignoreQueue) {{
                     window.tauri.promisified({{
                      cmd: 'validateSalt',
                      salt: salt
                    }}).then(function () {{
                      const listeners = (window['{listeners}'] && window['{listeners}'][payload.type]) || []
                      if (!ignoreQueue && listeners.length === 0) {{
                        window['{queue}'].push({{
                          payload: payload,
                          salt: salt
                        }})
                      }}

                      for (let i = listeners.length - 1; i >= 0; i--) {{
                        const listener = listeners[i]
                        if (listener.once)
                          listeners.splice(i, 1)
                        listener.handler(payload)
                      }}
                    }})
                  }}",
                  fn = crate::event::emit_function_name(),
                  listeners = crate::event::event_listeners_object_name(),
                  queue = crate::event::event_queue_object_name()
                ))
                .unwrap();

                Ok(())
            })
            .unwrap();
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
          webview.set_title(&title).unwrap();
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
          crate::spawn(move || {
            webbrowser::open(&uri).unwrap();
          });
        }

        ValidateSalt {
          salt,
          callback,
          error,
        } => {
          crate::salt::validate(webview, salt, callback, error);
        }
        Listen {
          event,
          handler,
          once,
        } => {
          webview
            .eval(&format!(
              "
                if (window['{listeners}'] === void 0) {{
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
            .unwrap();
        }
        #[cfg(any(feature = "all-api", feature = "answer"))]
        Emit { event, payload } => {
          crate::event::on_event(event, payload);
        }
        LoadAsset {
          asset,
          asset_type,
          callback,
          error,
        } => {
          if cfg!(not(any(
            feature = "dev-server",
            feature = "embedded-server"
          ))) {
            let handle = webview.handle();
            crate::execute_promise(
              webview,
              move || {
                let read_asset = ASSETS.get(&format!(
                  "{}{}{}",
                  env!("TAURI_DIST_DIR"),
                  if asset.starts_with("/") { "" } else { "/" },
                  asset
                ));
                if read_asset.is_err() {
                  return Err(r#""Asset not found""#.to_string());
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
                    base64::encode(&read_asset.unwrap().into_owned())
                  ))
                } else {
                  handle
                    .dispatch(move |_webview| {
                      _webview
                        .eval(&std::str::from_utf8(&read_asset.unwrap().into_owned()).unwrap())
                    })
                    .map_err(|err| format!("`{}`", err))
                    .map(|_| r#""Asset loaded successfully""#.to_string())
                }
              },
              callback,
              error,
            );
          }
        }
      }
      true
    }
  }
}
