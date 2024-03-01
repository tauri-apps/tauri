// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

pub trait Pixel: Copy + Into<f64> {
  fn from_f64(f: f64) -> Self;
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
  fn from_f64(f: f64) -> Self {
    f
  }
}

/// Checks that the scale factor is a normal positive `f64`.
///
/// All functions that take a scale factor assert that this will return `true`. If you're sourcing scale factors from
/// anywhere other than tao, it's recommended to validate them using this function before passing them to tao;
/// otherwise, you risk panics.
#[inline]
pub fn validate_scale_factor(scale_factor: f64) -> bool {
  scale_factor.is_sign_positive() && scale_factor.is_normal()
}

/// A position represented in logical pixels.
///
/// The position is stored as floats, so please be careful. Casting floats to integers truncates the
/// fractional part, which can cause noticeable issues. To help with that, an `Into<(i32, i32)>`
/// implementation is provided which does the rounding for you.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct LogicalPosition<P> {
  pub x: P,
  pub y: P,
}

impl<P> LogicalPosition<P> {
  #[inline]
  pub const fn new(x: P, y: P) -> Self {
    LogicalPosition { x, y }
  }
}

impl<P: Pixel> LogicalPosition<P> {
  #[inline]
  pub fn from_physical<T: Into<PhysicalPosition<X>>, X: Pixel>(
    physical: T,
    scale_factor: f64,
  ) -> Self {
    physical.into().to_logical(scale_factor)
  }

  #[inline]
  pub fn to_physical<X: Pixel>(&self, scale_factor: f64) -> PhysicalPosition<X> {
    assert!(validate_scale_factor(scale_factor));
    let x = self.x.into() * scale_factor;
    let y = self.y.into() * scale_factor;
    PhysicalPosition::new(x, y).cast()
  }

