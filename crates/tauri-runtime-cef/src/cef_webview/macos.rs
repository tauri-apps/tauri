use crate::cef_webview::CefBrowserExt;
use cef::*;
use objc2::{msg_send, rc::Retained};
use objc2_app_kit::{NSColor, NSView};
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
      y: (parent_frame.size.height as f64 - (rect.y as f64 + rect.height as f64)),
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
    screen.map(|s| s.backingScaleFactor() as f64).unwrap_or(1.0)
  }

  fn set_background_color(&self, color: cef::Color) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    let red = ((color >> 16) & 0xFF) as f64 / 255.0;
    let green = ((color >> 8) & 0xFF) as f64 / 255.0;
    let blue = (color & 0xFF) as f64 / 255.0;
    let alpha = ((color >> 24) & 0xFF) as f64 / 255.0;

    let color = unsafe { NSColor::colorWithRed_green_blue_alpha(red, green, blue, alpha) };
    let color = unsafe { color.CGColor() };

    nsview.setWantsLayer(true);

    let Some(layer) = (unsafe { nsview.layer() }) else {
      return;
    };
    let _: () = unsafe { msg_send![&layer, setBackgroundColor: &*color] };
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
