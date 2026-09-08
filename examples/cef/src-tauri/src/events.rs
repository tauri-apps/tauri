// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Forwards what the native CEF observers see to the frontend.
//!
//! Every CEF observer in this example — console messages, frame lifecycle
//! events, DevTools protocol traffic, popup requests, permission requests —
//! runs **synchronously on CEF's UI thread**, the thread that also drives the
//! browser. A handler that blocks there blocks the browser, so none of them may
//! wait on the Tauri event loop, and `Emitter::emit` is not something to call
//! from one: it reaches into the runtime to evaluate a script in the webview.
//!
//! So the handlers only push into a bounded channel with [`Sender::try_send`],
//! which never blocks and drops the event when the buffer is full, and a task on
//! Tauri's async runtime does the emitting.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_runtime_cef::{ConsoleMessage, ConsoleMessageLevel, DevToolsProtocol, FrameEvent};

/// The single event the frontend listens for. Each payload names its own `kind`.
const EVENT: &str = "cef://event";

/// How many observations may pile up before the oldest are dropped. A page in a
/// loop calling `console.log` produces them faster than the frontend can render.
const BUFFER: usize = 512;

/// One thing a native CEF observer saw, as the frontend consumes it.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CefEvent {
  /// A message the renderer wrote to the JavaScript console, observed without
  /// DevTools having to be open.
  #[serde(rename_all = "camelCase")]
  Console {
    window: String,
    level: String,
    message: String,
    source: String,
    line: i32,
  },
  /// A native frame lifecycle notification, child frames included.
  #[serde(rename_all = "camelCase")]
  Frame {
    window: String,
    browser_id: i32,
    frame_id: String,
    is_main: bool,
    event: String,
    url: Option<String>,
  },
  /// A Chrome DevTools Protocol event or method result.
  #[serde(rename_all = "camelCase")]
  DevTools {
    event: String,
    method: Option<String>,
    message_id: Option<i32>,
    success: Option<bool>,
    payload: String,
  },
  /// A `window.open()` the runtime asked the application about.
  #[serde(rename_all = "camelCase")]
  Popup {
    url: String,
    opener_source: Option<String>,
    label: String,
  },
  /// A permission request answered by the application's handler.
  #[serde(rename_all = "camelCase")]
  Permission {
    permission: String,
    response: String,
    note: String,
  },
  /// A deep link delivered to this already running instance.
  #[serde(rename_all = "camelCase")]
  DeepLink { urls: Vec<String> },
}

/// Hands observations from CEF's UI thread to the frontend.
#[derive(Clone)]
pub struct EventSink(tauri::async_runtime::Sender<CefEvent>);

impl EventSink {
  pub fn new(app: &AppHandle) -> Self {
    let (sender, mut receiver) = tauri::async_runtime::channel(BUFFER);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      while let Some(event) = receiver.recv().await {
        let _ = app.emit(EVENT, event);
      }
    });
    Self(sender)
  }

  /// Queues one observation. Never blocks: an observation that does not fit the
  /// buffer is dropped rather than stalling the browser's UI thread.
  pub fn send(&self, event: CefEvent) {
    let _ = self.0.try_send(event);
  }

  pub fn console(&self, window: &str, message: ConsoleMessage) {
    self.send(CefEvent::Console {
      window: window.to_string(),
      level: match message.level {
        ConsoleMessageLevel::Verbose => "verbose",
        ConsoleMessageLevel::Info => "info",
        ConsoleMessageLevel::Warning => "warning",
        ConsoleMessageLevel::Error => "error",
        ConsoleMessageLevel::Fatal => "fatal",
        // `ConsoleMessageLevel` is `#[non_exhaustive]`: a severity a later build
        // of the runtime names arrives here.
        _ => "other",
      }
      .to_string(),
      message: message.message,
      source: message.source,
      line: message.line,
    });
  }

  pub fn frame(&self, window: &str, event: FrameEvent) {
    use tauri_runtime_cef::FrameEventKind::*;

    // Only the navigation phases carry a URL; the rest are lifecycle-only.
    let (name, url) = match &event.kind {
      Created => ("created".to_string(), None),
      Attached => ("attached".to_string(), None),
      Detached => ("detached".to_string(), None),
      Destroyed => ("destroyed".to_string(), None),
      NavigationStarted { url } => ("navigationStarted".to_string(), Some(url.to_string())),
      DocumentCommitted { url } => ("documentCommitted".to_string(), Some(url.to_string())),
      NavigationFailed { url } => ("navigationFailed".to_string(), Some(url.to_string())),
      AddressChanged { url } => ("addressChanged".to_string(), Some(url.to_string())),
      MainFrameChanged => ("mainFrameChanged".to_string(), None),
      LoadingStateChanged { is_loading } => (
        if *is_loading {
          "loadingStarted".to_string()
        } else {
          "loadingFinished".to_string()
        },
        None,
      ),
      RendererTerminated => ("rendererTerminated".to_string(), None),
      // `FrameEventKind` is `#[non_exhaustive]`, so a phase a later build of the
      // runtime reports still shows up, spelled the way the runtime debugs it.
      kind => (format!("{kind:?}"), None),
    };

    self.send(CefEvent::Frame {
      window: window.to_string(),
      browser_id: event.browser_id,
      frame_id: event.frame_id,
      is_main: event.is_main,
      event: name,
      url,
    });
  }

  pub fn dev_tools(&self, protocol: DevToolsProtocol) {
    let event = match protocol {
      // The raw message is delivered for every notification, immediately before
      // the classified one, so surfacing it too would double every line.
      DevToolsProtocol::Message(_) => return,
      DevToolsProtocol::Event { method, params } => CefEvent::DevTools {
        event: "event".into(),
        method: Some(method),
        message_id: None,
        success: None,
        payload: String::from_utf8_lossy(&params).into_owned(),
      },
      DevToolsProtocol::MethodResult {
        message_id,
        success,
        result,
      } => CefEvent::DevTools {
        event: "result".into(),
        method: None,
        message_id: Some(message_id),
        success: Some(success),
        payload: String::from_utf8_lossy(&result).into_owned(),
      },
    };
    self.send(event);
  }
}
