// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

fn validate_scale_factor(scale_factor: f64) -> bool {
  scale_factor.is_sign_positive() && scale_factor.is_normal()
}

/// A pixel definition. Must be created from a `f64` value.
pub trait Pixel: Copy + Into<f64> {
  /// Creates the pixel from the `f64` value.
  fn from_f64(f: f64) -> Self;
  /// Casts a pixel.
  fn cast<P: Pixel>(self) -> P {
    P::from_f64(self.into())
  }
}

impl Pixel for u8 {
  fn from_f64(f: f64) -> Self {
    f.round() as u8
  }
}

impl Pixel for u16 {
  fn from_f64(f: f64) -> Self {
    f.round() as u16
  }
}

impl Pixel for u32 {
  fn from_f64(f: f64) -> Self {
    f.round() as u32
  }
}

impl Pixel for i8 {
  fn from_f64(f: f64) -> Self {
    f.round() as i8
  }
}

impl Pixel for i16 {
  fn from_f64(f: f64) -> Self {
    f.round() as i16
  }
}

impl Pixel for i32 {
  fn from_f64(f: f64) -> Self {
    f.round() as i32
  }
}

impl Pixel for f32 {
  fn from_f64(f: f64) -> Self {
    f as f32
  }
}

impl Pixel for f64 {
  #[allow(clippy::wrong_self_convention)]
  fn from_f64(f: f64) -> Self {
    f
  }
}

/// A position represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct PhysicalPosition<P> {
  /// Vertical axis value.
  pub x: P,
  /// Horizontal axis value.
  pub y: P,
}

impl<P: Pixel> PhysicalPosition<P> {
  /// Converts the physical position to a logical one, using the scale factor.
  #[inline]
  pub fn to_logical<X: Pixel>(self, scale_factor: f64) -> LogicalPosition<X> {
    assert!(validate_scale_factor(scale_factor));
    let x = self.x.into() / scale_factor;
    let y = self.y.into() / scale_factor;
    LogicalPosition { x, y }.cast()
  }
}

/// A position represented in logical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct LogicalPosition<P> {
  /// Vertical axis value.
  pub x: P,
  /// Horizontal axis value.
  pub y: P,
}

impl<T: Pixel> LogicalPosition<T> {
  /// Casts the logical size to another pixel type.
  #[inline]
  pub fn cast<X: Pixel>(&self) -> LogicalPosition<X> {
    LogicalPosition {
      x: self.x.cast(),
      y: self.y.cast(),
    }
  }
}

/// A position that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Position {
  /// Physical position.
  Physical(PhysicalPosition<i32>),
  /// Logical position.
  Logical(LogicalPosition<f64>),
}

/// A size represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct PhysicalSize<T> {
  /// Width.
  pub width: T,
  /// Height.
  pub height: T,
}

impl<T: Pixel> PhysicalSize<T> {
  /// Converts the physical size to a logical one, applying the scale factor.
  #[inline]
  pub fn to_logical<X: Pixel>(self, scale_factor: f64) -> LogicalSize<X> {
    assert!(validate_scale_factor(scale_factor));
    let width = self.width.into() / scale_factor;
    let height = self.height.into() / scale_factor;
    LogicalSize { width, height }.cast()
  }
}

/// A size represented in logical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct LogicalSize<T> {
  /// Width.
  pub width: T,
  /// Height.
  pub height: T,
}

impl<T: Pixel> LogicalSize<T> {
  /// Casts the logical size to another pixel type.
  #[inline]
  pub fn cast<X: Pixel>(&self) -> LogicalSize<X> {
    LogicalSize {
      width: self.width.cast(),
      height: self.height.cast(),
    }
  }
}

/// A size that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Size {
  /// Physical size.
  Physical(PhysicalSize<u32>),
  /// Logical size.
  Logical(LogicalSize<f64>),
}
