// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::cell::Cell;

use crate::{
  platform::{EventLoopExt, MonitorExt},
  webview::AppWebview,
  window::AppWindow,
};
use cef::{
  ImplBrowserHost,
  application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol},
};
use objc2::{
  ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, extern_methods,
  msg_send, rc::Retained, runtime::Bool,
};
use objc2_app_kit::{
  NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBezierPath, NSColor,
  NSDockTile, NSEvent, NSImageView, NSProgressIndicator, NSScreen, NSView, NSWindow,
  NSWindowButton, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSInsetRect, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use tauri_runtime::{
  Error, ProgressBarState, ProgressBarStatus, Result,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalRect, Position, Rect},
};
use tauri_utils::{TitleBarStyle, config::Color};
use winit::{
  event_loop::ActiveEventLoop, monitor::MonitorHandle, platform::macos::MonitorHandleExtMacOS,
};

#[derive(Default)]
struct CefWinitApplicationIvars {
  handling_send_event: Cell<Bool>,
}

define_class!(
  #[unsafe(super(NSApplication))]
  #[ivars = CefWinitApplicationIvars]
  struct CefWinitApplication;

  impl CefWinitApplication {
    #[unsafe(method(sendEvent:))]
    unsafe fn send_event(&self, event: &NSEvent) {
      let was_handling = self.ivars().handling_send_event.get();
      self.ivars().handling_send_event.set(Bool::YES);
      let _: () = unsafe { msg_send![super(self), sendEvent: event] };
      self.ivars().handling_send_event.set(was_handling);
    }
  }

  unsafe impl CrAppControlProtocol for CefWinitApplication {
    #[unsafe(method(setHandlingSendEvent:))]
    unsafe fn set_handling_send_event(&self, handling_send_event: Bool) {
      self.ivars().handling_send_event.set(handling_send_event);
    }
  }

  unsafe impl CrAppProtocol for CefWinitApplication {
    #[unsafe(method(isHandlingSendEvent))]
    unsafe fn is_handling_send_event(&self) -> Bool {
      self.ivars().handling_send_event.get()
    }
  }

  unsafe impl CefAppProtocol for CefWinitApplication {}
);

impl CefWinitApplication {
  extern_methods! {
    #[unsafe(method(sharedApplication))]
    fn shared_application() -> Retained<Self>;
  }
}

struct DockProgressIndicatorIvars {
  state: Cell<ProgressBarStatus>,
}

impl Default for DockProgressIndicatorIvars {
  fn default() -> Self {
    Self {
      state: Cell::new(ProgressBarStatus::None),
    }
  }
}

define_class!(
  #[unsafe(super(NSProgressIndicator))]
  #[ivars = DockProgressIndicatorIvars]
  struct DockProgressIndicator;

  impl DockProgressIndicator {
    #[unsafe(method(drawRect:))]
    fn draw_rect(&self, rect: NSRect) {
      let bar = NSRect::new(
        NSPoint::new(0.0, 4.0),
        NSSize::new(rect.size.width, 8.0),
      );
      let bar_inner = NSInsetRect(bar, 0.5, 0.5);
      let mut bar_progress = NSInsetRect(bar, 1.0, 1.0);

      let progress = (self.doubleValue() / 100.0).clamp(0.0, 1.0);
      bar_progress.size.width *= progress;

      NSColor::colorWithWhite_alpha(1.0, 0.05).set();
      draw_rounded_rect(bar);
      draw_rounded_rect(bar_inner);

      let progress_color = match self.ivars().state.get() {
        ProgressBarStatus::Paused => NSColor::systemYellowColor(),
        ProgressBarStatus::Error => NSColor::systemRedColor(),
        _ => NSColor::systemBlueColor(),
      };
      progress_color.set();
      draw_rounded_rect(bar_progress);
    }
  }
);

impl DockProgressIndicator {
  fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(DockProgressIndicatorIvars::default());
    unsafe { msg_send![super(this), initWithFrame: frame] }
  }

  fn set_state(&self, status: ProgressBarStatus) {
    self.ivars().state.set(status);
  }
}

fn draw_rounded_rect(rect: NSRect) {
  let radius = rect.size.height / 2.0;
  NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();
}

