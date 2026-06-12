use cef::*;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub(crate) use linux::repair_browser_window;

#[derive(Clone)]
pub enum CefWebview {
  BrowserView(cef::BrowserView),
  Browser(cef::Browser),
}

impl CefWebview {
  pub fn is_browser(&self) -> bool {
    matches!(self, CefWebview::Browser(_))
  }

  pub fn browser(&self) -> Option<cef::Browser> {
    match self {
      CefWebview::BrowserView(view) => view.browser(),
      CefWebview::Browser(browser) => Some(browser.clone()),
    }
  }

  #[allow(dead_code)]
  pub fn browser_id(&self) -> i32 {
    match self {
      CefWebview::BrowserView(view) => view.browser().map_or(-1, |b| b.identifier()),
      CefWebview::Browser(browser) => browser.identifier(),
    }
  }

  pub fn set_background_color(&self, color: Option<u32>) {
    if let CefWebview::BrowserView(view) = self {
      let window = view.window();
      let color = color.or_else(|| {
        window.map(|w| w.theme_color(ColorId::COLOR_PRIMARY_BACKGROUND.get_raw() as _))
      });

      if let Some(color) = color {
        view.set_background_color(color);
      }
    }
  }

  pub fn bounds(&self) -> cef::Rect {
    match self {
      CefWebview::BrowserView(view) => view.bounds(),
      CefWebview::Browser(browser) => browser.bounds(),
    }
  }

  pub fn set_bounds(&self, rect: Option<&cef::Rect>) {
    match self {
      CefWebview::BrowserView(view) => view.set_bounds(rect),
      CefWebview::Browser(browser) => browser.set_bounds(rect),
    }
  }

  pub fn scale_factor(&self) -> f64 {
    match self {
      CefWebview::BrowserView(view) => view
        .window()
        .and_then(|w| w.display())
        .map_or(1.0, |d| d.device_scale_factor() as f64),
      CefWebview::Browser(browser) => browser.scale_factor(),
    }
  }

  pub fn set_visible(&self, visible: i32) {
    match self {
      CefWebview::BrowserView(view) => view.set_visible(visible),
      CefWebview::Browser(browser) => browser.set_visible(visible),
    }
  }

  pub fn close(&self) {
    match self {
      CefWebview::BrowserView(view) => {
        // Alloy `BrowserView` (used for child webviews after the first): no
        // platform window to destroy, so we must drive CEF's normal close
        // lifecycle directly. Without this, dropping the `BrowserView`
        // refptr leaves the browser alive in CEF's internal state, which
        // hangs `cef::shutdown` and prevents the main process from exiting.
        if let Some(host) = view.browser().and_then(|b| b.host()) {
          let _ = host.try_close_browser();
        }
      }
      CefWebview::Browser(browser) => browser.close(),
    }
  }

  pub fn set_parent(&self, parent: &cef::Window) {
    match self {
      CefWebview::BrowserView(_) => {}
      CefWebview::Browser(browser) => browser.set_parent(parent),
    }
  }

  /// Raises the browser's X window above its siblings — most importantly above
  /// the CEF Views window's own full-size content surface, which Chromium
  /// restacks on top of the child browsers when the window is resized or
  /// shown again (e.g. tiling-WM workspace switches), leaving the webviews
  /// unpainted/occluded.
  #[cfg(target_os = "linux")]
  pub fn raise(&self) {
    if let CefWebview::Browser(browser) = self {
      browser.raise();
    }
  }

  /// Forces the browser's compositor to produce a fresh frame after the
  /// window manager hid and re-showed the toplevel. See the Linux
  /// `refresh_render_target` implementation for details.
  #[cfg(target_os = "linux")]
  pub fn refresh_render_target(&self) {
    if let CefWebview::Browser(browser) = self {
      browser.refresh_render_target();
    }
  }
}

trait CefBrowserExt {
  fn bounds(&self) -> cef::Rect;
  fn set_bounds(&self, rect: Option<&cef::Rect>);
  fn scale_factor(&self) -> f64;
  fn set_visible(&self, visible: i32);
  fn close(&self);
  fn set_parent(&self, parent: &cef::Window);

  #[cfg(target_os = "macos")]
  fn nsview(&self) -> Option<objc2::rc::Retained<objc2_app_kit::NSView>>;
  #[cfg(windows)]
  fn hwnd(&self) -> Option<::windows::Win32::Foundation::HWND>;
  #[cfg(target_os = "linux")]
  fn xid(&self) -> Option<u64>;
  #[cfg(target_os = "linux")]
  fn raise(&self);
  #[cfg(target_os = "linux")]
  fn refresh_render_target(&self);
}
