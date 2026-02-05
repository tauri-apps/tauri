// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! # Cross-Thread Window Creation Race Condition Fix
//!
//! ## The Bug (Fixed)
//!
//! Before this fix, when `RuntimeHandle::create_window()` was called from a
//! background thread (e.g., Tokio worker threads in Tauri command handlers):
//!
//! 1. `send_user_message()` would send `Message::CreateWindow` via the event proxy
//! 2. It would return IMMEDIATELY without waiting for the window to be created
//! 3. `create_window()` would return a `DetachedWindow` with a `WryWindowDispatcher`
//! 4. The caller would try to access window properties (e.g., `window.hwnd()`)
//! 5. This would send a `WindowMessage::RawWindowHandle` to the event loop
//! 6. The handler would look up `window_id` in the windows map
//! 7. **BUG**: Window NOT FOUND (the `CreateWindow` message hadn't been processed yet!)
//! 8. The sender `tx.send()` was never called, so `rx.recv()` would block forever
//! 9. **RESULT**: Application deadlock/freeze
//!
//! ## The Fix
//!
//! Modified `Context::create_window()` to wait for the `CreateWindow` message to be
//! fully processed before returning:
//!
//! 1. Added a completion channel (`Sender<Result<()>>`) to the `CreateWindow` message
//! 2. After `send_user_message()`, the caller waits on `rx.recv()` for completion
//! 3. The event loop handler sends the completion signal after `windows.insert()`
//! 4. Only then does `create_window()` return the `DetachedWindow`
//!
//! ## Platform Impact
//!
//! - **Windows ARM64**: This was the primary reproduction case (different thread IDs
//!   between main window and runtime threads)
//! - **All Platforms**: The fix is safe and correct on all platforms:
//!   - On main thread: `handle_user_message()` is called synchronously, the window
//!     is inserted immediately, and `rx.recv()` returns instantly
//!   - On background thread: The fix adds a small wait for event loop processing,
//!     which is the correct behavior to ensure the window exists before returning
//!
//! ## Performance Impact
//!
//! - **Main thread**: Zero overhead (synchronous path unchanged)
//! - **Background thread**: Minimal overhead - waits only for the event loop to
//!   process the single `CreateWindow` message. This is necessary for correctness
//!   and prevents the deadlock that would otherwise occur.

use std::thread;
use std::time::Duration;

use tauri_runtime::{
  window::{PendingWindow, WindowBuilder},
  EventLoopProxy, RunEvent, Runtime, RuntimeHandle, RuntimeInitArgs, WindowDispatch,
};
use tauri_runtime_wry::{WindowBuilderWrapper, Wry};

#[derive(Debug, Clone)]
enum TestEvent {
  Success(String),
  Failure(String),
}

/// Test that verifies the cross-thread window creation fix.
///
/// This test:
/// 1. Creates a runtime on the test thread
/// 2. Spawns a background thread that creates a window via `RuntimeHandle`
/// 3. Immediately tries to access `window_handle()` from the background thread
/// 4. Verifies that the access succeeds without hanging
///
/// Before the fix: This test would hang forever at `window_handle()`
/// After the fix: This test passes, window is accessible immediately
#[test]
fn test_cross_thread_window_creation() {
  println!("\n================================================================");
  println!("  Cross-Thread Window Creation - Race Condition Fix Test");
  println!("================================================================");
  println!("  This test verifies that create_window() from a background");
  println!("  thread properly waits for window creation to complete.");
  println!("================================================================\n");

  let runtime =
    Wry::<TestEvent>::new_any_thread(RuntimeInitArgs::default()).expect("Failed to create runtime");

  let handle = runtime.handle();
  let proxy = runtime.create_proxy();

  // Spawn background thread - simulates Tokio worker thread in Tauri commands
  let _bg_thread = thread::spawn(move || {
    println!(
      "[BG] Background thread started: {:?}",
      thread::current().id()
    );

    // Small delay to ensure event loop is running
    thread::sleep(Duration::from_millis(50));

    let pending = match PendingWindow::<TestEvent, Wry<TestEvent>>::new(
      WindowBuilderWrapper::new(),
      "test-window",
    ) {
      Ok(p) => p,
      Err(e) => {
        let _ = proxy.send_event(TestEvent::Failure(format!("PendingWindow failed: {:?}", e)));
        return;
      }
    };

    println!("[BG] Calling create_window() from background thread...");

    match handle.create_window(
      pending,
      Option::<Box<dyn Fn(tauri_runtime::window::RawWindow<'_>) + Send>>::None,
    ) {
      Ok(window) => {
        println!("[BG] create_window() returned successfully");
        println!("[BG] Window ID: {:?}", window.id);

        // This is the critical test - window_handle() should work immediately
        // Before the fix, this would hang forever
        println!("[BG] Calling window_handle() - should NOT hang...");

        match window.dispatcher.window_handle() {
          Ok(_) => {
            println!("[BG] window_handle() succeeded!");
            let _ = proxy.send_event(TestEvent::Success(
              "Window created and accessible from background thread".into(),
            ));
          }
          Err(e) => {
            let _ = proxy.send_event(TestEvent::Failure(format!(
              "window_handle() failed: {:?}",
              e
            )));
          }
        }
      }
      Err(e) => {
        let _ = proxy.send_event(TestEvent::Failure(format!(
          "create_window() failed: {:?}",
          e
        )));
      }
    }
  });

  println!(
    "[MAIN] Starting event loop on thread: {:?}",
    thread::current().id()
  );

  runtime.run(move |event| match event {
    RunEvent::UserEvent(TestEvent::Success(msg)) => {
      println!("\n================================================================");
      println!("[RESULT]  PASS: {}", msg);
      println!("================================================================\n");
      std::process::exit(0);
    }
    RunEvent::UserEvent(TestEvent::Failure(msg)) => {
      println!("\n================================================================");
      println!("[RESULT]  FAIL: {}", msg);
      println!("================================================================\n");
      std::process::exit(1);
    }
    _ => {}
  });
}
