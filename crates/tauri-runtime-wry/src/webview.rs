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
  pub type Webview = webkit2gtk::WebView;
}

#[cfg(target_vendor = "apple")]
mod imp {
  use std::ffi::c_void;

  pub struct Webview {
    pub webview: *mut c_void,
    pub manager: *mut c_void,
    #[cfg(target_os = "macos")]
    pub ns_window: *mut c_void,
    #[cfg(target_os = "ios")]
    pub view_controller: *mut c_void,
  }
}

#[cfg(windows)]
mod imp {
  use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller;
  pub struct Webview {
    pub controller: ICoreWebView2Controller,
  }
}

#[cfg(target_os = "android")]
mod imp {
  use wry::JniHandle;
  pub type Webview = JniHandle;
}

pub use imp::*;
