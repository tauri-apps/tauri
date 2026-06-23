// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(not(target_os = "macos"))]
use std::time::Duration;
use std::{
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
  },
};

use cef::*;
use tauri_runtime::{
  UserEvent,
  dpi::{LogicalPosition, LogicalSize},
  window::WindowId,
};
use winit::event_loop::EventLoopProxy as WinitEventLoopProxy;

use crate::{
  ipc, request_handler,
  runtime::{CefRuntime, Message, NewWindowOpener, RuntimeContext},
  webview::INITIAL_LOAD_URL,
};

#[cfg(target_os = "linux")]
type CefOsEvent<'a> = Option<&'a mut cef::sys::XEvent>;
#[cfg(target_os = "macos")]
type CefOsEvent<'a> = *mut u8;
#[cfg(windows)]
type CefOsEvent<'a> = Option<&'a mut cef::sys::MSG>;

#[derive(Default)]
pub(crate) struct DragDropState {
  pub(crate) paths: Option<Vec<PathBuf>>,
  pub(crate) native_entered: bool,
  pub(crate) entered: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragDropEventTarget {
  Window,
  Webview,
}

#[derive(Clone, serde::Deserialize)]
pub(crate) struct DragDropScriptEvent {
  #[serde(rename = "type")]
  pub(crate) kind: String,
  pub(crate) x: f64,
  pub(crate) y: f64,
}

fn collect_drag_data_paths(drag_data: &mut DragData) -> Vec<PathBuf> {
  let mut paths = CefStringList::new();
  if drag_data.file_paths(Some(&mut paths)) != 0 {
    let paths = paths
      .into_iter()
      .filter(|path| !path.is_empty())
      .map(PathBuf::from)
      .collect::<Vec<_>>();

    if !paths.is_empty() {
      return paths;
    }
  }

  let file_name = CefStringUtf16::from(&drag_data.file_name()).to_string();
  if file_name.is_empty() {
    Vec::new()
  } else {
    vec![PathBuf::from(file_name)]
  }
}

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

wrap_drag_handler! {
  struct TauriCefDragHandler {
    drag_drop_state: Arc<Mutex<DragDropState>>,
  }

  impl DragHandler {
    fn on_drag_enter(
      &self,
      _browser: Option<&mut Browser>,
      drag_data: Option<&mut DragData>,
      _mask: DragOperationsMask,
    ) -> ::std::os::raw::c_int {
      let mut state = self.drag_drop_state.lock().unwrap();
      state.entered = false;
      state.paths = drag_data
        .map(collect_drag_data_paths)
        .filter(|paths| !paths.is_empty());
      state.native_entered = state.paths.is_some();

      // Let Chromium continue with the drag operation so the injected script can
      // report over/drop/leave with accurate viewport positions.
      0
    }
  }
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

wrap_download_handler! {
  struct TauriCefDownloadHandler {
    download_handler: Arc<tauri_runtime::webview::DownloadHandler>,
  }

  impl DownloadHandler {
    fn can_download(
      &self,
      _browser: Option<&mut Browser>,
      _url: Option<&CefStringUtf16>,
      _request_method: Option<&CefStringUtf16>,
    ) -> ::std::os::raw::c_int {
      // on_before_download is the one that actually validates the download.
      // so we return 1 to allow the download here
      1
    }

    fn on_before_download(
      &self,
      _browser: Option<&mut Browser>,
      download_item: Option<&mut DownloadItem>,
      suggested_name: Option<&CefStringUtf16>,
      callback: Option<&mut BeforeDownloadCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(download_item) = download_item else {
        return 0;
      };
      let Some(callback) = callback else {
        return 0;
      };

      let url_str = CefString::from(&download_item.url()).to_string();
      let Ok(url) = url::Url::parse(&url_str) else {
        return 0;
      };

      let suggested_path = suggested_name
        .map(|s| s.to_string())
        .map(std::path::PathBuf::from)
        .unwrap_or_default();

      let mut destination = suggested_path.clone();

      // Call handler with Requested event.
      let should_allow =
        (self.download_handler)(tauri_runtime::webview::DownloadEvent::Requested {
          url: url.clone(),
          destination: &mut destination,
      });

      if should_allow {
        // Set the download path.
        let destination_cef = CefStringUtf16::from(destination.to_string_lossy().as_ref());

        // If the user callback did not modify the destination, show the dialog.
        let show_dialog = destination == suggested_path;
        callback.cont(Some(&destination_cef), show_dialog as ::std::os::raw::c_int);
      }

      1
    }

    fn on_download_updated(
      &self,
      _browser: Option<&mut Browser>,
      download_item: Option<&mut DownloadItem>,
      _callback: Option<&mut DownloadItemCallback>,
    ) {
      let Some(download_item) = download_item else {
        return;
      };

      // Get download URL.
      let url_str = CefString::from(&download_item.url()).to_string();
      let Ok(url) = url::Url::parse(&url_str) else {
        return;
      };

      // Check download state - CEF returns i32 where 0 is false, non-zero is true.
      let is_complete = download_item.is_complete() != 0;
      let is_canceled = download_item.is_canceled() != 0;
      let success = is_complete && !is_canceled;

      // Get full path if available - full_path() returns CefStringUserfreeUtf16.
      let full_path = if is_complete || is_canceled {
        let path_cef = download_item.full_path();
        let path_str = CefString::from(&path_cef).to_string();
        if !path_str.is_empty() {
          Some(std::path::PathBuf::from(path_str))
        } else {
          None
        }
      } else {
        None
      };

      // Only call handler when download is finished (complete or canceled).
      if is_complete || is_canceled {
        // Call handler with Finished event.
        (self.download_handler)(tauri_runtime::webview::DownloadEvent::Finished {
          url,
          path: full_path,
          success,
        });
      }
    }
  }
}

wrap_context_menu_handler! {
  struct TauriCefContextMenuHandler {
    devtools_enabled: bool,
  }

  impl ContextMenuHandler {
    fn on_before_context_menu(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _params: Option<&mut ContextMenuParams>,
      model: Option<&mut MenuModel>,
    ) {
      if !self.devtools_enabled
        && let Some(model) = model
      {
        model.remove_at(model.count() - 1);
      }
    }
  }
}

wrap_keyboard_handler! {
  struct TauriCefKeyboardHandler {
    devtools_enabled: bool,
  }

  impl KeyboardHandler {
    fn on_pre_key_event(
      &self,
      _browser: Option<&mut Browser>,
      event: Option<&KeyEvent>,
      _os_event: CefOsEvent<'_>,
      _is_keyboard_shortcut: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      // If devtools is disabled, block devtools keyboard shortcuts.
      if !self.devtools_enabled {
        let Some(event) = event else {
          return 0;
        };

        // Check if this is a keydown event.
        use cef::sys::cef_key_event_type_t;
        let keydown_type: cef::KeyEventType = cef_key_event_type_t::KEYEVENT_RAWKEYDOWN.into();
        if event.type_ != keydown_type {
          return 0;
        }

        // Get modifier keys.
        use cef::sys::cef_event_flags_t;
        #[cfg(windows)]
        let modifiers = event.modifiers as i32;
        #[cfg(not(windows))]
        let modifiers = event.modifiers;

        #[cfg(not(target_os = "macos"))]
        let ctrl = (modifiers & (cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0)) != 0;
        #[cfg(not(target_os = "macos"))]
        let shift = (modifiers & (cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0)) != 0;

        let key_code = event.windows_key_code;

        // Block F12 (key code 123).
        if key_code == 123 {
          if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
            *is_keyboard_shortcut = 1;
          }
          return 1;
        }

        // Block Ctrl+Shift+I (key code 73 = 'I') on Linux/Windows.
        #[cfg(not(target_os = "macos"))]
        if key_code == 73 && ctrl && shift {
          if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
            *is_keyboard_shortcut = 1;
          }
          return 1;
        }

        // Block Cmd+Opt+I on macOS.
        #[cfg(target_os = "macos")]
        {
          let meta = (modifiers & cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0) != 0;
          let alt = (modifiers & cef_event_flags_t::EVENTFLAG_ALT_DOWN.0) != 0;
          if key_code == 73 && meta && alt {
            if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
              *is_keyboard_shortcut = 1;
            }
            return 1;
          }
        }
      }

      0
    }
  }
}

