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

pub use imp::*;
