//! CEF-only extension: native interception of Web Notifications.
//!
//! `cef-helper` (the renderer subprocess) installs a `RenderProcessHandler`
//! that swaps `window.Notification` and `ServiceWorkerRegistration.prototype.
//! showNotification` for native V8 functions. Those functions send a
//! `ProcessMessage` named `"openhuman.notify"` over to the browser process,
//! where `BrowserClient::on_process_message_received` decodes it and calls
//! whatever handler the embedder has registered for the originating browser id.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSource {
  /// Page called `new Notification(...)`.
  Window,
  /// Service worker called `registration.showNotification(...)`.
  ServiceWorker,
}

#[derive(Debug, Clone)]
pub struct NotificationPayload {
  pub source: NotificationSource,
  pub title: String,
  pub body: Option<String>,
  pub icon: Option<String>,
  pub tag: Option<String>,
  pub silent: bool,
  /// `frame.url()` at the time of the call. Useful for origin-based routing.
  pub origin: String,
}

pub type NotificationHandler = Arc<dyn Fn(NotificationPayload) + Send + Sync>;

pub(crate) const IPC_MESSAGE_NAME: &str = "openhuman.notify";

static REGISTRY: OnceLock<Mutex<HashMap<i32, NotificationHandler>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<i32, NotificationHandler>> {
  REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a handler keyed by CEF browser id (`cef::Browser::identifier()`).
/// Replaces any previously registered handler for the same browser.
pub fn register<F>(browser_id: i32, handler: F)
where
  F: Fn(NotificationPayload) + Send + Sync + 'static,
{
  registry()
    .lock()
    .unwrap()
    .insert(browser_id, Arc::new(handler));
}

pub fn unregister(browser_id: i32) {
  registry().lock().unwrap().remove(&browser_id);
}

/// Called by `BrowserClient` when a `"openhuman.notify"` IPC arrives.
pub(crate) fn dispatch(browser_id: i32, payload: NotificationPayload) {
  let handler = registry().lock().unwrap().get(&browser_id).cloned();
  if let Some(h) = handler {
    h(payload);
  }
}
