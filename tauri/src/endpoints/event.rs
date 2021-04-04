use crate::{
  endpoints::InvokeResponse,
  runtime::{sealed::ManagerPrivate, window::Window, Manager, Params},
};
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
  pub fn run<M: Params>(self, window: Window<M>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Listen { event, handler } => {
        let event_id = rand::random();
        window.eval(&listen_js(&window, event, event_id, handler))?;
        Ok(event_id.into())
      }
      Self::Unlisten { event_id } => {
        window.eval(&unlisten_js(&window, event_id))?;
        Ok(().into())
      }
      Self::Emit {
        event,
        window_label,
        payload,
      } => {
        let e: M::Event = event
          .parse()
          .unwrap_or_else(|_| panic!("todo: invalid event str"));

        let window_label: Option<M::Label> = window_label.map(|l| {
          l.parse()
            .unwrap_or_else(|_| panic!("todo: invalid window label"))
        });

        // dispatch the event to Rust listeners
        window.trigger(e.clone(), payload.clone());

        if let Some(target) = window_label {
          window.emit_to(&target, e, payload)?;
        } else {
          window.emit_all(e, payload)?;
        }
        Ok(().into())
      }
    }
  }
}

pub fn unlisten_js<M: Params>(window: &Window<M>, event_id: u64) -> String {
  format!(
    "
      for (var event in (window['{listeners}'] || {{}})) {{
        var listeners = (window['{listeners}'] || {{}})[event]
        if (listeners) {{
          window['{listeners}'][event] = window['{listeners}'][event].filter(function (e) {{ e.id !== {event_id} }})
        }}
      }}
    ",
    listeners = window.manager().event_listeners_object_name(),
    event_id = event_id,
  )
}

pub fn listen_js<M: Params>(
  window: &Window<M>,
  event: String,
  event_id: u64,
  handler: String,
) -> String {
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
    listeners = window.manager().event_listeners_object_name(),
    queue = window.manager().event_queue_object_name(),
    emit = window.manager().event_emit_function_name(),
    event = event,
    event_id = event_id,
    handler = handler
  )
}
