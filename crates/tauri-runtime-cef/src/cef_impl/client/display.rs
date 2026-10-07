// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use cef::*;

use crate::macros::wrap_with_args;
use crate::webview::INITIAL_LOAD_URL;

#[cfg(target_os = "macos")]
type CefCursorHandle = *mut u8;
#[cfg(not(target_os = "macos"))]
type CefCursorHandle = cef::sys::cef_cursor_handle_t;

wrap_with_args! {
  wrap_display_handler => TauriCefDisplayHandlerArgs;

  pub struct TauriCefDisplayHandler {
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    frame_event_handler: Option<Arc<crate::FrameEventHandler>>,
    console_message_handler: Option<Arc<crate::ConsoleMessageHandler>>,
    frame_navigation_state: crate::FrameNavigationState,
    offscreen: Option<crate::OffscreenView>,
  }

  impl DisplayHandler {
    fn on_cursor_change(
      &self,
      browser: Option<&mut Browser>,
      _cursor: CefCursorHandle,
      type_: CursorType,
      info: Option<&CursorInfo>,
    ) -> i32 {
      if let Some(view) = &self.offscreen
        && browser.is_some_and(|browser| {
          self.frame_navigation_state.has_browser_id(browser.identifier())
        })
      {
        view.set_cursor(cursor_update(type_, info));
        return 1;
      }
      0
    }

    fn on_title_change(
      &self,
      _browser: Option<&mut Browser>,
      title: Option<&CefString>,
    ) {
      let Some(handler) = &self.document_title_changed_handler else {
        return;
      };
      let Some(title) = title else {
        return;
      };

      handler(title.to_string());
    }

    fn on_address_change(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      url: Option<&CefString>,
    ) {
      let Some(url) = url else {
        return;
      };
      let url = url.to_string();

      if url == INITIAL_LOAD_URL {
        return;
      }

      if let Ok(url) = url::Url::parse(&url) {
        crate::frame::emit_frame_event(
          &self.frame_event_handler,
          browser,
          frame,
          crate::FrameEventKind::AddressChanged { url },
        );
      }
    }

    fn on_console_message(
      &self,
      browser: Option<&mut Browser>,
      level: LogSeverity,
      message: Option<&CefString>,
      source: Option<&CefString>,
      line: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int {
      if let Some(handler) = &self.console_message_handler {
        // Scoped the way the frame observer is: CEF routes browsers this webview
        // does not own through this very client — a DevTools window is the
        // standing case, and its frontend is itself a page that logs.
        let observed = browser
          .map(|browser| self.frame_navigation_state.has_browser_id(browser.identifier()))
          .unwrap_or(false);
        if observed {
          handler(crate::ConsoleMessage::from_cef(level, message, source, line));
        }
      }

      // 0 leaves CEF's own logging of the message exactly as it was.
      0
    }
  }
}

fn cursor_update(
  cursor: cef::CursorType,
  info: Option<&cef::CursorInfo>,
) -> crate::offscreen::CursorUpdate {
  use crate::offscreen::CursorUpdate;
  use cef::CursorType as C;
  use winit::cursor::CursorIcon as W;
  if cursor == C::NONE {
    return CursorUpdate::Hidden;
  }
  if cursor == C::CUSTOM
    && let Some(cursor) = custom_cursor(info)
  {
    return cursor;
  }
  let icon = match cursor {
    C::CROSS => W::Crosshair,
    C::HAND => W::Pointer,
    C::IBEAM => W::Text,
    C::WAIT => W::Wait,
    C::HELP => W::Help,
    C::MOVE => W::Move,
    C::EASTRESIZE => W::EResize,
    C::NORTHRESIZE => W::NResize,
    C::NORTHEASTRESIZE => W::NeResize,
    C::NORTHWESTRESIZE => W::NwResize,
    C::SOUTHRESIZE => W::SResize,
    C::SOUTHEASTRESIZE => W::SeResize,
    C::SOUTHWESTRESIZE => W::SwResize,
    C::WESTRESIZE => W::WResize,
    C::NORTHSOUTHRESIZE => W::NsResize,
    C::EASTWESTRESIZE => W::EwResize,
    C::NORTHEASTSOUTHWESTRESIZE => W::NeswResize,
    C::NORTHWESTSOUTHEASTRESIZE => W::NwseResize,
    C::COLUMNRESIZE => W::ColResize,
    C::ROWRESIZE => W::RowResize,
    C::VERTICALTEXT => W::VerticalText,
    C::CELL => W::Cell,
    C::CONTEXTMENU => W::ContextMenu,
    C::ALIAS => W::Alias,
    C::PROGRESS => W::Progress,
    C::NODROP => W::NoDrop,
    C::COPY => W::Copy,
    C::NOTALLOWED => W::NotAllowed,
    C::ZOOMIN => W::ZoomIn,
    C::ZOOMOUT => W::ZoomOut,
    C::GRAB => W::Grab,
    C::GRABBING => W::Grabbing,
    _ => W::Default,
  };
  CursorUpdate::Ready(icon.into())
}

fn custom_cursor(info: Option<&cef::CursorInfo>) -> Option<crate::offscreen::CursorUpdate> {
  use winit::cursor::{CustomCursorSource, MAX_CURSOR_SIZE};
  let info = info?;
  let (width, height) = (
    u16::try_from(info.size.width).ok()?,
    u16::try_from(info.size.height).ok()?,
  );
  let (x, y) = (
    u16::try_from(info.hotspot.x).ok()?,
    u16::try_from(info.hotspot.y).ok()?,
  );
  if info.buffer.is_null()
    || width == 0
    || height == 0
    || width > MAX_CURSOR_SIZE
    || height > MAX_CURSOR_SIZE
  {
    return None;
  }
  let length = width as usize * height as usize * 4;
  // CEF lends this BGRA image for the callback. A native cursor outlives it,
  // unlike paint frames, so the runtime copies this small platform resource.
  let source = unsafe { std::slice::from_raw_parts(info.buffer.cast::<u8>(), length) };
  // AppKit's winit cursor constructor interprets image dimensions as points.
  // Other desktop backends consume pixel dimensions directly.
  let scale = if cfg!(target_os = "macos")
    && info.image_scale_factor.is_finite()
    && info.image_scale_factor > 0.0
  {
    info.image_scale_factor as f64
  } else {
    1.0
  };
  let output_width = (width as f64 / scale)
    .round()
    .clamp(1.0, MAX_CURSOR_SIZE as f64) as u16;
  let output_height = (height as f64 / scale)
    .round()
    .clamp(1.0, MAX_CURSOR_SIZE as f64) as u16;
  let mut rgba = Vec::with_capacity(output_width as usize * output_height as usize * 4);
  for row in 0..output_height {
    for column in 0..output_width {
      let source_x = ((column as f64 + 0.5) * scale)
        .floor()
        .min(width as f64 - 1.0) as usize;
      let source_y = ((row as f64 + 0.5) * scale)
        .floor()
        .min(height as f64 - 1.0) as usize;
      let offset = (source_y * width as usize + source_x) * 4;
      rgba.extend_from_slice(&[
        source[offset + 2],
        source[offset + 1],
        source[offset],
        source[offset + 3],
      ]);
    }
  }
  let x = (x as f64 / scale).round().min(output_width as f64 - 1.0) as u16;
  let y = (y as f64 / scale).round().min(output_height as f64 - 1.0) as u16;
  CustomCursorSource::from_rgba(rgba, output_width, output_height, x, y)
    .ok()
    .map(crate::offscreen::CursorUpdate::Image)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn offscreen_custom_cursor_owns_its_pixels_and_preserves_hotspot_units() {
    let mut pixels = [10u8, 20, 30, 255].repeat(16);
    let info = cef::CursorInfo {
      size: cef::Size {
        width: 4,
        height: 4,
      },
      hotspot: cef::Point { x: 2, y: 2 },
      image_scale_factor: 2.0,
      buffer: pixels.as_mut_ptr().cast(),
    };
    let Some(crate::offscreen::CursorUpdate::Image(winit::cursor::CustomCursorSource::Image(
      image,
    ))) = custom_cursor(Some(&info))
    else {
      panic!("expected cursor image");
    };
    pixels.fill(0);
    assert_eq!(&image.buffer()[..4], &[30, 20, 10, 255]);
    let (size, hotspot) = if cfg!(target_os = "macos") {
      (2, 1)
    } else {
      (4, 2)
    };
    assert_eq!((image.width(), image.height()), (size, size));
    assert_eq!((image.hotspot_x(), image.hotspot_y()), (hotspot, hotspot));
  }
}
