pub fn init() -> crate::Result<String> {
  #[cfg(not(event))]
  return Ok(String::from(""));
  #[cfg(event)]
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
        window.__TAURI__.promisified({{
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
