#![allow(missing_docs)]

use std::sync::atomic::{AtomicU32, Ordering};

pub struct AtomicCounter(AtomicU32);

impl AtomicCounter {
  pub const fn new() -> Self {
    Self(AtomicU32::new(1))
  }

  pub const fn new_with_start(start: u32) -> Self {
    Self(AtomicU32::new(start))
  }

  pub fn next(&self) -> u32 {
    self.0.fetch_add(1, Ordering::Relaxed)
  }

  pub fn current(&self) -> u32 {
    self.0.load(Ordering::Relaxed)
  }
}
