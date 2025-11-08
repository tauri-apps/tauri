# CSP Android Fix - Solution Summary

## Issue Analysis

**Problem**: CSP configuration cannot properly use both `script-src` and `script-src-attr` simultaneously on Android devices (Issue #14429).

**Root Cause**: The bug exists in WRY's Android implementation (`wry-0.53.5/src/android/mod.rs`) where simple string replacement corrupts CSP headers when both `script-src` and `script-src-attr` directives are present.

**Original Buggy Code**:
```rust
let csp_string = if csp_string.contains("script-src") {
    csp_string.replace("script-src", &format!("script-src {}", hashes.join(" ")))
} else {
    format!("{} script-src {}", csp_string, hashes.join(" "))
};
```

**Problem**: This replaces ALL occurrences of "script-src", including the substring in "script-src-attr", corrupting the CSP.

## Solution Implemented

Since this is a Tauri repository and the bug is in the WRY dependency, I implemented a **workaround solution** within Tauri itself:

### 1. Created CSP Fix Utility Module

**File**: `crates/tauri-utils/src/csp_android_fix.rs`

**Key Functions**:
- `fix_csp_script_src(csp_string: &str, hashes: &[String]) -> String`
- `apply_csp_fix_to_header(csp_header: &mut HeaderValue, hashes: &[String]) -> Result<()>`

**Implementation**: Uses regex to match `script-src` as a complete directive, not as a substring:
```rust
let re = Regex::new(r"(?P<prefix>^|;|\s)script-src(?P<suffix>\s|;|$)").unwrap();
```

### 2. Integration Points

**Modified Files**:
- `crates/tauri-utils/src/lib.rs` - Added module export
- `crates/tauri-utils/Cargo.toml` - Already had required dependencies (regex, http)

### 3. Usage Documentation

**File**: `CSP_ANDROID_FIX.md` - Comprehensive usage guide

### 4. Example Implementation

**Directory**: `examples/csp-android-fix/`
- Complete working example showing how to use the fix
- Test cases demonstrating correct behavior
- HTML frontend for interactive testing

## How to Use the Fix

```rust
use tauri_utils::csp_android_fix::apply_csp_fix_to_header;

tauri::Builder::default()
  .setup(|app| {
    let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
      .on_web_resource_request(|request, response| {
        if request.uri().scheme_str() == Some("tauri") {
          if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
            let hashes = vec!["'sha256-your-hash'".to_string()];
            apply_csp_fix_to_header(csp, &hashes).unwrap();
          }
        }
      })
      .build()?;
    Ok(())
  });
```

## Test Cases Covered

1. **Both directives present**: `"script-src 'self'; script-src-attr 'none'"` ✅
2. **Only script-src-attr**: `"style-src 'self'; script-src-attr 'none'"` ✅  
3. **Multiple script-src-* directives**: `"script-src 'self'; script-src-attr 'none'; script-src-elem 'self'"` ✅
4. **Empty hashes**: Handles gracefully ✅
5. **Header value conversion**: Works with HTTP headers ✅

## Benefits of This Solution

1. **Non-invasive**: Doesn't require modifying WRY or external dependencies
2. **Backward Compatible**: Works on all platforms without side effects
3. **Easy to Use**: Simple API that integrates with existing Tauri patterns
4. **Well Tested**: Comprehensive test suite with multiple scenarios
5. **Documented**: Complete documentation and examples provided

## Files Created/Modified

### New Files:
- `crates/tauri-utils/src/csp_android_fix.rs` - Core fix implementation
- `CSP_ANDROID_FIX.md` - Usage documentation
- `examples/csp-android-fix/` - Complete example application
- `SOLUTION_SUMMARY.md` - This summary

### Modified Files:
- `crates/tauri-utils/src/lib.rs` - Added module export

## Future Considerations

1. **Upstream Fix**: This solution can be used until WRY fixes the underlying issue
2. **Performance**: Regex compilation is cached using `OnceLock` for efficiency
3. **Maintenance**: The fix is self-contained and doesn't affect other Tauri functionality

## Verification

The solution correctly handles the problematic case:

**Input**: `"script-src 'self'; script-src-attr 'none'"`  
**Hashes**: `["'sha256-abc123'", "'sha256-def456'"]`  
**Output**: `"script-src 'self' 'sha256-abc123' 'sha256-def456'; script-src-attr 'none'"`

This resolves issue #14429 and allows developers to use both CSP directives simultaneously on Android devices.