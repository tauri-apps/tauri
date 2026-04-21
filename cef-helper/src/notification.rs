//! Renderer-side Web Notification interception.
//!
//! `NotifyRenderProcessHandler::on_context_created` replaces `window.Notification`
//! and `ServiceWorkerRegistration.prototype.showNotification` with native V8
//! functions backed by `NotifyV8Handler`. Those functions encode the call as a
//! CEF `ProcessMessage` and forward it to the browser process, where
//! `tauri-runtime-cef::notification` dispatches to the registered callback.

use cef::{rc::*, *};

/// IPC message name — must match `tauri_runtime_cef::notification::IPC_MESSAGE_NAME`.
pub const IPC_NAME: &str = "openhuman.notify";

#[derive(Clone, Copy)]
pub enum NotificationSource {
  Window = 0,
  ServiceWorker = 1,
}

// ---------------------------------------------------------------------------
// App wrapper — returns our render-process handler to CEF.
// ---------------------------------------------------------------------------

wrap_app! {
  pub struct NotifyApp;

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(NotifyRenderProcessHandler::new())
    }
  }
}

// ---------------------------------------------------------------------------
// RenderProcessHandler — installs V8 shims on every new JS context.
// ---------------------------------------------------------------------------

wrap_render_process_handler! {
  struct NotifyRenderProcessHandler;

  impl RenderProcessHandler {
    fn on_context_created(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      context: Option<&mut V8Context>,
    ) {
      let (Some(frame), Some(context)) = (frame, context) else { return; };

      // Capture the frame URL early as the origin string.
      let origin = CefString::from(&frame.url()).to_string();

      let Some(global) = context.global() else { return; };

      // --- window.Notification shim ---
      install_notification_shim(&global, &origin, NotificationSource::Window);

      // --- ServiceWorkerRegistration.prototype.showNotification shim ---
      // This global may not exist in every frame (e.g. normal page contexts
      // don't expose SWRegistration directly). Silently skip if absent.
      install_sw_shim(&global, &origin);

      // --- navigator.permissions.query shim ---
      // Slack (and other apps) check navigator.permissions.query({ name:
      // 'notifications' }) before calling Notification.requestPermission().
      // CEF's built-in Permissions API returns "prompt" until the native
      // permission dialog is accepted, so the check disagrees with our
      // Notification.permission = "granted" shim above. Patch query() to
      // return "granted" for notifications while delegating everything else
      // to the original.
      install_permissions_query_shim(context);
    }
  }
}

/// Replace `global.Notification` with a native function that sends an IPC.
fn install_notification_shim(global: &V8Value, origin: &str, source: NotificationSource) {
  let mut handler = NotifyV8Handler::new(source, origin.to_owned());
  let Some(mut shim) =
    v8_value_create_function(Some(&CefString::from("Notification")), Some(&mut handler))
  else {
    return;
  };

  // Notification.permission = "granted" — pages that check this won't prompt.
  if let Some(mut perm) = v8_value_create_string(Some(&CefString::from("granted"))) {
    shim.set_value_bykey(
      Some(&CefString::from("permission")),
      Some(&mut perm),
      V8Propertyattribute::default(),
    );
  }

  // Notification.requestPermission = fn() => Promise.resolve("granted")
  let mut rp_handler = ResolveGrantedHandler::new();
  if let Some(mut rp_fn) = v8_value_create_function(
    Some(&CefString::from("requestPermission")),
    Some(&mut rp_handler),
  ) {
    shim.set_value_bykey(
      Some(&CefString::from("requestPermission")),
      Some(&mut rp_fn),
      V8Propertyattribute::default(),
    );
  }

  global.set_value_bykey(
    Some(&CefString::from("Notification")),
    Some(&mut shim),
    V8Propertyattribute::default(),
  );
}

/// Replace `ServiceWorkerRegistration.prototype.showNotification` if available.
fn install_sw_shim(global: &V8Value, origin: &str) {
  // Guard: if this context doesn't expose ServiceWorkerRegistration, skip.
  let Some(sw_reg) = global.value_bykey(Some(&CefString::from("ServiceWorkerRegistration")))
  else {
    return;
  };
  if sw_reg.is_object() == 0 {
    return;
  }
  let Some(proto) = sw_reg.value_bykey(Some(&CefString::from("prototype"))) else {
    return;
  };
  if proto.is_object() == 0 {
    return;
  }

  let mut handler = NotifyV8Handler::new(NotificationSource::ServiceWorker, origin.to_owned());
  let Some(mut shim) =
    v8_value_create_function(Some(&CefString::from("showNotification")), Some(&mut handler))
  else {
    return;
  };

  proto.set_value_bykey(
    Some(&CefString::from("showNotification")),
    Some(&mut shim),
    V8Propertyattribute::default(),
  );
}

// ---------------------------------------------------------------------------
// V8Handler — packs notification args into a ProcessMessage and sends it.
// ---------------------------------------------------------------------------

