# CSP Android Fix Example

This example demonstrates how to use the CSP (Content Security Policy) Android fix utility to resolve the issue where both `script-src` and `script-src-attr` directives cannot be used simultaneously on Android devices.

## Problem

The issue occurs in WRY's Android implementation where the CSP string replacement logic has a flaw. When a CSP contains both `script-src` and `script-src-attr`, the simple `contains("script-src")` check returns true, and then `replace("script-src", ...)` replaces **ALL** occurrences of "script-src", including the one in "script-src-attr", corrupting the CSP.

## Solution

This example shows how to use the `tauri_utils::csp_android_fix` module to properly handle CSP modifications.

## Running the Example

```bash
# From the tauri root directory
cd examples/csp-android-fix
cargo run
```

## Key Features Demonstrated

1. **Proper CSP Handling**: Shows how to use `apply_csp_fix_to_header` in web resource request handlers
2. **Test Cases**: Includes comprehensive test cases that verify the fix works correctly
3. **Real-world Usage**: Demonstrates how to integrate the fix into a Tauri application

## Files

- `main.rs` - Main application code with CSP fix implementation
- `tauri.conf.json` - Tauri configuration with CSP settings
- `index.html` - Frontend demonstrating the fix
- `README.md` - This documentation

## Related

- [CSP_ANDROID_FIX.md](../../CSP_ANDROID_FIX.md) - Comprehensive documentation
- [Issue #14429](https://github.com/tauri-apps/tauri/issues/14429) - Original bug report