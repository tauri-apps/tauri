// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::ImplBrowserHost;
use dispatch2::MainThreadBound;
use objc2::{
  MainThreadMarker,
  rc::Retained,
  runtime::{AnyClass, AnyObject, Bool, Imp, Sel},
  sel,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSView, NSWindow, NSWindowButton, NSWindowCollectionBehavior,
  NSWindowStyleMask,
};
use std::{cell::Cell, ffi::c_void, mem, ptr};
use tauri_runtime::dpi::Position;
use tauri_utils::{TitleBarStyle, config::Color};

use crate::window::AppWindow;

use super::utils;

type CanBecomeWindow = extern "C-unwind" fn(&AnyObject, Sel) -> Bool;

static ORIGINAL_WINDOW_CAN_BECOME_KEY: MainThreadBound<Cell<Option<CanBecomeWindow>>> =
  MainThreadBound::new(
    Cell::new(None),
    // SAFETY: This is created in a static, where no thread is associated with the value.
    unsafe { MainThreadMarker::new_unchecked() },
  );
static ORIGINAL_WINDOW_CAN_BECOME_MAIN: MainThreadBound<Cell<Option<CanBecomeWindow>>> =
  MainThreadBound::new(
    Cell::new(None),
    // SAFETY: This is created in a static, where no thread is associated with the value.
    unsafe { MainThreadMarker::new_unchecked() },
  );
static ORIGINAL_PANEL_CAN_BECOME_KEY: MainThreadBound<Cell<Option<CanBecomeWindow>>> =
  MainThreadBound::new(
    Cell::new(None),
    // SAFETY: This is created in a static, where no thread is associated with the value.
    unsafe { MainThreadMarker::new_unchecked() },
  );

extern "C-unwind" fn window_can_become_key_window(this: &AnyObject, sel: Sel) -> Bool {
  can_become_window(this, sel, &ORIGINAL_WINDOW_CAN_BECOME_KEY)
}

extern "C-unwind" fn window_can_become_main_window(this: &AnyObject, sel: Sel) -> Bool {
  can_become_window(this, sel, &ORIGINAL_WINDOW_CAN_BECOME_MAIN)
}

extern "C-unwind" fn panel_can_become_key_window(this: &AnyObject, sel: Sel) -> Bool {
  can_become_window(this, sel, &ORIGINAL_PANEL_CAN_BECOME_KEY)
}

fn can_become_window(
  this: &AnyObject,
  sel: Sel,
  original: &MainThreadBound<Cell<Option<CanBecomeWindow>>>,
) -> Bool {
  if unsafe { !objc2::ffi::objc_getAssociatedObject(this, focusable_association_key()).is_null() } {
    return false.into();
  }

  let mtm = MainThreadMarker::new().expect("NSWindow focusability must be queried on main thread");
  let original = original
    .get(mtm)
    .get()
    .expect("no existing NSWindow focusability handler set");
  original(this, sel)
}

fn focusable_association_key() -> *const c_void {
  window_can_become_key_window as CanBecomeWindow as *const c_void
}

fn install_focusable_hooks() {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };

  if let Some(class) = AnyClass::get(c"WinitWindow") {
    override_can_become_window(
      mtm,
      class,
      sel!(canBecomeKeyWindow),
      window_can_become_key_window,
      &ORIGINAL_WINDOW_CAN_BECOME_KEY,
    );
    override_can_become_window(
      mtm,
      class,
      sel!(canBecomeMainWindow),
      window_can_become_main_window,
      &ORIGINAL_WINDOW_CAN_BECOME_MAIN,
    );
  }

  if let Some(class) = AnyClass::get(c"WinitPanel") {
    override_can_become_window(
      mtm,
      class,
      sel!(canBecomeKeyWindow),
      panel_can_become_key_window,
      &ORIGINAL_PANEL_CAN_BECOME_KEY,
    );
  }
}

fn override_can_become_window(
  mtm: MainThreadMarker,
  class: &AnyClass,
  selector: Sel,
  replacement: CanBecomeWindow,
  original: &MainThreadBound<Cell<Option<CanBecomeWindow>>>,
) {
  let Some(method) = class.instance_method(selector) else {
    return;
  };

  let overridden = unsafe { mem::transmute::<CanBecomeWindow, Imp>(replacement) };

  #[allow(unknown_lints, unpredictable_function_pointer_comparisons)]
  if overridden == method.implementation() {
    return;
  }

  let previous = unsafe { method.set_implementation(overridden) };
  let previous = unsafe { mem::transmute::<Imp, CanBecomeWindow>(previous) };
  original.get(mtm).set(Some(previous));
}

impl AppWindow {
  pub(crate) fn nsview(&self) -> Option<Retained<NSView>> {
    let handle = self.raw_handle_as_cef_handle();
    let view = handle.cast::<NSView>();
    unsafe { Retained::<NSView>::retain(view) }
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let Some(nsview) = self.nsview() else {
      return;
    };
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
      .and_then(|nsview| nsview.window())
      .map(|nswindow| nswindow.attachedSheet().is_none())
      .unwrap_or(true)
  }

  pub(crate) fn set_focusable(&self, focusable: bool) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(nswindow) = nsview.window() else {
      return;
    };

    install_focusable_hooks();
    let nswindow = (&*nswindow) as *const NSWindow as *mut AnyObject;
    let value = if focusable { ptr::null_mut() } else { nswindow };
    unsafe {
      objc2::ffi::objc_setAssociatedObject(
        nswindow,
        focusable_association_key(),
        value,
        objc2::ffi::OBJC_ASSOCIATION_ASSIGN,
      );
    }
  }

  pub(crate) fn apply_traffic_light_position(&self, position: &Position) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let Some(close) = nswindow.standardWindowButton(NSWindowButton::CloseButton) else {
      return;
    };
    let Some(miniaturize) = nswindow.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
      return;
    };
    let Some(zoom) = nswindow.standardWindowButton(NSWindowButton::ZoomButton) else {
      return;
    };

    let pos = position.to_logical::<f64>(nswindow.backingScaleFactor());
    let title_bar_container_view = unsafe { close.superview().and_then(|view| view.superview()) };
    let Some(title_bar_container_view) = title_bar_container_view else {
      return;
    };

    let close_rect = close.frame();
    let title_bar_frame_height = close_rect.size.height + pos.y;
    let mut title_bar_rect = title_bar_container_view.frame();
    title_bar_rect.size.height = title_bar_frame_height;
    title_bar_rect.origin.y = nswindow.frame().size.height - title_bar_frame_height;
    title_bar_container_view.setFrame(title_bar_rect);

    let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
    for (index, button) in [close, miniaturize, zoom].into_iter().enumerate() {
      let mut origin = button.frame().origin;
      origin.x = pos.x + (index as f64 * space_between);
      button.setFrameOrigin(origin);
    }
  }

  pub(crate) fn set_title_bar_style(&self, style: TitleBarStyle) {
    let Some(nsview) = self.nsview() else {
      return;
    };
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
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(nswindow) = nsview.window() else {
      return;
    };

    let mut collection_behavior = nswindow.collectionBehavior();
    collection_behavior.set(NSWindowCollectionBehavior::CanJoinAllSpaces, visible);
    nswindow.setCollectionBehavior(collection_behavior);
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let Some(nsview) = self.nsview() else {
      return;
    };
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
