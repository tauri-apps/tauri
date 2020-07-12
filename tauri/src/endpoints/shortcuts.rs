use super::cmd::ShortcutHandler;
use tauri_api::shortcuts::ShortcutManager;
use web_view::WebView;

pub fn add_shortcuts<T: 'static>(
  webview: &mut WebView<'_, T>,
  shortcut_handlers: Vec<ShortcutHandler>,
) {
  let callback_handle = webview.handle();
  let error_handle = webview.handle();
  let callback_handle = std::sync::Arc::new(callback_handle.clone());

  crate::spawn(move || {
    let mut manager = ShortcutManager::new();
    for shortcut_handler in shortcut_handlers {
      let callback_handle = callback_handle.clone();

      let callback_identifier = shortcut_handler.callback.clone();
      let callback_identifier = std::sync::Arc::new(callback_identifier);

      manager.register_shortcut(
        shortcut_handler.shortcut.clone(),
        move || {
          let callback_handle = callback_handle.clone();
          let callback_string =
            tauri_api::rpc::format_callback(callback_identifier.to_string(), "void 0".to_string());
          callback_handle
            .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
            .expect("Failed to dispatch shortcut callback");
        },
        |e| {
          if let Some(error) = &shortcut_handler.error {
            let callback_string = tauri_api::rpc::format_callback(error.to_string(), e);
            error_handle
              .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
              .expect("Failed to dispatch shortcut error");
          }
        },
      );
    }
    manager.listen();
  });

  /*let handle = webview.handle();
  let handle = std::sync::Arc::new(handle);
  let j = shortcut_handlers.clone();
  crate::spawn(move || {
    let handlers: Vec<ShortcutHandler> = j.iter().map(|handler| {
      let handle = handle.clone();
      ShortcutHandler {
        shortcut: handler.shortcut.clone(),
        callback: Box::new(move || {
          let callback_string = tauri_api::rpc::format_callback(handler.callback.clone(), "void 0".to_string());
          handle
            .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
            .expect("Failed to dispatch shortcut callback");
        }),
        error: Box::new(|e| {
          /*lif let Some(error) = &handler.error {
            let callback_string = tauri_api::rpc::format_callback(error, e);
            handle
              .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
              .expect("Failed to dispatch shortcut callback");
          }*/
        })
      }
    }).collect();
  });*/
}
