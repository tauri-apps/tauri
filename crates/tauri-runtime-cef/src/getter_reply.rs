// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::mpsc::{Receiver, RecvError};

/// A getter dispatched inline already has a reply. A queued getter on the UI
/// thread cannot wait for the same event loop to drain its request. Return an
/// error instead; worker callers still wait for their ordinary reply.
pub(crate) fn receive<T>(on_main_thread: bool, receiver: Receiver<T>) -> Result<T, RecvError> {
  if on_main_thread {
    receiver.try_recv().map_err(|_| RecvError)
  } else {
    receiver.recv()
  }
}

#[cfg(test)]
mod tests {
  use super::receive;
  use std::{sync::mpsc, thread, time::Duration};

  #[test]
  fn queued_main_thread_getter_does_not_wait_for_its_own_loop() {
    let (sender, receiver) = mpsc::channel();
    // If receive regresses to blocking, reply later so the test fails rather
    // than hanging the suite indefinitely. Keep the sender connected meanwhile.
    let delayed = thread::spawn(move || {
      thread::sleep(Duration::from_millis(50));
      let _ = sender.send(7);
    });
    assert!(receive(true, receiver).is_err());
    delayed.join().unwrap();
  }

  #[test]
  fn inline_main_thread_reply_is_returned_including_runtime_errors() {
    let (sender, receiver) = mpsc::channel();
    sender.send(Err::<(), _>("runtime-error")).unwrap();
    assert_eq!(receive(true, receiver).unwrap(), Err("runtime-error"));
  }

  #[test]
  fn worker_getter_waits_for_its_ordinary_reply() {
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || receive(false, receiver));
    sender.send(7).unwrap();
    assert_eq!(worker.join().unwrap().unwrap(), 7);
  }

  #[test]
  fn disconnected_getter_remains_an_error_on_both_threads() {
    for main in [true, false] {
      let (sender, receiver) = mpsc::channel::<()>();
      drop(sender);
      assert!(receive(main, receiver).is_err());
    }
  }
}