  #[inline]
  pub fn cast<X: Pixel>(&self) -> LogicalPosition<X> {
    LogicalPosition {
      x: self.x.cast(),
      y: self.y.cast(),
    }
  }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for LogicalPosition<P> {
  fn from((x, y): (X, X)) -> LogicalPosition<P> {
    LogicalPosition::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<LogicalPosition<P>> for (X, X) {
  fn from(pos: LogicalPosition<P>) -> Self {
    (pos.x.cast(), pos.y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for LogicalPosition<P> {
  fn from([x, y]: [X; 2]) -> LogicalPosition<P> {
    LogicalPosition::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<LogicalPosition<P>> for [X; 2] {
  fn from(pos: LogicalPosition<P>) -> Self {
    [pos.x.cast(), pos.y.cast()]
  }
}

/// A position represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct PhysicalPosition<P> {
  pub x: P,
  pub y: P,
}

impl<P> PhysicalPosition<P> {
  #[inline]
  pub const fn new(x: P, y: P) -> Self {
    PhysicalPosition { x, y }
  }
}

impl<P: Pixel> PhysicalPosition<P> {
  #[inline]
  pub fn from_logical<T: Into<LogicalPosition<X>>, X: Pixel>(
    logical: T,
    scale_factor: f64,
  ) -> Self {
    logical.into().to_physical(scale_factor)
  }

  #[inline]
  pub fn to_logical<X: Pixel>(&self, scale_factor: f64) -> LogicalPosition<X> {
    assert!(validate_scale_factor(scale_factor));
    let x = self.x.into() / scale_factor;
    let y = self.y.into() / scale_factor;
    LogicalPosition::new(x, y).cast()
  }

  #[inline]
  pub fn cast<X: Pixel>(&self) -> PhysicalPosition<X> {
    PhysicalPosition {
      x: self.x.cast(),
      y: self.y.cast(),
    }
  }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for PhysicalPosition<P> {
  fn from((x, y): (X, X)) -> PhysicalPosition<P> {
    PhysicalPosition::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<PhysicalPosition<P>> for (X, X) {
  fn from(pos: PhysicalPosition<P>) -> Self {
    (pos.x.cast(), pos.y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for PhysicalPosition<P> {
  fn from([x, y]: [X; 2]) -> PhysicalPosition<P> {
    PhysicalPosition::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<PhysicalPosition<P>> for [X; 2] {
  fn from(pos: PhysicalPosition<P>) -> Self {
    [pos.x.cast(), pos.y.cast()]
  }
}

/// A size represented in logical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct LogicalSize<P> {
  pub width: P,
  pub height: P,
}

impl<P> LogicalSize<P> {
  #[inline]
  pub const fn new(width: P, height: P) -> Self {
    LogicalSize { width, height }
  }
}

impl<P: Pixel> LogicalSize<P> {
  #[inline]
  pub fn from_physical<T: Into<PhysicalSize<X>>, X: Pixel>(physical: T, scale_factor: f64) -> Self {
    physical.into().to_logical(scale_factor)
  }

  #[inline]
  pub fn to_physical<X: Pixel>(&self, scale_factor: f64) -> PhysicalSize<X> {
    assert!(validate_scale_factor(scale_factor));
    let width = self.width.into() * scale_factor;
    let height = self.height.into() * scale_factor;
    PhysicalSize::new(width, height).cast()
  }

  #[inline]
  pub fn cast<X: Pixel>(&self) -> LogicalSize<X> {
    LogicalSize {
      width: self.width.cast(),
      height: self.height.cast(),
    }
  }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for LogicalSize<P> {
  fn from((x, y): (X, X)) -> LogicalSize<P> {
    LogicalSize::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<LogicalSize<P>> for (X, X) {
  fn from(size: LogicalSize<P>) -> Self {
    (size.width.cast(), size.height.cast())
  }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for LogicalSize<P> {
  fn from([x, y]: [X; 2]) -> LogicalSize<P> {
    LogicalSize::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<LogicalSize<P>> for [X; 2] {
  fn from(size: LogicalSize<P>) -> Self {
    [size.width.cast(), size.height.cast()]
  }
}

/// A size represented in physical pixels.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize)]
pub struct PhysicalSize<P> {
  pub width: P,
  pub height: P,
}

impl<P> PhysicalSize<P> {
  #[inline]
  pub const fn new(width: P, height: P) -> Self {
    PhysicalSize { width, height }
  }
}

impl<P: Pixel> PhysicalSize<P> {
  #[inline]
  pub fn from_logical<T: Into<LogicalSize<X>>, X: Pixel>(logical: T, scale_factor: f64) -> Self {
    logical.into().to_physical(scale_factor)
  }

  #[inline]
  pub fn to_logical<X: Pixel>(&self, scale_factor: f64) -> LogicalSize<X> {
    assert!(validate_scale_factor(scale_factor));
    let width = self.width.into() / scale_factor;
    let height = self.height.into() / scale_factor;
    LogicalSize::new(width, height).cast()
  }

  #[inline]
  pub fn cast<X: Pixel>(&self) -> PhysicalSize<X> {
    PhysicalSize {
      width: self.width.cast(),
      height: self.height.cast(),
    }
  }
}

impl<P: Pixel, X: Pixel> From<(X, X)> for PhysicalSize<P> {
  fn from((x, y): (X, X)) -> PhysicalSize<P> {
    PhysicalSize::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<PhysicalSize<P>> for (X, X) {
  fn from(size: PhysicalSize<P>) -> Self {
    (size.width.cast(), size.height.cast())
  }
}

impl<P: Pixel, X: Pixel> From<[X; 2]> for PhysicalSize<P> {
  fn from([x, y]: [X; 2]) -> PhysicalSize<P> {
    PhysicalSize::new(x.cast(), y.cast())
  }
}

impl<P: Pixel, X: Pixel> From<PhysicalSize<P>> for [X; 2] {
  fn from(size: PhysicalSize<P>) -> Self {
    [size.width.cast(), size.height.cast()]
  }
}

/// A size that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Size {
  Physical(PhysicalSize<u32>),
  Logical(LogicalSize<f64>),
}

impl Size {
  pub fn new<S: Into<Size>>(size: S) -> Size {
    size.into()
  }

  pub fn to_logical<P: Pixel>(&self, scale_factor: f64) -> LogicalSize<P> {
    match *self {
      Size::Physical(size) => size.to_logical(scale_factor),
      Size::Logical(size) => size.cast(),
    }
  }

  pub fn to_physical<P: Pixel>(&self, scale_factor: f64) -> PhysicalSize<P> {
    match *self {
      Size::Physical(size) => size.cast(),
      Size::Logical(size) => size.to_physical(scale_factor),
    }
  }
}

impl<P: Pixel> From<PhysicalSize<P>> for Size {
  #[inline]
  fn from(size: PhysicalSize<P>) -> Size {
    Size::Physical(size.cast())
  }
}

impl<P: Pixel> From<LogicalSize<P>> for Size {
  #[inline]
  fn from(size: LogicalSize<P>) -> Size {
    Size::Logical(size.cast())
  }
}

/// A position that's either physical or logical.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Position {
  Physical(PhysicalPosition<i32>),
  Logical(LogicalPosition<f64>),
}

impl Position {
  pub fn new<S: Into<Position>>(position: S) -> Position {
    position.into()
  }

  pub fn to_logical<P: Pixel>(&self, scale_factor: f64) -> LogicalPosition<P> {
    match *self {
      Position::Physical(position) => position.to_logical(scale_factor),
      Position::Logical(position) => position.cast(),
    }
  }

  pub fn to_physical<P: Pixel>(&self, scale_factor: f64) -> PhysicalPosition<P> {
    match *self {
      Position::Physical(position) => position.cast(),
      Position::Logical(position) => position.to_physical(scale_factor),
    }
  }
}

impl<P: Pixel> From<PhysicalPosition<P>> for Position {
  #[inline]
  fn from(position: PhysicalPosition<P>) -> Position {
    Position::Physical(position.cast())
  }
}

impl<P: Pixel> From<LogicalPosition<P>> for Position {
  #[inline]
  fn from(position: LogicalPosition<P>) -> Position {
    Position::Logical(position.cast())
  }
}
