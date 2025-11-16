# Drag and Drop Event Architecture

## Critical Implementation Details

This document describes the **critical path** for drag-drop events to reach the `app.run()` callback. 
**DO NOT MODIFY** this flow without understanding the consequences.

## The Bug (Tauri v1)

In Tauri v1, drag-drop events would reach the **frontend** JavaScript listeners but **NOT** the backend `app.run()` callback. This was caused by the drag-drop handler calling event listeners directly instead of routing through the event loop.

**Fixed in:** Commit `cb640c8e9` (PR #8393, December 28, 2023)

## The Critical Path

For drag-drop events to work correctly, they MUST follow this exact path:

```
1. User drags/drops files
   ↓
2. wry captures the native OS event
   ↓
3. with_drag_drop_handler closure is invoked (lib.rs ~line 4646)
   ↓
4. Event is converted from WryDragDropEvent to DragDropEvent
   ↓
5. **CRITICAL**: proxy.send_event() is called (lib.rs ~line 4727)
   ↓
6. Event loop processes Message::Webview (lib.rs ~line 4034 or ~line 4060)
   ↓
7. **CRITICAL**: callback(RunEvent::WebviewEvent) or callback(RunEvent::WindowEvent) is invoked
   ↓
8. User's app.run(|app_handle, event| { ... }) receives the event ✅
```

## Code Locations

### 1. Drag-Drop Handler Registration
**File:** `crates/tauri-runtime-wry/src/lib.rs`
**Line:** ~4646-4735

```rust
if webview_attributes.drag_drop_handler_enabled {
  let proxy = context.proxy.clone();  // Must clone the proxy!
  webview_builder = webview_builder.with_drag_drop_handler(move |event| {
    // Convert event...
    
    // ⚠️  CRITICAL: This line MUST be present
    proxy.send_event(Message::Webview(window_id, id, message));
    // ⚠️  Without this, events won't reach app.run()
    
    true
  });
}
```

### 2. Event Loop Processing (WebviewEvent)
**File:** `crates/tauri-runtime-wry/src/lib.rs`
**Line:** ~4034-4056

```rust
Event::UserEvent(Message::Webview(
  window_id,
  webview_id,
  WebviewMessage::WebviewEvent(event),
)) => {
  // ...
  
  // ⚠️  CRITICAL: This callback delivers events to app.run()
  callback(RunEvent::WebviewEvent { label, event: event.clone() });
  // ⚠️  Without this, events won't reach user code
}
```

### 3. Event Loop Processing (WindowEvent)
**File:** `crates/tauri-runtime-wry/src/lib.rs`
**Line:** ~4060-4084

```rust
Event::UserEvent(Message::Webview(
  window_id,
  _webview_id,
  WebviewMessage::SynthesizedWindowEvent(event),
)) => {
  // ...
  
  // ⚠️  CRITICAL: This callback delivers events to app.run()
  callback(RunEvent::WindowEvent { label, event: event.clone() });
  // ⚠️  Without this, events won't reach user code
}
```

### 4. App Event Conversion
**File:** `crates/tauri/src/app.rs`
**Line:** ~2407-2413

```rust
RuntimeRunEvent::WindowEvent { label, event } => RunEvent::WindowEvent {
  label,
  event: event.into(),
},
RuntimeRunEvent::WebviewEvent { label, event } => RunEvent::WebviewEvent {
  label,
  event: event.into(),
},
```

## How to Verify the Fix

### 1. Check Integration Tests
Run the drag-drop integration tests:
```bash
cargo test --package tauri-runtime-wry drag_drop
```

### 2. Manual Testing
```rust
use tauri::RunEvent;

tauri::Builder::default()
  .build(tauri::generate_context!())
  .expect("error while building tauri application")
  .run(|_app_handle, event| {
    if let RunEvent::WindowEvent { event: tauri::WindowEvent::DragDrop(drop_event), .. } = event {
      println!("✅ Drag-drop event received in app.run(): {:?}", drop_event);
    }
  });
```

If you see the log message when dropping files, the fix is working.
If you DON'T see the log message, the bug has been reintroduced.

### 3. Enable Debug Logging
```bash
RUST_LOG=debug cargo run
```

Look for these log messages:
- `"Drag-drop event received"` - Handler captured the event
- `"Drag-drop event sent to event loop successfully"` - Event was sent via proxy
- `"Processing WebviewEvent"` or `"Processing SynthesizedWindowEvent"` - Event loop received it
- `"WebviewEvent callback invoked successfully"` - Callback was called

If any of these are missing, the critical path is broken.

## Common Pitfalls

### ❌ DON'T: Call listeners directly without using proxy
```rust
// WRONG - This was the v1 bug!
webview_builder.with_drag_drop_handler(|event| {
  // Convert event...
  for listener in listeners {
    listener(&event);  // ❌ Events only go to local listeners
  }
  true
});
```

### ✅ DO: Always use proxy.send_event()
```rust
// CORRECT - Events go through event loop to app.run()
webview_builder.with_drag_drop_handler(move |event| {
  // Convert event...
  proxy.send_event(Message::Webview(..., message));  // ✅ Events reach app.run()
  true
});
```

### ❌ DON'T: Remove the callback() invocation
```rust
// WRONG - Events won't reach user code!
Event::UserEvent(Message::Webview(_, _, WebviewMessage::WebviewEvent(event))) => {
  // Process event...
  // callback(RunEvent::WebviewEvent { ... });  // ❌ Commented out!
}
```

### ✅ DO: Always invoke the callback
```rust
// CORRECT - Events reach app.run()
Event::UserEvent(Message::Webview(_, _, WebviewMessage::WebviewEvent(event))) => {
  callback(RunEvent::WebviewEvent { label, event });  // ✅ User code receives event
}
```

## Testing Checklist

Before merging any PR that touches drag-drop code:

- [ ] Run integration tests: `cargo test drag_drop`
- [ ] Manually test drag-drop in example app
- [ ] Verify events reach app.run() callback
- [ ] Check debug logs show full event flow
- [ ] Test with file extension filters
- [ ] Test on all platforms (Windows, macOS, Linux)

## Related Issues & PRs

- Original bug report: #8206
- Fix PR: #8393
- Commit: cb640c8e9

## New Features (v2.9.3+)

### File Extension Filtering
You can now filter drag-drop events by file extension:

```rust
WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
  .drag_drop_file_extensions(vec!["png".to_string(), "jpg".to_string()])
  .build()?;
```

This is implemented in the drag-drop handler BEFORE sending to the event loop,
so filtered files never even enter the application.

### Helper Methods
```rust
if event.is_drop() {
  let paths = event.paths().unwrap();
  // Handle file drop...
}
```

## Maintainer Notes

If you're debugging why drag-drop events aren't reaching `app.run()`:

1. Enable tracing: Add `features = ["tracing"]` to `tauri-runtime-wry`
2. Set `RUST_LOG=debug`
3. Trace the event flow through the logs
4. Check each critical step is executing
5. If any step is missing, you've found the bug

**Remember:** Both the proxy.send_event() AND the callback() invocation are REQUIRED.
Removing either will break drag-drop event delivery to the backend.
