#!/usr/bin/env bash
# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

# This script verifies that critical drag-drop event handling code is present.
# It should be run as part of CI to catch regressions.

set -e

echo "🔍 Checking drag-drop event handling integrity..."

RUNTIME_WRY_LIB="crates/tauri-runtime-wry/src/lib.rs"

# Check 1: Verify proxy.send_event() is called in drag-drop handler
if grep -q "proxy.send_event(Message::Webview.*WebviewMessage.*DragDrop" "$RUNTIME_WRY_LIB"; then
    echo "✅ Drag-drop handler calls proxy.send_event()"
else
    echo "❌ CRITICAL: Drag-drop handler does NOT call proxy.send_event()"
    echo "   This means drag-drop events will NOT reach app.run() callback!"
    echo "   See crates/tauri-runtime-wry/DRAG_DROP_ARCHITECTURE.md"
    exit 1
fi

# Check 2: Verify WebviewEvent callback is invoked
if grep -q "callback(RunEvent::WebviewEvent" "$RUNTIME_WRY_LIB"; then
    echo "✅ WebviewEvent callback is invoked"
else
    echo "❌ CRITICAL: WebviewEvent callback is NOT invoked"
    echo "   This means webview events will NOT reach app.run() callback!"
    exit 1
fi

# Check 3: Verify WindowEvent callback is invoked
if grep -q "callback(RunEvent::WindowEvent" "$RUNTIME_WRY_LIB"; then
    echo "✅ WindowEvent callback is invoked"
else
    echo "❌ CRITICAL: WindowEvent callback is NOT invoked"
    echo "   This means window events will NOT reach app.run() callback!"
    exit 1
fi

# Check 4: Verify DragDropEvent has helper methods
RUNTIME_WINDOW="crates/tauri-runtime/src/window.rs"
if grep -q "pub fn paths(&self)" "$RUNTIME_WINDOW" && \
   grep -q "pub fn position(&self)" "$RUNTIME_WINDOW" && \
   grep -q "pub fn is_drop(&self)" "$RUNTIME_WINDOW"; then
    echo "✅ DragDropEvent has helper methods"
else
    echo "⚠️  WARNING: DragDropEvent may be missing helper methods"
fi

# Check 5: Verify file extension filter field exists
RUNTIME_WEBVIEW="crates/tauri-runtime/src/webview.rs"
if grep -q "drag_drop_file_extensions" "$RUNTIME_WEBVIEW"; then
    echo "✅ File extension filter field exists"
else
    echo "⚠️  WARNING: File extension filter field may be missing"
fi

# Check 6: Run integration tests
echo "🧪 Running drag-drop integration tests..."
if cargo test --package tauri-runtime-wry drag_drop --quiet 2>&1 | grep -q "test result: ok"; then
    echo "✅ Drag-drop integration tests passed"
else
    echo "❌ Drag-drop integration tests FAILED"
    echo "   Run: cargo test --package tauri-runtime-wry drag_drop"
    exit 1
fi

echo ""
echo "✅ All drag-drop integrity checks passed!"
echo "   Drag-drop events will correctly reach app.run() callback."
