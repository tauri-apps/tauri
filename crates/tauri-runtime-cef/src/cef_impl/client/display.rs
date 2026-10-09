// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use cef::*;

use crate::macros::wrap_with_args;
use crate::webview::INITIAL_LOAD_URL;

#[cfg(target_os = "macos")]
type CefCursorHandle = *mut u8;
#[cfg(not(target_os = "macos"))]
type CefCursorHandle = cef::sys::cef_cursor_handle_t;

wrap_with_args! {
  wrap_display_handler => TauriCefDisplayHandlerArgs;

  pub struct TauriCefDisplayHandler {
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    frame_event_handler: Option<Arc<crate::FrameEventHandler>>,
    console_message_handler: Option<Arc<crate::ConsoleMessageHandler>>,
    frame_navigation_state: crate::FrameNavigationState,
    cursor_handler: Option<Arc<crate::CursorHandler>>,
  }

  impl DisplayHandler {
    fn on_cursor_change(
      &self,
      browser: Option<&mut Browser>,
      _cursor: CefCursorHandle,
      type_: CursorType,
      info: Option<&CursorInfo>,
    ) -> i32 {
      if let Some(handler) = &self.cursor_handler
        && browser.is_some_and(|browser| {
          self.frame_navigation_state.has_browser_id(browser.identifier())
        })
      {
        handler(type_, info);
        return 1;
      }
      0
    }

    fn on_title_change(
      &self,
      _browser: Option<&mut Browser>,
      title: Option<&CefString>,
    ) {
      let Some(handler) = &self.document_title_changed_handler else {
        return;
      };
      let Some(title) = title else {
        return;
      };

      handler(title.to_string());
    }

    fn on_address_change(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      url: Option<&CefString>,
    ) {
      let Some(url) = url else {
        return;
      };
      let url = url.to_string();

      if url == INITIAL_LOAD_URL {
        return;
      }

      if let Ok(url) = url::Url::parse(&url) {
        crate::frame::emit_frame_event(
          &self.frame_event_handler,
          browser,
          frame,
          crate::FrameEventKind::AddressChanged { url },
        );
      }
    }

    fn on_console_message(
      &self,
      browser: Option<&mut Browser>,
      level: LogSeverity,
      message: Option<&CefString>,
      source: Option<&CefString>,
      line: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
      if let Some(handler) = &self.console_message_handler {
        // Scoped the way the frame observer is: CEF routes browsers this webview
        // does not own through this very client — a DevTools window is the
        // standing case, and its frontend is itself a page that logs.
        let observed = browser
          .map(|browser| self.frame_navigation_state.has_browser_id(browser.identifier()))
          .unwrap_or(false);
        if observed {
          handler(crate::ConsoleMessage::from_cef(level, message, source, line));
        }
      }

      // 0 leaves CEF's own logging of the message exactly as it was.
      0
    }
  }
}
