// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{Arc, mpsc::Sender};

use cef::*;
use tauri_runtime::{
  UserEvent,
  dpi::{LogicalPosition, LogicalSize},
  window::WindowId,
};
use winit::event_loop::EventLoopProxy as WinitEventLoopProxy;

use crate::runtime::{CefRuntime, Message, NewWindowOpener, RuntimeContext};

// There is some race condition on CEF that causes the app loading to fail
// when there is a network service crash:
// "[85296:47750637:0127/131203.017395:ERROR:content/browser/network_service_instance_impl.cc:610] Network service crashed or was terminated, restarting service."
// We check the app URL for a while until it actually loads the initial URL.
fn check_and_reload_if_blank(browser: cef::Browser, initial_url: String) {
  if initial_url == "about:blank" {
    return;
  }

  std::thread::spawn(move || {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    let check_interval = std::time::Duration::from_millis(100);

    while start_time.elapsed() < timeout {
      if let Some(frame) = browser.main_frame() {
        let url = frame.url();
        let current_url = cef::CefString::from(&url).to_string();
        if current_url.is_empty() || current_url == "about:blank" {
          frame.load_url(Some(&cef::CefString::from(initial_url.as_str())));
          // Continue checking in case it loads about:blank again.
        } else {
          return;
        }
      }
      std::thread::sleep(check_interval);
    }
  });
}

wrap_life_span_handler! {
  pub struct TauriCefChildLifeSpanHandler<T: UserEvent> {
    sender: Sender<Message<T>>,
    proxy: WinitEventLoopProxy,
    window_id: WindowId,
    webview_id: u32,
    context: RuntimeContext<T>,
    new_window_handler: Option<Arc<tauri_runtime::webview::NewWindowHandler<T, CefRuntime<T>>>>,
    initial_url: Option<String>,
  }

  impl LifeSpanHandler {
    fn on_after_created(&self, browser: Option<&mut Browser>) {
      if let Some(browser) = browser
        && let Some(initial_url) = &self.initial_url
      {
        check_and_reload_if_blank(browser.clone(), initial_url.clone());
      }
    }

    fn on_before_popup(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _popup_id: std::os::raw::c_int,
      target_url: Option<&CefString>,
      _target_frame_name: Option<&CefString>,
      _target_disposition: WindowOpenDisposition,
      _user_gesture: std::os::raw::c_int,
      popup_features: Option<&PopupFeatures>,
      _window_info: Option<&mut WindowInfo>,
      _client: Option<&mut Option<Client>>,
      _settings: Option<&mut BrowserSettings>,
      _extra_info: Option<&mut Option<DictionaryValue>>,
      _no_javascript_access: Option<&mut i32>,
    ) -> std::os::raw::c_int {
      let Some(handler) = &self.new_window_handler else {
        return 0;
      };

      let Some(target_url) = target_url else {
        return 1;
      };

      let url_str = target_url.to_string();
      let Ok(url) = url::Url::parse(&url_str) else {
        return 1;
      };

      // window.open() features are CSS pixels, which map to Tauri's logical units.
      let size = popup_features.and_then(|features| {
        (features.width_set != 0 && features.height_set != 0)
          .then(|| LogicalSize::new(features.width as f64, features.height as f64))
      });
      let position = popup_features.and_then(|features| {
        (features.x_set != 0 && features.y_set != 0)
          .then(|| LogicalPosition::new(features.x as f64, features.y as f64))
      });
      let features =
        tauri_runtime::webview::NewWindowFeatures::new(size, position, NewWindowOpener {});

      match handler(url, features) {
        tauri_runtime::webview::NewWindowResponse::Allow => 0,
        tauri_runtime::webview::NewWindowResponse::Create { window_id } => {
          // CEF cannot transplant a popup's contents into an existing
          // browser, so cancel the popup and navigate the designated
          // window's first webview to the URL instead — the closest
          // equivalent of wry hosting the popup in that window's webview.
          // Note `window.opener` is not linked to the new document.
          let _ = self.context.send_message(Message::NavigateFirstWebview {
            window_id,
            url: url_str,
          });
          1
        }
        tauri_runtime::webview::NewWindowResponse::Deny => 1,
      }
    }

    fn on_before_close(&self, browser: Option<&mut Browser>) {
      if browser.is_none() {
        return;
      }
      let _ = self
        .sender
        .send(Message::BrowserClosed(self.window_id, self.webview_id));
      self.proxy.wake_up();
    }
  }
}
