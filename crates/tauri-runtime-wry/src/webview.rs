// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod imp {
  /// The platform webview handle backed by the wry runtime.
  pub struct Webview {
    webview: webkit2gtk::WebView,
  }

  impl Webview {
    pub(crate) fn new(webview: webkit2gtk::WebView) -> Self {
      Self { webview }
    }

    /// Returns the [`webkit2gtk::WebView`] handle.
    pub fn inner(&self) -> webkit2gtk::WebView {
      self.webview.clone()
    }
  }
}

#[cfg(target_vendor = "apple")]
mod imp {
  use std::ffi::c_void;

  // These pointers are borrowed from ObjC `Retained` handles owned elsewhere and must
  // not be mutated through. TODO: change these to `*const c_void` in v3 (breaking change).
  pub struct Webview {
    webview: *mut c_void,
    manager: *mut c_void,
    #[cfg(target_os = "macos")]
    ns_window: *mut c_void,
    #[cfg(target_os = "ios")]
    view_controller: *mut c_void,
  }

  impl Webview {
    pub(crate) fn new(
      webview: *mut c_void,
      manager: *mut c_void,
      #[cfg(target_os = "macos")] ns_window: *mut c_void,
      #[cfg(target_os = "ios")] view_controller: *mut c_void,
    ) -> Self {
      Self {
        webview,
        manager,
        #[cfg(target_os = "macos")]
        ns_window,
        #[cfg(target_os = "ios")]
        view_controller,
      }
    }

    /// Returns the [WKWebView] handle.
    ///
    /// [WKWebView]: https://developer.apple.com/documentation/webkit/wkwebview
    pub fn inner(&self) -> *mut c_void {
      self.webview
    }

    /// Returns WKWebView [controller] handle.
    ///
    /// [controller]: https://developer.apple.com/documentation/webkit/wkusercontentcontroller
    pub fn controller(&self) -> *mut c_void {
      self.manager
    }

    /// Returns [NSWindow] associated with the WKWebView webview.
    ///
    /// [NSWindow]: https://developer.apple.com/documentation/appkit/nswindow
    #[cfg(target_os = "macos")]
    pub fn ns_window(&self) -> *mut c_void {
      self.ns_window
    }

    /// Returns [UIViewController] used by the WKWebView webview NSWindow.
    ///
    /// [UIViewController]: https://developer.apple.com/documentation/uikit/uiviewcontroller
    #[cfg(target_os = "ios")]
    pub fn view_controller(&self) -> *mut c_void {
      self.view_controller
    }
  }
}

#[cfg(windows)]
mod imp {
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Controller, ICoreWebView2Environment,
  };

  /// The platform webview handle backed by the wry runtime.
  pub struct Webview {
    controller: ICoreWebView2Controller,
    environment: ICoreWebView2Environment,
  }

  impl Webview {
    pub(crate) fn new(
      controller: ICoreWebView2Controller,
      environment: ICoreWebView2Environment,
    ) -> Self {
      Self {
        controller,
        environment,
      }
    }

    /// Returns the WebView2 controller.
    pub fn controller(&self) -> ICoreWebView2Controller {
      self.controller.clone()
    }

    /// Returns the WebView2 environment.
    pub fn environment(&self) -> ICoreWebView2Environment {
      self.environment.clone()
    }
  }
}

#[cfg(target_os = "android")]
mod imp {
  use wry::JniHandle;

  /// The platform webview handle backed by the wry runtime.
  pub struct Webview {
    handle: JniHandle,
  }

  impl Webview {
    pub(crate) fn new(handle: JniHandle) -> Self {
      Self { handle }
    }

    /// Returns the handle for JNI execution.
    pub fn jni_handle(&self) -> JniHandle {
      self.handle
    }
  }
}

pub use imp::*;
