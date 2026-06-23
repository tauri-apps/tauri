// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
  mpsc::Sender,
};
use std::time::Duration;

use cef::*;
use tauri_runtime::{UserEvent, window::WindowId};
use winit::event_loop::EventLoopProxy as WinitEventLoopProxy;

use crate::{
  ipc, request_handler,
  runtime::{Message, RuntimeContext},
  webview::INITIAL_LOAD_URL,
};

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

wrap_load_handler! {
  struct TauriCefLoadHandler {
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
  }

  impl LoadHandler {
    fn on_load_start(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _transition_type: TransitionType,
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
        handler(url, tauri_runtime::webview::PageLoadEvent::Started);
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

wrap_display_handler! {
  struct TauriCefDisplayHandler {
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    address_changed_handler: Option<Arc<tauri_runtime::webview::AddressChangedHandler>>,
  }

  impl DisplayHandler {
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
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      url: Option<&CefString>,
    ) {
      // Only fire for main frame URL changes (matches on_before_browse behavior).
      if let Some(frame) = frame
        && frame.is_main() == 0
      {
        return;
      }
      let Some(handler) = &self.address_changed_handler else {
        return;
      };
      let Some(url) = url else {
        return;
      };
      let url = url.to_string();

      if url == INITIAL_LOAD_URL {
        return;
      }

      if let Ok(url) = url::Url::parse(&url) {
        handler(&url);
      }
    }
  }
}

wrap_life_span_handler! {
  struct TauriCefChildLifeSpanHandler<T: UserEvent> {
    sender: Sender<Message<T>>,
    proxy: WinitEventLoopProxy,
    window_id: WindowId,
    webview_id: u32,
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

wrap_client! {
  pub(crate) struct TauriCefBrowserClient<T: UserEvent> {
    pub(crate) context: RuntimeContext<T>,
    pub(crate) window_id: WindowId,
    pub(crate) webview_id: u32,
    pub(crate) label: String,
    initial_url: Option<String>,
    pub(crate) ipc_handler: Option<Arc<ipc::IpcHandler<T>>>,
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    navigation_handler: Option<Arc<tauri_runtime::webview::NavigationHandler>>,
    address_changed_handler: Option<Arc<tauri_runtime::webview::AddressChangedHandler>>,
    proxy: WinitEventLoopProxy,
    sender: Sender<Message<T>>,
  }

  impl Client {
    fn request_handler(&self) -> Option<RequestHandler> {
      Some(request_handler::WebRequestHandler::new(
        self.navigation_handler.clone(),
        self.context.clone(),
        self.window_id,
        self.webview_id,
      ))
    }

    fn life_span_handler(&self) -> Option<LifeSpanHandler> {
      Some(TauriCefChildLifeSpanHandler::new(
        self.sender.clone(),
        self.proxy.clone(),
        self.window_id,
        self.webview_id,
        self.initial_url.clone(),
      ))
    }

    fn load_handler(&self) -> Option<LoadHandler> {
      Some(TauriCefLoadHandler::new(self.on_page_load_handler.clone()))
    }

    fn display_handler(&self) -> Option<DisplayHandler> {
      Some(TauriCefDisplayHandler::new(
        self.document_title_changed_handler.clone(),
        self.address_changed_handler.clone(),
      ))
    }

    fn on_process_message_received(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      source_process: ProcessId,
      message: Option<&mut ProcessMessage>,
    ) -> std::os::raw::c_int {
      ipc::on_process_message_received(self, frame, source_process, message)
    }
  }
}

wrap_browser_process_handler! {
  pub(crate) struct TauriCefBrowserProcessHandler<T: UserEvent> {
    sender: Sender<Message<T>>,
    proxy: WinitEventLoopProxy,
    context_initialized: Arc<AtomicBool>,
    deep_link_schemes: Vec<String>,
  }

  impl BrowserProcessHandler {
    fn on_context_initialized(&self) {
      self.context_initialized.store(true, Ordering::SeqCst);
      self.proxy.wake_up();
    }

    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
      let delay = Duration::from_millis(delay_ms.max(0) as u64);
      let _ = self.sender.send(Message::CefWork(delay));
      self.proxy.wake_up();
    }

    fn on_already_running_app_relaunch(
      &self,
      command_line: Option<&mut CommandLine>,
      _current_directory: Option<&CefString>,
    ) -> std::os::raw::c_int {
      let Some(command_line) = command_line else {
        return 0;
      };
      let mut list = CefStringList::new();
      command_line.arguments(Some(&mut list));
      let args: Vec<String> = list.into_iter().collect();
      if let Ok(url) = url::Url::parse(&args[0]) {
        let scheme = url.scheme().to_string();
        if self.deep_link_schemes.iter().any(|s| s == &scheme) {
          let _ = self.sender.send(Message::Opened(vec![url]));
          self.proxy.wake_up();
          return 1;
        }
      }
      // TODO: add event
      1
    }
  }
}
