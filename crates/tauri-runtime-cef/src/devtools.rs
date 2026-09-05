// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_MESSAGE_ID: AtomicI32 = AtomicI32::new(1);

/// The process-local native DevTools request identifier space is exhausted.
#[derive(Clone, Copy, Debug)]
pub struct DevToolsMessageIdExhausted;

impl std::fmt::Display for DevToolsMessageIdExhausted {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("native CEF DevTools message identifiers are exhausted")
  }
}

impl std::error::Error for DevToolsMessageIdExhausted {}

/// Allocates one native DevTools request ID shared by the runtime and its
/// callers. All BrowserHost messages observed through `on_dev_tools_protocol`
/// must use this allocator to avoid consuming another producer's result.
///
/// IDs are positive and never reused, including after cancellation or browser
/// teardown. Numeric correlation does not authorize a browser or document.
pub fn allocate_devtools_message_id() -> Result<i32, DevToolsMessageIdExhausted> {
  allocate_from(&NEXT_MESSAGE_ID)
}

fn allocate_from(counter: &AtomicI32) -> Result<i32, DevToolsMessageIdExhausted> {
  counter
    .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
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
  fn exhaustion_cannot_reuse_a_late_response_id() {
    let counter = AtomicI32::new(i32::MAX - 1);
    assert_eq!(allocate_from(&counter).unwrap(), i32::MAX - 1);
    assert!(allocate_from(&counter).is_err());
    assert!(allocate_from(&counter).is_err());
    assert_eq!(counter.load(Ordering::Relaxed), i32::MAX);
  }
}
