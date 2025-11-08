# CSP Android Fix

This document describes how to fix the CSP (Content Security Policy) issue on Android devices where both `script-src` and `script-src-attr` directives cannot be used simultaneously.

## Problem

The issue occurs in WRY's Android implementation where the CSP string replacement logic has a flaw. When a CSP contains both `script-src` and `script-src-attr`, the simple `contains("script-src")` check returns true, and then `replace("script-src", ...)` replaces **ALL** occurrences of "script-src", including the one in "script-src-attr", corrupting the CSP.

For example:
- Input: `"script-src 'self'; script-src-attr 'none'"`
- After WRY's faulty replacement: `"script-src hash1 hash2-attr 'none'"` (corrupted!)

## Solution

Tauri now provides a utility module `tauri_utils::csp_android_fix` that can be used to properly handle CSP modifications in web resource request handlers.

## Usage

### Basic Usage

```rust
use tauri::utils::config::{Csp, CspDirectiveSources, WebviewUrl};
use tauri::webview::WebviewWindowBuilder;
use tauri_utils::csp_android_fix::apply_csp_fix_to_header;
use http::header::HeaderValue;
use std::collections::HashMap;

tauri::Builder::default()
  .setup(|app| {
    let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
      .on_web_resource_request(|request, response| {
        if request.uri().scheme_str() == Some("tauri") {
          // Check if we have a CSP header that needs fixing
          if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
            // Example hashes that need to be injected
            let hashes = vec![
              "'sha256-abc123def456'".to_string(),
              "'sha256-789xyz012'".to_string(),
            ];
            
            // Apply the CSP fix for Android
            if let Err(e) = apply_csp_fix_to_header(csp, &hashes) {
              eprintln!("Failed to apply CSP fix: {}", e);
            }
          }
        }
      })
      .build()?;
    Ok(())
  });
```

### Advanced Usage with Dynamic Hash Generation

```rust
use tauri::utils::config::{Csp, CspDirectiveSources, WebviewUrl};
use tauri::webview::WebviewWindowBuilder;
use tauri_utils::csp_android_fix::fix_csp_script_src;
use http::header::HeaderValue;

tauri::Builder::default()
  .setup(|app| {
    let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
      .on_web_resource_request(|request, response| {
        if request.uri().scheme_str() == Some("tauri") {
          if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
            let csp_string = csp.to_str().unwrap_or("");
            
            // Generate hashes dynamically based on your application's needs
            let mut hashes = Vec::new();
            
            // Add inline script hashes
            hashes.push("'sha256-your-inline-script-hash'".to_string());
            
            // Add nonce if needed
            if let Some(nonce) = get_current_nonce() {
              hashes.push(format!("'nonce-{}'", nonce));
            }
            
            // Apply the fix
            let fixed_csp = fix_csp_script_src(csp_string, &hashes);
            
            if let Ok(new_header) = HeaderValue::from_str(&fixed_csp) {
              *csp = new_header;
            }
          }
        }
      })
      .build()?;
    Ok(())
  });

fn get_current_nonce() -> Option<String> {
  // Your nonce generation logic here
  Some("random-nonce-value".to_string())
}
```

### Testing the Fix

You can test that the fix works correctly by checking the CSP header in your application:

```rust
use tauri_utils::csp_android_fix::fix_csp_script_src;

fn test_csp_fix() {
    let original_csp = "script-src 'self'; script-src-attr 'none'; style-src 'self'";
    let hashes = vec!["'sha256-abc123'".to_string(), "'nonce-xyz789'".to_string()];
    
    let fixed_csp = fix_csp_script_src(original_csp, &hashes);
    
    println!("Original: {}", original_csp);
    println!("Fixed:    {}", fixed_csp);
    
    // Should output:
    // Original: script-src 'self'; script-src-attr 'none'; style-src 'self'
    // Fixed:    script-src 'self' 'sha256-abc123' 'nonce-xyz789'; script-src-attr 'none'; style-src 'self'
    
    assert!(fixed_csp.contains("script-src 'self' 'sha256-abc123' 'nonce-xyz789'"));
    assert!(fixed_csp.contains("script-src-attr 'none'"));
    assert!(!fixed_csp.contains("script-src-attr 'none' 'sha256-abc123'"));
}
```

## API Reference

### `fix_csp_script_src(csp_string: &str, hashes: &[String]) -> String`

Fixes CSP string by properly handling script-src directive injection.

**Parameters:**
- `csp_string`: The original CSP string
- `hashes`: Vector of hash values to inject into script-src directive

**Returns:** Fixed CSP string with hashes properly injected into script-src directive

### `apply_csp_fix_to_header(csp_header: &mut HeaderValue, hashes: &[String]) -> Result<(), Box<dyn Error>>`

Applies the CSP fix to an HTTP header value.

**Parameters:**
- `csp_header`: Mutable reference to the CSP header value
- `hashes`: Vector of hash values to inject

**Returns:** Result indicating success or failure of the header update

## Platform Compatibility

This fix is designed to work on all platforms but is specifically needed for Android devices where the WRY CSP bug occurs. On other platforms, it will work correctly without any negative side effects.

## Related Issues

- [Tauri Issue #14429](https://github.com/tauri-apps/tauri/issues/14429): CSP configuration cannot properly use both script-src and script-src-attr simultaneously on Android devices
- [WRY CSP Android Bug](https://github.com/tauri-apps/wry): The underlying issue in WRY's Android implementation

## Contributing

If you encounter issues with this fix or have suggestions for improvements, please open an issue in the Tauri repository.