wrap_permission_handler! {
  struct TauriCefPermissionHandler {}

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow microphone and camera when requested.
      let allowed = requested_permissions
        & (cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE
          as u32
          | cef::sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE
            as u32);
      if allowed != 0 {
        callback.cont(requested_permissions);
        return 1;
      }
      0
    }

    fn on_show_permission_prompt(
      &self,
      _browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      _requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow permission prompt (e.g. microphone/camera).
      callback.cont(PermissionRequestResult::from(
        cef::sys::cef_permission_request_result_t::CEF_PERMISSION_RESULT_ACCEPT,
      ));
      1
    }
  }
}

wrap_life_span_handler! {
  struct TauriCefChildLifeSpanHandler<T: UserEvent> {
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

pub(crate) struct TauriCefBrowserClientHandlers<T: UserEvent> {
  pub(crate) ipc_handler: Option<Arc<ipc::IpcHandler<T>>>,
  pub(crate) on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
  pub(crate) document_title_changed_handler:
    Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
  pub(crate) navigation_handler: Option<Arc<tauri_runtime::webview::NavigationHandler>>,
  pub(crate) address_changed_handler: Option<Arc<tauri_runtime::webview::AddressChangedHandler>>,
  pub(crate) new_window_handler:
    Option<Arc<tauri_runtime::webview::NewWindowHandler<T, CefRuntime<T>>>>,
  pub(crate) download_handler: Option<Arc<tauri_runtime::webview::DownloadHandler>>,
}

impl<T: UserEvent> Clone for TauriCefBrowserClientHandlers<T> {
  fn clone(&self) -> Self {
    Self {
      ipc_handler: self.ipc_handler.clone(),
      on_page_load_handler: self.on_page_load_handler.clone(),
      document_title_changed_handler: self.document_title_changed_handler.clone(),
      navigation_handler: self.navigation_handler.clone(),
      address_changed_handler: self.address_changed_handler.clone(),
      new_window_handler: self.new_window_handler.clone(),
      download_handler: self.download_handler.clone(),
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
    devtools_enabled: bool,
    drag_drop_event_target: DragDropEventTarget,
    drag_drop_handler_enabled: bool,
    drag_drop_state: Arc<Mutex<DragDropState>>,
    pub(crate) handlers: TauriCefBrowserClientHandlers<T>,
    proxy: WinitEventLoopProxy,
    sender: Sender<Message<T>>,
  }

  impl Client {
    fn drag_handler(&self) -> Option<DragHandler> {
      self
        .drag_drop_handler_enabled
        .then(|| TauriCefDragHandler::new(self.drag_drop_state.clone()))
    }

    fn request_handler(&self) -> Option<RequestHandler> {
      Some(request_handler::WebRequestHandler::new(
        self.handlers.navigation_handler.clone(),
        self.context.clone(),
        self.window_id,
        self.webview_id,
        self.drag_drop_event_target,
        self.drag_drop_handler_enabled,
        self.drag_drop_state.clone(),
      ))
    }

    fn life_span_handler(&self) -> Option<LifeSpanHandler> {
      Some(TauriCefChildLifeSpanHandler::new(
        self.sender.clone(),
        self.proxy.clone(),
        self.window_id,
        self.webview_id,
        self.context.clone(),
        self.handlers.new_window_handler.clone(),
        self.initial_url.clone(),
      ))
    }

    fn load_handler(&self) -> Option<LoadHandler> {
      Some(TauriCefLoadHandler::new(
        self.handlers.on_page_load_handler.clone(),
      ))
    }

    fn display_handler(&self) -> Option<DisplayHandler> {
      Some(TauriCefDisplayHandler::new(
        self.handlers.document_title_changed_handler.clone(),
        self.handlers.address_changed_handler.clone(),
      ))
    }

    fn download_handler(&self) -> Option<DownloadHandler> {
      self
        .handlers
        .download_handler
        .clone()
        .map(TauriCefDownloadHandler::new)
    }

    fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
      Some(TauriCefContextMenuHandler::new(self.devtools_enabled))
    }

    fn keyboard_handler(&self) -> Option<KeyboardHandler> {
      Some(TauriCefKeyboardHandler::new(self.devtools_enabled))
    }

    fn permission_handler(&self) -> Option<PermissionHandler> {
      Some(TauriCefPermissionHandler::new())
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
    context: RuntimeContext<T>,
    context_initialized: Arc<AtomicBool>,
    deep_link_schemes: Vec<String>,
  }

  impl BrowserProcessHandler {
    fn on_context_initialized(&self) {
      self.context_initialized.store(true, Ordering::SeqCst);
      self.context.proxy.wake_up();
    }

    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
      #[cfg(target_os = "macos")]
      {
        self.context.cef_pump.schedule_message_pump_work(delay_ms);
      }
      #[cfg(not(target_os = "macos"))]
      {
        let delay = Duration::from_millis(delay_ms.max(0) as u64);
        let _ = self.context.sender.send(Message::CefWork(delay));
        self.context.proxy.wake_up();
      }
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
          let _ = self.context.sender.send(Message::Opened(vec![url]));
          self.context.proxy.wake_up();
          return 1;
        }
      }
      // TODO: add event
      1
    }
  }
}