fn set_dock_progress_bar(state: ProgressBarState) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  let app = NSApplication::sharedApplication(mtm);
  let dock_tile = app.dockTile();
  let Some(progress_indicator) = dock_progress_indicator(&app, &dock_tile, mtm) else {
    return;
  };

  if let Some(progress) = state.progress {
    progress_indicator.setDoubleValue(progress.min(100) as f64);
    progress_indicator.setHidden(false);
  }

  if let Some(status) = state.status {
    progress_indicator.set_state(status);
    progress_indicator.setHidden(matches!(status, ProgressBarStatus::None));
  }

  dock_tile.display();
}

fn dock_progress_indicator(
  app: &NSApplication,
  dock_tile: &NSDockTile,
  mtm: MainThreadMarker,
) -> Option<Retained<DockProgressIndicator>> {
  let content_view = match dock_tile.contentView(mtm) {
    Some(content_view) => content_view,
    None => {
      let app_icon = app.applicationIconImage()?;
      let image_view = NSImageView::imageViewWithImage(&app_icon, mtm);
      dock_tile.setContentView(Some(&image_view));
      dock_tile.contentView(mtm)?
    }
  };

  if let Some(progress_indicator) = existing_progress_indicator(&content_view) {
    return Some(progress_indicator);
  }

  let dock_tile_size = dock_tile.size();
  let frame = NSRect::new(
    NSPoint::new(0.0, 0.0),
    NSSize::new(dock_tile_size.width, 15.0),
  );
  let progress_indicator = DockProgressIndicator::new(mtm, frame);
  content_view.addSubview(&progress_indicator);

  Some(progress_indicator)
}

fn existing_progress_indicator(content_view: &NSView) -> Option<Retained<DockProgressIndicator>> {
  let subviews = content_view.subviews();
  for idx in 0..subviews.count() {
    let subview = subviews.objectAtIndex(idx);
    if let Ok(progress_indicator) = subview.downcast::<DockProgressIndicator>() {
      return Some(progress_indicator);
    }
  }

  None
}

pub fn setup_application() {
  let _ = CefWinitApplication::shared_application();
  let mtm = MainThreadMarker::new().expect("macOS application must start on the main thread");
  assert!(NSApp(mtm).isKindOfClass(CefWinitApplication::class()));
}

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let Some(ns_screen) = self.ns_screen() else {
      return super::monitor_bounds(self);
    };

    let ns_screen: &NSScreen = unsafe { &*ns_screen.cast() };
    let screen_frame = ns_screen.frame();
    let visible_frame = ns_screen.visibleFrame();
    let scale_factor = self.scale_factor();

    let position = self.position().unwrap_or_default();
    let mut position = position.to_logical::<f64>(scale_factor);
    position.x += visible_frame.origin.x - screen_frame.origin.x;
    position.y += (screen_frame.origin.y + screen_frame.size.height)
      - (visible_frame.origin.y + visible_frame.size.height);

    let size = LogicalSize::new(visible_frame.size.width, visible_frame.size.height);

    PhysicalRect {
      position: position.to_physical(scale_factor),
      size: size.to_physical(scale_factor),
    }
  }
}

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_activation_policy(&self, policy: tauri_runtime::ActivationPolicy) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let policy = match policy {
      tauri_runtime::ActivationPolicy::Regular => NSApplicationActivationPolicy::Regular,
      tauri_runtime::ActivationPolicy::Accessory => NSApplicationActivationPolicy::Accessory,
      tauri_runtime::ActivationPolicy::Prohibited => NSApplicationActivationPolicy::Prohibited,
      _ => NSApplicationActivationPolicy::Regular,
    };
    app.setActivationPolicy(policy);
  }

  fn set_dock_visibility(&self, visible: bool) {
    self.set_activation_policy(if visible {
      tauri_runtime::ActivationPolicy::Regular
    } else {
      tauri_runtime::ActivationPolicy::Accessory
    });
  }

  fn show_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    NSApp(mtm).unhide(None);
  }

  fn hide_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    NSApp(mtm).hide(None);
  }

  fn set_progress_bar(&self, state: ProgressBarState) {
    let _ = self;
    set_dock_progress_bar(state);
  }

  fn set_badge_count(&self, count: Option<i64>, _desktop_filename: Option<String>) {
    self.set_badge_label(count.map(|count| count.to_string()));
  }

  fn set_badge_label(&self, label: Option<String>) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let dock_tile = app.dockTile();
    let ns_label = label.map(|label| NSString::from_str(&label));
    dock_tile.setBadgeLabel(ns_label.as_deref());
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    let Some(mtm) = MainThreadMarker::new() else {
      return Err(Error::FailedToGetCursorPosition);
    };
    // `NSEvent::mouseLocation` uses a bottom-left origin; flip to top-left.
    let location: NSPoint = NSEvent::mouseLocation();
    let screen_height = NSScreen::mainScreen(mtm)
      .map(|screen| screen.frame().size.height)
      .ok_or(Error::FailedToGetCursorPosition)?;
    Ok(PhysicalPosition::new(
      location.x,
      screen_height - location.y,
    ))
  }
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
      if let Some(attached) = unsafe { nswindow.attachedSheet() } {
        unsafe { nswindow.endSheet(&attached) };
      }
    } else {
      if unsafe { nswindow.attachedSheet() }.is_some() {
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
      nswindow.beginSheet_completionHandler(&sheet, None);
    }
  }

  pub(crate) fn is_enabled(&self) -> bool {
    self
      .nsview()
      .and_then(|nsview| nsview.window())
      .map(|nswindow| unsafe { nswindow.attachedSheet() }.is_none())
      .unwrap_or(true)
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
      .map(ns_color_from_tauri_color)
      .unwrap_or_else(NSColor::windowBackgroundColor);
    nswindow.setOpaque(color.map(|color| color.3 == u8::MAX).unwrap_or(true));
    nswindow.setBackgroundColor(Some(&nscolor));
  }
}

