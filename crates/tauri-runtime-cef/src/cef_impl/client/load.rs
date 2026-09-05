// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use cef::*;

wrap_load_handler! {
  pub struct TauriCefLoadHandler {
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
    frame_event_handler: Option<Arc<crate::FrameEventHandler>>,
  }

  impl LoadHandler {
    fn on_loading_state_change(
      &self,
      browser: Option<&mut Browser>,
      is_loading: ::std::os::raw::c_int,
      _can_go_back: ::std::os::raw::c_int,
      _can_go_forward: ::std::os::raw::c_int,
    ) {
      if let Some(browser) = browser {
        let mut frame = browser.main_frame();
        crate::frame::emit_frame_event(
          &self.frame_event_handler,
          Some(browser),
          frame.as_mut(),
          crate::FrameEventKind::LoadingStateChanged { is_loading: is_loading != 0 },
        );
      }
    }

    fn on_load_start(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _transition_type: TransitionType,
    ) {
      let Some(frame) = frame else {
        return;
      };
      let url = cef::CefString::from(&frame.url()).to_string();
      if let Ok(url) = url::Url::parse(&url) {
        let is_main = frame.is_main() != 0;
        crate::frame::emit_frame_event(
          &self.frame_event_handler,
          browser,
          Some(frame),
          crate::FrameEventKind::DocumentCommitted { url: url.clone() },
        );
        if is_main && let Some(handler) = &self.on_page_load_handler {
          handler(url, tauri_runtime::webview::PageLoadEvent::Started);
        }
      }
    }

    fn on_load_error(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _error_code: Errorcode,
      _error_text: Option<&CefString>,
      failed_url: Option<&CefString>,
    ) {
      if let Some(failed_url) = failed_url
        && let Ok(url) = url::Url::parse(&failed_url.to_string())
      {
        crate::frame::emit_frame_event(
          &self.frame_event_handler,
          browser,
          frame,
          crate::FrameEventKind::NavigationFailed { url },
        );
      }
    }

    fn on_load_end(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _http_status_code: ::std::os::raw::c_int,
    ) {
      let Some(handler) = &self.on_page_load_handler else {
        return;
      };
      let Some(frame) = frame else {
        return;
      };

      if frame.is_main() == 0 {
        return;
      }

      let url = cef::CefString::from(&frame.url()).to_string();
      if let Ok(url) = url::Url::parse(&url) {
        handler(url, tauri_runtime::webview::PageLoadEvent::Finished);
      }
    }
  }
}
