use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Listen to an event.
  Listen { event: String, handler: String },
  /// Unlisten to an event.
  #[serde(rename_all = "camelCase")]
  Unlisten { event_id: u64 },
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
  pub fn run<A: crate::ApplicationExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<A>,
  ) -> crate::Result<InvokeResponse> {
    match self {
      Self::Listen { event, handler } => {
        let event_id = rand::random();
        webview_manager
          .current_webview()?
          .eval(&listen_js(event, event_id, handler))?;
        Ok(event_id.into())
      }
      Self::Unlisten { event_id } => {
        webview_manager
          .current_webview()?
          .eval(&unlisten_js(event_id))?;
        Ok(().into())
      }
      Self::Emit {
        event,
        window_label,
        payload,
      } => {
        if let Some(label) = window_label {
          let dispatcher = webview_manager.get_webview(&label)?;
          // dispatch the event to Rust listeners
          dispatcher.on_event(event.to_string(), payload.clone());
          // dispatch the event to JS listeners
          dispatcher.emit(event, payload)?;
        } else {
          // dispatch the event to Rust listeners
          webview_manager.on_event(event.to_string(), payload.clone());
          // dispatch the event to JS listeners
          webview_manager.emit(event, payload)?;
        }
        Ok(().into())
      }
    }
  }
}

pub fn unlisten_js(event_id: u64) -> String {
  format!(
    "
      for (var event in (window['{listeners}'] || {{}})) {{
        var listeners = (window['{listeners}'] || {{}})[event]
        if (listeners) {{
          window['{listeners}'][event] = window['{listeners}'][event].filter(function (e) {{ e.id !== {event_id} }})
        }}
      }}
    ",
    listeners = crate::app::event::event_listeners_object_name(),
    event_id = event_id,
  )
}

pub fn listen_js(event: String, event_id: u64, handler: String) -> String {
  format!(
    "if (window['{listeners}'] === void 0) {{
      window['{listeners}'] = {{}}
    }}
    if (window['{listeners}']['{event}'] === void 0) {{
      window['{listeners}']['{event}'] = []
    }}
    window['{listeners}']['{event}'].push({{
      id: {event_id},
      handler: window['{handler}']
    }});

    for (let i = 0; i < (window['{queue}'] || []).length; i++) {{
      const e = window['{queue}'][i];
      window['{emit}'](e.eventData, e.salt, true)
    }}
  ",
    listeners = crate::app::event::event_listeners_object_name(),
    queue = crate::app::event::event_queue_object_name(),
    emit = crate::app::event::emit_function_name(),
    event = event,
    event_id = event_id,
    handler = handler
  )
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  // check the listen_js for various usecases.
  proptest! {
    #[test]
    fn check_listen_js(event in "", id in proptest::bits::u64::ANY, handler in "") {
      super::listen_js(event, id, handler);
    }
  }
}
