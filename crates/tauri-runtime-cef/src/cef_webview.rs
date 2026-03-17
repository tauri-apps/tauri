use cef::*;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

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
        window.map(|w| w.theme_color(ColorId::CEF_ColorPrimaryBackground.get_raw() as _))
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
      CefWebview::BrowserView(_) => {}
      CefWebview::Browser(browser) => browser.close(),
    }
  }

  pub fn set_parent(&self, parent: &cef::Window) {
    match self {
      CefWebview::BrowserView(_) => {}
      CefWebview::Browser(browser) => browser.set_parent(parent),
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
}
