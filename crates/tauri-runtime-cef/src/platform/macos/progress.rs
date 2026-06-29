// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::cell::Cell;

use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained};
use objc2_app_kit::{
  NSApplication, NSBezierPath, NSColor, NSDockTile, NSImageView, NSProgressIndicator, NSView,
};
use objc2_foundation::{NSInsetRect, NSPoint, NSRect, NSSize};
use tauri_runtime::{ProgressBarState, ProgressBarStatus};

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

pub fn set_dock_progress_bar(state: ProgressBarState) {
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
