// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

/// One browser-process notification about a native CEF frame.
///
/// Notifications run synchronously on CEF's UI thread. Handlers must return
/// promptly and must not call APIs that wait for the event loop. Unlike Tauri's
/// portable navigation callbacks, these notifications include child frames.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct FrameEvent {
  /// Native browser identity. A popup has a different identity from its opener.
  pub browser_id: i32,
  /// CEF's opaque identifier for this native frame lifetime.
  pub frame_id: String,
  /// Whether CEF identifies this as the main frame at callback time.
  pub is_main: bool,
  /// Native lifecycle phase.
  pub kind: FrameEventKind,
}

/// Native lifecycle phases reported for every frame.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum FrameEventKind {
  /// A frame object exists, but may not yet have a renderer connection.
  Created,
  /// Commands can be routed to the renderer. Reattachment is also reported.
  Attached,
  /// The frame can no longer route commands to its renderer.
  Detached,
  /// The native frame object is being destroyed.
  Destroyed,
  /// A navigation was admitted by the existing navigation policy, before commit.
  NavigationStarted { url: url::Url },
  /// A document committed, before its contents begin loading.
  DocumentCommitted { url: url::Url },
  /// Navigation failed or was cancelled. This is not a document-ready signal.
  NavigationFailed { url: url::Url },
  /// An address changed, including a same-document history or fragment change.
  AddressChanged { url: url::Url },
  /// The browser assigned this frame as its main frame.
  MainFrameChanged,
  /// Browser-wide load state. `false` follows every frame's load-end/error
  /// notifications, including cancelled navigation. It does not assert DOM,
  /// application, network-idle, or renderer responsiveness.
  LoadingStateChanged { is_loading: bool },
}

/// Synchronous observer for native frame lifecycle events.
pub type FrameEventHandler = dyn Fn(FrameEvent) + Send + Sync + 'static;

pub(crate) fn emit_frame_event(
  handler: &Option<Arc<FrameEventHandler>>,
  browser: Option<&mut cef::Browser>,
  frame: Option<&mut cef::Frame>,
  kind: FrameEventKind,
) {
  use cef::{ImplBrowser, ImplFrame};
  if let (Some(handler), Some(browser), Some(frame)) = (handler, browser, frame) {
    handler(FrameEvent {
      browser_id: browser.identifier(),
      frame_id: cef::CefString::from(&frame.identifier()).to_string(),
      is_main: frame.is_main() != 0,
      kind,
    });
  }
}
