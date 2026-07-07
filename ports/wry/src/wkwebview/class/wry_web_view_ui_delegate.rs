// Copyright 2020-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "macos")]
use std::{cell::RefCell, ptr::null_mut, rc::Rc};

use block2::Block;
use objc2::{
  define_class, msg_send, rc::Retained, runtime::NSObject, DefinedClass, MainThreadOnly,
};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSModalResponse, NSModalResponseOK, NSOpenPanel, NSWindowDelegate};
use objc2_foundation::{MainThreadMarker, NSObjectProtocol};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSArray, NSURL};

#[cfg(target_os = "macos")]
use objc2_web_kit::WKOpenPanelParameters;
use objc2_web_kit::{
  WKFrameInfo, WKMediaCaptureType, WKPermissionDecision, WKSecurityOrigin, WKUIDelegate,
};

use crate::{NewWindowFeatures, NewWindowResponse, PermissionKind, PermissionResponse, WryWebView};

#[cfg(target_os = "macos")]
struct NewWindow {
  #[allow(dead_code)]
  ns_window: Retained<objc2_app_kit::NSWindow>,
  #[allow(dead_code)]
  webview: Retained<objc2_web_kit::WKWebView>,
  #[allow(dead_code)]
  delegate: Retained<WryNSWindowDelegate>,
}

// SAFETY: we are not using the new window at all, just dropping it on another thread
#[cfg(target_os = "macos")]
unsafe impl Send for NewWindow {}

#[cfg(target_os = "macos")]
impl Drop for NewWindow {
  fn drop(&mut self) {
    unsafe {
      self.webview.removeFromSuperview();
    }
  }
}

#[cfg(target_os = "macos")]
struct WryNSWindowDelegateIvars {
  on_close: Box<dyn Fn()>,
}

#[cfg(target_os = "macos")]
define_class!(
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[ivars = WryNSWindowDelegateIvars]
  struct WryNSWindowDelegate;

  unsafe impl NSObjectProtocol for WryNSWindowDelegate {}

  unsafe impl NSWindowDelegate for WryNSWindowDelegate {
    #[unsafe(method(windowWillClose:))]
    unsafe fn will_close(&self, _notification: &objc2_foundation::NSNotification) {
      let on_close = &self.ivars().on_close;
      on_close();
    }
  }
);

#[cfg(target_os = "macos")]
impl WryNSWindowDelegate {
  pub fn new(mtm: MainThreadMarker, on_close: Box<dyn Fn()>) -> Retained<Self> {
    let delegate = mtm
      .alloc::<WryNSWindowDelegate>()
      .set_ivars(WryNSWindowDelegateIvars { on_close });
    unsafe { msg_send![super(delegate), init] }
  }
}

pub struct WryWebViewUIDelegateIvars {
  #[cfg(target_os = "macos")]
  new_window_req_handler: Option<Box<dyn Fn(String, NewWindowFeatures) -> NewWindowResponse>>,
  #[cfg(target_os = "macos")]
  new_windows: Rc<RefCell<Vec<NewWindow>>>,
  permission_handler: Option<Box<dyn Fn(PermissionKind) -> PermissionResponse + Send + Sync>>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[ivars = WryWebViewUIDelegateIvars]
  pub struct WryWebViewUIDelegate;

  unsafe impl NSObjectProtocol for WryWebViewUIDelegate {}

  unsafe impl WKUIDelegate for WryWebViewUIDelegate {
    #[cfg(target_os = "macos")]
    #[unsafe(method(webView:runOpenPanelWithParameters:initiatedByFrame:completionHandler:))]
    fn run_file_upload_panel(
      &self,
      _webview: &WryWebView,
      open_panel_params: &WKOpenPanelParameters,
      _frame: &WKFrameInfo,
      handler: &block2::Block<dyn Fn(*const NSArray<NSURL>)>,
    ) {
      unsafe {
        if let Some(mtm) = MainThreadMarker::new() {
          let open_panel = NSOpenPanel::openPanel(mtm);
          open_panel.setCanChooseFiles(true);
          let allow_multi = open_panel_params.allowsMultipleSelection();
          open_panel.setAllowsMultipleSelection(allow_multi);
          let allow_dir = open_panel_params.allowsDirectories();
          open_panel.setCanChooseDirectories(allow_dir);
          let ok: NSModalResponse = open_panel.runModal();
          if ok == NSModalResponseOK {
            let url = open_panel.URLs();
            (*handler).call((Retained::as_ptr(&url),));
          } else {
            (*handler).call((null_mut(),));
          }
        }
      }
    }

    #[unsafe(method(webView:requestMediaCapturePermissionForOrigin:initiatedByFrame:type:decisionHandler:))]
    fn request_media_capture_permission(
      &self,
      _webview: &WryWebView,
      _origin: &WKSecurityOrigin,
      _frame: &WKFrameInfo,
      capture_type: WKMediaCaptureType,
      decision_handler: &Block<dyn Fn(WKPermissionDecision)>,
    ) {
      // Call user's permission handler if set
      let decision = if let Some(handler) = &self.ivars().permission_handler {
        let translate_response = |res: PermissionResponse| match res {
          PermissionResponse::Allow => WKPermissionDecision::Grant,
          PermissionResponse::Deny => WKPermissionDecision::Deny,
          PermissionResponse::Default => WKPermissionDecision::Prompt,
        };

        match capture_type {
          WKMediaCaptureType::Camera => translate_response(handler(PermissionKind::Camera)),
          WKMediaCaptureType::Microphone => translate_response(handler(PermissionKind::Microphone)),
          WKMediaCaptureType::CameraAndMicrophone => {
            let mic_res = handler(PermissionKind::Microphone);
            let cam_res = handler(PermissionKind::Camera);

            match (mic_res, cam_res) {
              (PermissionResponse::Allow, PermissionResponse::Allow) => WKPermissionDecision::Grant,
              (PermissionResponse::Deny, _) | (_, PermissionResponse::Deny) => {
                WKPermissionDecision::Deny
              }
              _ => WKPermissionDecision::Prompt,
            }
          }
          _ => translate_response(handler(PermissionKind::Other)),
        }
      } else {
        WKPermissionDecision::Grant
      };

      (*decision_handler).call((decision,));
    }