wrap_v8_handler! {
  struct NotifyV8Handler {
    source: NotificationSource,
    origin: String,
  }

  impl V8Handler {
    fn execute(
      &self,
      _name: Option<&CefString>,
      _object: Option<&mut V8Value>,
      arguments: Option<&[Option<V8Value>]>,
      retval: Option<&mut Option<V8Value>>,
      _exception: Option<&mut CefString>,
    ) -> ::std::os::raw::c_int {
      let args = arguments.unwrap_or(&[]);

      // Title is the first argument; treat missing/non-string as empty.
      let title = args
        .first()
        .and_then(|v| v.as_ref())
        .filter(|v| v.is_string() != 0)
        .map(|v| CefString::from(&v.string_value()).to_string())
        .unwrap_or_default();

      // Second argument is the options object (optional).
      let opts = args.get(1).and_then(|v| v.as_ref()).filter(|v| v.is_object() != 0);

      let body = read_opt_str(opts, "body");
      let icon = read_opt_str(opts, "icon");
      let tag = read_opt_str(opts, "tag");
      let silent = opts
        .and_then(|o| o.value_bykey(Some(&CefString::from("silent"))))
        .map(|v| v.bool_value() != 0)
        .unwrap_or(false);

      // Build and send the IPC.
      if let Some(mut msg) = process_message_create(Some(&CefString::from(IPC_NAME))) {
        if let Some(list) = msg.argument_list() {
          list.set_size(7);
          list.set_int(0, self.source as i32);
          list.set_string(1, Some(&CefString::from(title.as_str())));
          list.set_string(2, Some(&CefString::from(body.as_deref().unwrap_or(""))));
          list.set_string(3, Some(&CefString::from(icon.as_deref().unwrap_or(""))));
          list.set_string(4, Some(&CefString::from(tag.as_deref().unwrap_or(""))));
          list.set_int(5, if silent { 1 } else { 0 });
          list.set_string(6, Some(&CefString::from(self.origin.as_str())));
        }
        if let Some(ctx) = v8_context_get_current_context() {
          if let Some(frame) = ctx.frame() {
            frame.send_process_message(ProcessId::BROWSER, Some(&mut msg));
          }
        }
      }

      // Return an empty object stub for `new Notification(...)` so that callers
      // that do `n.onclick = fn` don't NPE.
      match self.source {
        NotificationSource::Window => {
          if let Some(retval) = retval {
            *retval = v8_value_create_object(None, None);
          }
        }
        NotificationSource::ServiceWorker => {
          // SW callers expect a Promise; resolve it immediately with undefined
          // because we've already forwarded the notification.
          if let Some(retval) = retval {
            if let Some(promise) = v8_value_create_promise() {
              if let Some(mut undef) = v8_value_create_undefined() {
                promise.resolve_promise(Some(&mut undef));
              }
              *retval = Some(promise);
            }
          }
        }
      }
      1
    }
  }
}

// ---------------------------------------------------------------------------
// Helper: `Notification.requestPermission()` → Promise.resolve("granted")
// ---------------------------------------------------------------------------

wrap_v8_handler! {
  struct ResolveGrantedHandler;

  impl V8Handler {
    fn execute(
      &self,
      _name: Option<&CefString>,
      _object: Option<&mut V8Value>,
      _arguments: Option<&[Option<V8Value>]>,
      retval: Option<&mut Option<V8Value>>,
      _exception: Option<&mut CefString>,
    ) -> ::std::os::raw::c_int {
      if let Some(retval) = retval {
        if let Some(promise) = v8_value_create_promise() {
          if let Some(mut granted) = v8_value_create_string(Some(&CefString::from("granted"))) {
            promise.resolve_promise(Some(&mut granted));
          }
          *retval = Some(promise);
        }
      }
      1
    }
  }
}

// ---------------------------------------------------------------------------
// navigator.permissions.query shim — returns "granted" for notifications.
// ---------------------------------------------------------------------------

fn install_permissions_query_shim(context: &V8Context) {
  // Runs as a self-executing function so `_orig` is captured in closure scope
  // and non-notification queries are transparently forwarded.
  let js = r#"(function() {
    try {
      var perms = navigator && navigator.permissions;
      if (!perms || typeof perms.query !== 'function') return;
      var _orig = perms.query.bind(perms);
      perms.query = function(descriptor) {
        if (descriptor && descriptor.name === 'notifications') {
          return Promise.resolve({ state: 'granted', onchange: null });
        }
        return _orig(descriptor);
      };
    } catch (_) {}
  })();"#;

  let mut retval: Option<V8Value> = None;
  let mut exception: Option<V8Exception> = None;
  context.eval(
    Some(&CefString::from(js)),
    Some(&CefString::from("openhuman://notification-shim")),
    0,
    Some(&mut retval),
    Some(&mut exception),
  );
}

// ---------------------------------------------------------------------------
// Utility: read a string property from a V8 object, returning None if absent.
// ---------------------------------------------------------------------------

fn read_opt_str(obj: Option<&V8Value>, key: &str) -> Option<String> {
  let v = obj?.value_bykey(Some(&CefString::from(key)))?;
  if v.is_string() != 0 {
    Some(CefString::from(&v.string_value()).to_string())
  } else {
    None
  }
}
