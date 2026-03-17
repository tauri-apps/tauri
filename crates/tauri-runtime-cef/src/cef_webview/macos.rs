use crate::cef_webview::CefBrowserExt;
use cef::*;
use objc2::rc::Retained;
use objc2_app_kit::NSView;
use objc2_foundation::{NSPoint, NSRect, NSSize};

impl CefBrowserExt for cef::Browser {
  fn nsview(&self) -> Option<objc2::rc::Retained<objc2_app_kit::NSView>> {
    let host = self.host()?;
    let nsview = host.window_handle() as *mut NSView;
    unsafe { Retained::<NSView>::retain(nsview) }
  }

  fn bounds(&self) -> cef::Rect {
    let Some(nsview) = self.nsview() else {
      return cef::Rect::default();
    };

    let parent = unsafe { nsview.superview().unwrap() };
    let parent_frame = parent.frame();
    let webview_frame = nsview.frame();

    cef::Rect {
      x: webview_frame.origin.x as i32,
      y: (parent_frame.size.height - webview_frame.origin.y - webview_frame.size.height) as i32,
      width: webview_frame.size.width as i32,
      height: webview_frame.size.height as i32,
    }
  }

  fn set_bounds(&self, rect: Option<&cef::Rect>) {
    let Some(rect) = rect else {
      return;
    };

    let Some(nsview) = self.nsview() else {
      return;
    };

    let parent = unsafe { nsview.superview().unwrap() };
    let parent_frame = parent.frame();

    let origin = NSPoint {
      x: rect.x as f64,
      y: (parent_frame.size.height - (rect.y as f64 + rect.height as f64)),
    };

    let size = NSSize {
      width: rect.width as f64,
      height: rect.height as f64,
    };

    unsafe { nsview.setFrame(NSRect { origin, size }) };
  }

  fn scale_factor(&self) -> f64 {
    let Some(nsview) = self.nsview() else {
      return 1.0;
    };

    let screen = nsview.window().and_then(|w| w.screen());
    screen.map(|s| s.backingScaleFactor()).unwrap_or(1.0)
  }

  fn set_visible(&self, visible: i32) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    if visible != 0 {
      nsview.setHidden(false);
    } else {
      nsview.setHidden(true);
    }
  }

  fn close(&self) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    unsafe { nsview.removeFromSuperview() };
  }

  fn set_parent(&self, parent: &cef::Window) {
    crate::cef_impl::ensure_valid_content_view(parent.window_handle());

    let Some(nsview) = self.nsview() else {
      return;
    };

    let parent_nsview = parent.window_handle();
    let Some(parent_nsview) = (unsafe { Retained::<NSView>::retain(parent_nsview as _) }) else {
      return;
    };

    unsafe { parent_nsview.addSubview(&nsview) };
  }
}