fn ns_color_from_tauri_color(color: Color) -> objc2::rc::Retained<NSColor> {
  let Color(red, green, blue, alpha) = color;
  let scale = u8::MAX as f64;
  NSColor::colorWithSRGBRed_green_blue_alpha(
    red as f64 / scale,
    green as f64 / scale,
    blue as f64 / scale,
    alpha as f64 / scale,
  )
}

impl AppWebview {
  pub(crate) fn nsview(&self) -> Option<Retained<NSView>> {
    let handle = self.host.window_handle();
    let view = handle.cast::<NSView>();
    unsafe { Retained::<NSView>::retain(view) }
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    nsview.setWantsLayer(true);

    let Some(layer) = nsview.layer() else {
      return;
    };

    let nscolor = color
      .map(ns_color_from_tauri_color)
      .unwrap_or_else(NSColor::windowBackgroundColor);
    let cg_color = nscolor.CGColor();
    layer.setBackgroundColor(Some(&cg_color));
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let Some(nsview) = self.nsview() else {
      return None;
    };

    let parent = unsafe { nsview.superview()? };
    let parent_frame = parent.frame();
    let frame = nsview.frame();

    let y = if parent.isFlipped() {
      frame.origin.y
    } else {
      parent_frame.size.height - frame.origin.y - frame.size.height
    };

    let position = LogicalPosition::new(frame.origin.x, y);
    let size = LogicalSize::new(frame.size.width, frame.size.height);

    Some(Rect {
      position: position.into(),
      size: size.into(),
    })
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let Some(view) = self.nsview() else {
      return;
    };
    let Some(parent) = parent.nsview() else {
      return;
    };

    parent.addSubview(&view);
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let Some(nsview) = self.nsview() else {
      return;
    };

    nsview.setHidden(!visible);
  }

  pub(crate) fn apply_physical_bounds(&self, scale: f64, x: i32, y: i32, width: i32, height: i32) {
    let Some(nsview) = self.nsview() else {
      return;
    };
    let Some(parent) = (unsafe { nsview.superview() }) else {
      return;
    };

    // CEF provides child bounds as physical pixels, but NSView frames are logical pixels.
    let x = x as f64 / scale;
    let y = y as f64 / scale;
    let width = width as f64 / scale;
    let height = height as f64 / scale;

    let parent_frame = parent.frame();
    let y = if parent.isFlipped() {
      y
    } else {
      parent_frame.size.height - (y + height)
    };

    let frame = NSRect::new(NSPoint::new(x, y), NSSize::new(width, height));
    nsview.setFrame(frame);
  }
}
