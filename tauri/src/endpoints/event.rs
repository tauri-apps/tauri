use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The event listen API.
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  /// The event emit API.
  Emit {
    event: String,
    payload: Option<String>,
  },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<()> {
    match self {
      Self::Listen {
        event,
        handler,
        once,
      } => {
        #[cfg(event)]
        {
          let js_string = listen_fn(event, handler, once)?;
          webview_manager.current_webview()?.eval(&js_string);
        }
        #[cfg(not(event))]
        throw_allowlist_error(webview_manager, "event");
      }
      Self::Emit { event, payload } => {
        // TODO emit to optional window
        #[cfg(event)]
        webview_manager.current_webview()?.on_event(event, payload);
        #[cfg(not(event))]
        throw_allowlist_error(webview_manager, "event");
      }
    }
    Ok(())
  }
}

#[cfg(event)]
pub fn listen_fn(event: String, handler: String, once: bool) -> crate::Result<String> {
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
    listeners = crate::app::event::event_listeners_object_name(),
    queue = crate::app::event::event_queue_object_name(),
    emit = crate::app::event::emit_function_name(),
    evt = event,
    handler = handler,
    once_flag = if once { "true" } else { "false" }
  ))
}
