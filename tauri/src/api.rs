mod cmd;

use web_view::WebView;

#[allow(unused_variables)]
pub fn handler<T: 'static>(webview: &mut WebView<'_, T>, arg: &str) -> bool {
  use cmd::Cmd::*;
  match serde_json::from_str(arg) {
    Err(_) => false,
    Ok(command) => {
      match command {
        Init {} => {
          #[cfg(feature = "dev-server")]
          let handler_call = "window[listener.handler](payload)";
          #[cfg(not(feature = "dev-server"))]
          let handler_call = "window.frames[0].postMessage({ type: 'tauri-callback', payload: payload, callback: listener.handler }, '*')";
          webview
            .handle()
            .dispatch(move |_webview| {
              _webview
                .eval(&format!(
                  "window['{queue}'] = [];
                  window['{fn}'] = function (payload, ignoreQueue) {{
                    const listeners = (window['{listeners}'] && window['{listeners}'][payload.type]) || []
                    if (!ignoreQueue && listeners.length === 0) {{
                      window['{queue}'].push({{
                        payload: payload
                      }})
                    }}

                    for (let i = listeners.length - 1; i >= 0; i--) {{
                      const listener = listeners[i]
                      if (listener.once)
                        listeners.splice(i, 1)
                      {handler_call}
                    }}
                  }}",
                  fn = crate::event::emit_function_name(),
                  listeners = crate::event::event_listeners_object_name(),
                  queue = crate::event::event_queue_object_name(),
                  handler_call = handler_call
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
        #[cfg(any(feature = "all-api", feature = "open"))]
        Open { uri } => {
          super::spawn(move || {
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
                  handler: '{handler}',
                  once: {once_flag}
                }});

                for (let i = 0; i < (window['{queue}'] || []).length; i++) {{
                  const e = window['{queue}'][i];
                  window['{emit}'](e.payload, true)
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
        },
        LoadAsset { asset, callback, error } => {
          println!("loading asset named {}", asset);
          let handle = webview.handle();
          crate::execute_promise(
            webview,
            move || {
              handle.dispatch(move |_webview| {
                let asset_str = std::fs::read_to_string(format!("{}/{}", env!("TAURI_DIST_DIR"), asset));
                if asset_str.is_err() {
                  Err(web_view::Error::Custom(Box::new("Asset not found")))
                } else {
                  let asset_script = asset_str.unwrap().replace("\"", "\\\"");
                  /*println!(
                    r#"window.frames[0].postMessage({{ type: "tauri-asset", payload: "{}" }}, '*')"#, 
                    asset_script
                  );
                  _webview.eval("window.a = 5")*/
                  _webview.eval(
                    &format!(
                      r#"window.frames[0].postMessage({{ type: "tauri-asset", payload: "{}" }}, '*')"#, 
                      asset_script
                    )
                  )
                }
              })
                .map_err(|err| format!("`{}`", err))
                .map(|_| r#""Asset load successfully""#.to_string())
            },
            callback,
            error
          );
        }
      }
      true
    }
  }
}
