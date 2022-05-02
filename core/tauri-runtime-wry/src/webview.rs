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

pub use imp::*;
