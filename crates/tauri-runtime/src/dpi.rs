// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use dpi::*;
use serde::Serialize;

/// A rectangular region.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rect {
  /// Rect position.
  pub position: dpi::Position,
  /// Rect size.
  pub size: dpi::Size,
}

impl Rect {
  pub fn to_physical<P: dpi::Pixel, S: dpi::Pixel>(self, scale: f64) -> PhysicalRect<P, S> {
    PhysicalRect {
      position: self.position.to_physical(scale),
      size: self.size.to_physical(scale),
    }
  }

  pub fn to_logical<P: dpi::Pixel, S: dpi::Pixel>(self, scale: f64) -> LogicalRect<P, S> {
    LogicalRect {
      position: self.position.to_logical(scale),
      size: self.size.to_logical(scale),
    }
  }
}

impl Default for Rect {
  fn default() -> Self {
    Self {
      position: Position::Logical((0, 0).into()),
      size: Size::Logical((0, 0).into()),
    }
  }
}

/// A rectangular region in physical pixels.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PhysicalRect<P: dpi::Pixel, S: dpi::Pixel> {
  /// Rect position.
  pub position: dpi::PhysicalPosition<P>,
  /// Rect size.
  pub size: dpi::PhysicalSize<S>,
}

impl<P: dpi::Pixel, S: dpi::Pixel> Default for PhysicalRect<P, S> {
  fn default() -> Self {
    Self {
      position: (0, 0).into(),
      size: (0, 0).into(),
    }
  }
}

impl<P: dpi::Pixel, S: dpi::Pixel> PhysicalRect<P, S> {
  pub fn to_logical(self, scale: f64) -> LogicalRect<P, S> {
    LogicalRect {
      position: self.position.to_logical(scale),
      size: self.size.to_logical(scale),
    }
  }
}

/// A rectangular region in logical pixels.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct LogicalRect<P: dpi::Pixel, S: dpi::Pixel> {
  /// Rect position.
  pub position: dpi::LogicalPosition<P>,
  /// Rect size.
  pub size: dpi::LogicalSize<S>,
}

impl<P: dpi::Pixel, S: dpi::Pixel> Default for LogicalRect<P, S> {
  fn default() -> Self {
    Self {
      position: (0, 0).into(),
      size: (0, 0).into(),
    }
  }
}

impl<P: dpi::Pixel, S: dpi::Pixel> LogicalRect<P, S> {
  pub fn to_physical(self, scale: f64) -> PhysicalRect<P, S> {
    PhysicalRect {
      position: self.position.to_physical(scale),
      size: self.size.to_physical(scale),
    }
  }
}