    #[cfg(target_os = "macos")]
    #[unsafe(method_id(webView:createWebViewWithConfiguration:forNavigationAction:windowFeatures:))]
    unsafe fn create_web_view_for_navigation_action(
      &self,
      webview: &WryWebView,
      configuration: &objc2_web_kit::WKWebViewConfiguration,
      action: &objc2_web_kit::WKNavigationAction,
      window_features: &objc2_web_kit::WKWindowFeatures,
    ) -> Option<Retained<objc2_web_kit::WKWebView>> {
      if let Some(new_window_req_handler) = &self.ivars().new_window_req_handler {
        let request = action.request();
        let url = request.URL().unwrap().absoluteString().unwrap();

        let current_window = webview.window().unwrap();
        let screen = current_window.screen().unwrap();
        let screen_frame = screen.frame();

        match new_window_req_handler(
          url.to_string(),
          NewWindowFeatures {
            size: if let (Some(width), Some(height)) =
              (window_features.width(), window_features.height())
            {
              Some(dpi::LogicalSize::new(
                width.doubleValue(),
                height.doubleValue(),
              ))
            } else {
              None
            },
            position: if let (Some(x), Some(y)) = (window_features.x(), window_features.y()) {
              Some(dpi::LogicalPosition::new(x.doubleValue(), y.doubleValue()))
            } else {
              None
            },
            opener: crate::NewWindowOpener {
              webview: webview.into(),
              target_configuration: configuration.into(),
            },
          },
        ) {
          NewWindowResponse::Allow => {
            let mtm = MainThreadMarker::new().unwrap();

            let defaults = current_window.frame();
            let size = objc2_foundation::NSSize::new(
              window_features
                .width()
                .map_or(defaults.size.width, |width| width.doubleValue()),
              window_features
                .height()
                .map_or(defaults.size.height, |height| height.doubleValue()),
            );
            let position = objc2_foundation::NSPoint::new(
              window_features
                .x()
                .map_or(defaults.origin.x, |x| x.doubleValue()),
              window_features.y().map_or(defaults.origin.y, |y| {
                screen_frame.size.height - y.doubleValue() - size.height
              }),
            );
            let rect = objc2_foundation::NSRect::new(position, size);

            let mut flags = objc2_app_kit::NSWindowStyleMask::Titled
              | objc2_app_kit::NSWindowStyleMask::Closable
              | objc2_app_kit::NSWindowStyleMask::Miniaturizable;
            let resizable = window_features
              .allowsResizing()
              .map_or(true, |resizable| resizable.boolValue());
            if resizable {
              flags |= objc2_app_kit::NSWindowStyleMask::Resizable;
            }

            let window = objc2_app_kit::NSWindow::initWithContentRect_styleMask_backing_defer(
              mtm.alloc::<objc2_app_kit::NSWindow>(),
              rect,
              flags,
              objc2_app_kit::NSBackingStoreType::Buffered,
              false,
            );

            // SAFETY: Disable auto-release when closing windows.
            // This is required when creating `NSWindow` outside a window
            // controller.
            window.setReleasedWhenClosed(false);

            let webview = objc2_web_kit::WKWebView::initWithFrame_configuration(
              mtm.alloc::<objc2_web_kit::WKWebView>(),
              window.frame(),
              configuration,
            );

            let new_windows = self.ivars().new_windows.clone();
            let window_id = Retained::as_ptr(&window) as usize;
            let delegate = WryNSWindowDelegate::new(
              mtm,
              Box::new(move || {
                new_windows
                  .borrow_mut()
                  .retain(|window| Retained::as_ptr(&window.ns_window) as usize != window_id);
              }),
            );
            window.setDelegate(Some(objc2::runtime::ProtocolObject::from_ref(&*delegate)));

            window.setContentView(Some(&webview));
            window.makeKeyAndOrderFront(None);

            self.ivars().new_windows.borrow_mut().push(NewWindow {
              ns_window: window,
              webview: webview.clone(),
              delegate,
            });

            Some(webview)
          }
          NewWindowResponse::Create { webview } => Some(webview),
          NewWindowResponse::Deny => None,
        }
      } else {
        None
      }
    }
  }
);

impl WryWebViewUIDelegate {
  pub fn new(
    mtm: MainThreadMarker,
    new_window_req_handler: Option<Box<dyn Fn(String, NewWindowFeatures) -> NewWindowResponse>>,
    permission_handler: Option<Box<dyn Fn(PermissionKind) -> PermissionResponse + Send + Sync>>,
  ) -> Retained<Self> {
    #[cfg(target_os = "ios")]
    let _new_window_req_handler = new_window_req_handler;

    let delegate = mtm
      .alloc::<WryWebViewUIDelegate>()
      .set_ivars(WryWebViewUIDelegateIvars {
        #[cfg(target_os = "macos")]
        new_window_req_handler,
        #[cfg(target_os = "macos")]
        new_windows: Rc::new(RefCell::new(vec![])),
        permission_handler,
      });
    unsafe { msg_send![super(delegate), init] }
  }
}
