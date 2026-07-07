use std::sync::atomic::{AtomicU32, Ordering};

pub struct Counter(AtomicU32);

impl Counter {
  pub const fn new() -> Self {
    Self(AtomicU32::new(1))
  }

  pub fn next(&self) -> u32 {
    self.0.fetch_add(1, Ordering::Relaxed)
  }
}
