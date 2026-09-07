// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::atomic::{AtomicI32, Ordering};

/// First identifier of the range reserved for the runtime's own requests.
///
/// Callers allocate below this bound, starting at 1, and the runtime allocates
/// from it up to `i32::MAX`. Each counter is bounded by its own range and fails
/// closed there, so neither can ever reach the other one, no matter how many
/// identifiers are allocated. CDP identifiers are signed 32-bit integers, so
/// both ranges stay within what the DevTools agent accepts.
const RESERVED_RANGE_START: i32 = 1_000_000_000;

static NEXT_MESSAGE_ID: AtomicI32 = AtomicI32::new(1);
static NEXT_RESERVED_MESSAGE_ID: AtomicI32 = AtomicI32::new(RESERVED_RANGE_START);

/// The process-local native DevTools request identifier space is exhausted.
#[derive(Clone, Copy, Debug)]
pub struct DevToolsMessageIdExhausted;

impl std::fmt::Display for DevToolsMessageIdExhausted {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("native CEF DevTools message identifiers are exhausted")
  }
}

impl std::error::Error for DevToolsMessageIdExhausted {}

/// Allocates one native DevTools request ID for a caller of this crate.
///
/// Observers see the whole browser, so every BrowserHost message a caller sends
/// must use this allocator to avoid consuming another caller's result. The
/// runtime allocates its own requests from a reserved range this function never
/// returns, so a caller cannot answer an internal request by picking its number.
///
/// IDs are positive and never reused, including after cancellation or browser
/// teardown. Numeric correlation does not authorize a browser or document.
pub fn allocate_devtools_message_id() -> Result<i32, DevToolsMessageIdExhausted> {
  allocate_from(&NEXT_MESSAGE_ID, RESERVED_RANGE_START - 1)
}

/// Allocates one native DevTools request ID for the runtime itself.
///
/// The identifiers come from the range reserved above every caller-visible one,
/// so internal correlation (`pending_initial_loads` and the script evaluation
/// callbacks) only ever matches requests the runtime sent, even when a caller
/// ignores [`allocate_devtools_message_id`] and hardcodes an `id`.
pub(crate) fn allocate_runtime_devtools_message_id() -> Result<i32, DevToolsMessageIdExhausted> {
  allocate_from(&NEXT_RESERVED_MESSAGE_ID, i32::MAX)
}

/// Allocates the next identifier of `counter`, which hands out values up to
/// `last`. Each range fails closed at its own boundary instead of wrapping or
/// growing into the other one.
fn allocate_from(counter: &AtomicI32, last: i32) -> Result<i32, DevToolsMessageIdExhausted> {
  counter
    .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
      if value > last {
        return None;
      }
      value.checked_add(1)
    })
    .map_err(|_| DevToolsMessageIdExhausted)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn concurrent_native_producers_never_share_a_response_id() {
    let threads = (0..4)
      .map(|_| {
        std::thread::spawn(|| {
          (0..1_000)
            .map(|_| allocate_devtools_message_id().unwrap())
            .collect::<Vec<_>>()
        })
      })
      .collect::<Vec<_>>();
    let ids = threads
      .into_iter()
      .flat_map(|thread| thread.join().unwrap())
      .collect::<Vec<_>>();
    assert!(ids.iter().all(|id| *id > 0));
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 4_000);
  }

  #[test]
  fn callers_and_the_runtime_never_share_a_response_id() {
    let caller_ids = (0..1_000)
      .map(|_| allocate_devtools_message_id().unwrap())
      .collect::<HashSet<_>>();
    let runtime_ids = (0..1_000)
      .map(|_| allocate_runtime_devtools_message_id().unwrap())
      .collect::<HashSet<_>>();
    assert!(
      caller_ids
        .iter()
        .all(|id| *id > 0 && *id < RESERVED_RANGE_START)
    );
    assert!(runtime_ids.iter().all(|id| *id >= RESERVED_RANGE_START));
    assert!(caller_ids.is_disjoint(&runtime_ids));
  }

  #[test]
  fn caller_exhaustion_cannot_reach_the_reserved_range() {
    let last = RESERVED_RANGE_START - 1;
    let counter = AtomicI32::new(last);
    assert_eq!(allocate_from(&counter, last).unwrap(), last);
    assert!(allocate_from(&counter, last).is_err());
    assert!(allocate_from(&counter, last).is_err());
    assert_eq!(counter.load(Ordering::Relaxed), RESERVED_RANGE_START);
  }

  #[test]
  fn exhaustion_cannot_reuse_a_late_response_id() {
    let counter = AtomicI32::new(i32::MAX - 1);
    assert_eq!(allocate_from(&counter, i32::MAX).unwrap(), i32::MAX - 1);
    assert!(allocate_from(&counter, i32::MAX).is_err());
    assert!(allocate_from(&counter, i32::MAX).is_err());
    assert_eq!(counter.load(Ordering::Relaxed), i32::MAX);
  }
}
