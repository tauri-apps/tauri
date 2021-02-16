use serde::Deserialize;
use serde_json::Value as JsonValue;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Listen to an event.
  Listen {
    event: String,
    handler: String,
    once: bool,
  },
  /// Emit an event to the webview associated with the given window.
  /// If the window_label is omitted, the event will be triggered on all listeners.
  #[serde(rename_all = "camelCase")]
  Emit {
    event: String,
    window_label: Option<String>,
    payload: Option<String>,
  },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<JsonValue> {
    #[cfg(not(event))]
    return Err(crate::Error::ApiNotAllowlisted("event".to_string()));
    #[cfg(event)]
    match self {
      Self::Listen {
        event,
        handler,
        once,
      } => {
        let js_string = listen_fn(event, handler, once)?;
        webview_manager.current_webview().await?.eval(&js_string)?;
        Ok(JsonValue::Null)
      }
      Self::Emit {
        event,
        window_label,
        payload,
      } => {
        if let Some(label) = window_label {
          let dispatcher = webview_manager.get_webview(&label).await?;
          // dispatch the event to Rust listeners
          dispatcher.on_event(event.to_string(), payload.clone());
          // dispatch the event to JS listeners
          dispatcher.emit(event, payload)?;
        } else {
          // dispatch the event to Rust listeners
          webview_manager.on_event(event.to_string(), payload.clone());
          // dispatch the event to JS listeners
          webview_manager.emit(event, payload).await?;
        }
        Ok(JsonValue::Null)
      }
    }
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

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  // check the listen_fn for various usecases.
  proptest! {
    #[cfg(event)]
    #[test]
    fn check_listen_fn(event in "", handler in "", once in proptest::bool::ANY) {
      super::listen_fn(event, handler, once).expect("listen_fn failed");
    }
  }
}
