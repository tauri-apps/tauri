// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::Cell, mem, ptr};

use dispatch2::MainThreadBound;
use objc2::{
  rc::Retained,
  runtime::{Imp, Sel},
  sel,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSView, NSWindow, NSWindowButton, NSWindowCollectionBehavior,
  NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri_runtime::dpi::Position;
use tauri_utils::{TitleBarStyle, config::Color};

use crate::window::AppWindow;

use super::{AppkitState, utils};

impl AppWindow {
  pub(crate) fn raw_cef_handle(&self) -> cef::sys::cef_window_handle_t {
    let nsview = self.nsview();
    Retained::as_ptr(&nsview).cast_mut().cast()
  }

  pub(crate) fn nsview(&self) -> Retained<NSView> {
    let handle = self
      .window
      .window_handle()
      .expect("failed to get window handle");
    match handle.as_raw() {
      RawWindowHandle::AppKit(handle) => unsafe {
        Retained::<NSView>::retain(handle.ns_view.as_ptr().cast::<NSView>())
          .expect("failed to retain NSView")
      },
      other => panic!("expected AppKit window handle, got {other:?}"),
    }
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    if enabled {
      if let Some(attached) = nswindow.attachedSheet() {
        nswindow.endSheet(&attached);
      }
    } else {
      if nswindow.attachedSheet().is_some() {
        return;
      }

      let Some(mtm) = MainThreadMarker::new() else {
        return;
      };
      let frame = nswindow.frame();
      let sheet = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          mtm.alloc(),
          frame,
          NSWindowStyleMask::Titled,
          NSBackingStoreType::Buffered,
          false,
        )
      };
      sheet.setAlphaValue(0.5);
      (&*nswindow).beginSheet_completionHandler(&*sheet, None);
    }
  }

  pub(crate) fn is_enabled(&self) -> bool {
    self
      .nsview()
      .window()
      .map(|nswindow| nswindow.attachedSheet().is_none())
      .unwrap_or(true)
  }

  pub(crate) fn set_traffic_light_position(&self, position: &Position) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let pos = position.to_logical::<f64>(nswindow.backingScaleFactor());
    if let Ok(mut state) = self.appkit_state.write() {
      let pos = NSPoint::new(pos.x, pos.y);
      state.traffic_light_position = Some(pos);
    }

    inset_traffic_lights(&nswindow, pos.x, pos.y);
    swizzle_draw_rect(&nsview);
  }

  pub(crate) fn associate_appkit_state(&self) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    AppkitState::associate(&self.appkit_state, &nswindow);
  }

  pub(crate) fn set_title_bar_style(&self, style: TitleBarStyle) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    match style {
      TitleBarStyle::Visible => {
        nswindow.setTitlebarAppearsTransparent(false);
        let mut mask = nswindow.styleMask();
        mask.remove(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      TitleBarStyle::Transparent => {
        nswindow.setTitlebarAppearsTransparent(true);
        let mut mask = nswindow.styleMask();
        mask.remove(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      TitleBarStyle::Overlay => {
        nswindow.setTitlebarAppearsTransparent(true);
        let mut mask = nswindow.styleMask();
        mask.insert(NSWindowStyleMask::FullSizeContentView);
        nswindow.setStyleMask(mask);
      }
      _ => {}
    }
  }

  pub(crate) fn set_visible_on_all_workspaces(&self, visible: bool) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let mut collection_behavior = nswindow.collectionBehavior();
    collection_behavior.set(NSWindowCollectionBehavior::CanJoinAllSpaces, visible);
    nswindow.setCollectionBehavior(collection_behavior);
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let nsview = self.nsview();
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let nscolor = color
      .map(utils::ns_color_from_tauri_color)
      .unwrap_or_else(NSColor::windowBackgroundColor);
    nswindow.setOpaque(color.map(|color| color.3 == u8::MAX).unwrap_or(true));
    nswindow.setBackgroundColor(Some(&nscolor));
  }
}

fn inset_traffic_lights(nswindow: &NSWindow, x: f64, y: f64) {
  let Some(close) = nswindow.standardWindowButton(NSWindowButton::CloseButton) else {
    return;
  };
  let Some(miniaturize) = nswindow.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
    return;
  };
  let Some(zoom) = nswindow.standardWindowButton(NSWindowButton::ZoomButton) else {
    return;
  };

  let title_bar_container_view = unsafe { close.superview().and_then(|view| view.superview()) };
  let Some(title_bar_container_view) = title_bar_container_view else {
    return;
  };

  let close_rect = close.frame();
  let title_bar_frame_height = close_rect.size.height + y;
  let mut title_bar_rect = title_bar_container_view.frame();
  title_bar_rect.size.height = title_bar_frame_height;
  title_bar_rect.origin.y = nswindow.frame().size.height - title_bar_frame_height;
  title_bar_container_view.setFrame(title_bar_rect);

  let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
  for (index, button) in [close, miniaturize, zoom].into_iter().enumerate() {
    let mut origin = button.frame().origin;
    origin.x = x + (index as f64 * space_between);
    button.setFrameOrigin(origin);
  }
}

type DrawRect = extern "C-unwind" fn(&NSView, Sel, NSRect);

static ORIGINAL_DRAW_RECT: MainThreadBound<Cell<Option<DrawRect>>> = {
  // SAFETY: Creating in a `const` context, where there is no concept of the main thread.
  MainThreadBound::new(Cell::new(None), unsafe {
    MainThreadMarker::new_unchecked()
  })
};

extern "C-unwind" fn draw_rect(view: &NSView, sel: Sel, rect: NSRect) {
  let mtm = MainThreadMarker::from(view);
  let original = ORIGINAL_DRAW_RECT
    .get(mtm)
    .get()
    .expect("no existing drawRect: handler set");

  original(view, sel, rect);

  post_draw_rect(view, rect);
}

fn post_draw_rect(view: &NSView, _rect: NSRect) {
  let Some(nswindow) = view.window() else {
    return;
  };

  let Some(state) = AppkitState::from_window(&nswindow) else {
    return;
  };
  let Ok(state) = state.read() else {
    return;
  };

  if let Some(pos) = state.traffic_light_position {
    inset_traffic_lights(&nswindow, pos.x, pos.y);
  }
}

fn swizzle_draw_rect(nsview: &NSView) {
  let mtm = MainThreadMarker::from(nsview);
  let class = nsview.class();
  let Some(method) = class.instance_method(sel!(drawRect:)) else {
    return;
  };

  let overridden = unsafe { mem::transmute::<DrawRect, Imp>(draw_rect) };
  if ptr::fn_addr_eq(overridden, method.implementation()) {
    return;
  }

  let original = unsafe { method.set_implementation(overridden) };
  let original = unsafe { mem::transmute::<Imp, DrawRect>(original) };
  ORIGINAL_DRAW_RECT.get(mtm).set(Some(original));
}
