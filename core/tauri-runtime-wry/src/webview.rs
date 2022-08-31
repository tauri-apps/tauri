// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
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
  use std::rc::Rc;

  pub type Webview = Rc<webkit2gtk::WebView>;
}

#[cfg(target_os = "macos")]
mod imp {
  use cocoa::base::id;

  pub struct Webview {
    pub webview: id,
    pub manager: id,
    pub ns_window: id,
  }
}

#[cfg(windows)]
mod imp {
  use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller;
  pub struct Webview {
    pub controller: ICoreWebView2Controller,
  }
}

pub use imp::*;
