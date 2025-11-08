// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Example demonstrating the CSP Android fix
//! 
//! This example shows how to use the CSP Android fix utility to properly handle
//! CSP headers when both script-src and script-src-attr directives are present.

use tauri::utils::config::WebviewUrl;
use tauri::webview::WebviewWindowBuilder;
use tauri_utils::csp_android_fix::apply_csp_fix_to_header;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let webview_window = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .on_web_resource_request(|request, response| {
                // Only process tauri:// protocol requests
                if request.uri().scheme_str() == Some("tauri") {
                    // Check if we have a CSP header that might need fixing
                    if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
                        println!("Original CSP: {:?}", csp);

                        // Example hashes that need to be injected into script-src
                        // In a real application, these would be dynamically generated
                        // based on your inline scripts and other security requirements
                        let hashes = vec![
                            "'sha256-abc123def456789'".to_string(),
                            "'sha256-xyz789abc123def'".to_string(),
                            "'nonce-random-value-123'".to_string(),
                        ];

                        // Apply the CSP fix for Android
                        match apply_csp_fix_to_header(csp, &hashes) {
                            Ok(()) => {
                                println!("CSP fix applied successfully");
                                println!("Fixed CSP: {:?}", csp);
                            }
                            Err(e) => {
                                eprintln!("Failed to apply CSP fix: {}", e);
                            }
                        }
                    }
                }
            })
            .build()?;

            println!("Application started with CSP Android fix enabled");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_utils::csp_android_fix::fix_csp_script_src;

    #[test]
    fn test_csp_android_fix_example() {
        // Test case 1: CSP with both script-src and script-src-attr
        let original_csp = "script-src 'self'; script-src-attr 'none'; style-src 'self'";
        let hashes = vec![
            "'sha256-abc123'".to_string(),
            "'nonce-xyz789'".to_string(),
        ];

        let fixed_csp = fix_csp_script_src(original_csp, &hashes);

        println!("Test 1:");
        println!("  Original: {}", original_csp);
        println!("  Fixed:    {}", fixed_csp);

        // Verify that script-src was modified correctly
        assert!(fixed_csp.contains("script-src 'self' 'sha256-abc123' 'nonce-xyz789'"));
        // Verify that script-src-attr was NOT modified
        assert!(fixed_csp.contains("script-src-attr 'none'"));
        // Verify that script-src-attr doesn't contain the hashes
        assert!(!fixed_csp.contains("script-src-attr 'none' 'sha256-abc123'"));

        // Test case 2: CSP with only script-src-attr (no script-src)
        let original_csp2 = "style-src 'self'; script-src-attr 'none'";
        let fixed_csp2 = fix_csp_script_src(original_csp2, &hashes);

        println!("\nTest 2:");
        println!("  Original: {}", original_csp2);
        println!("  Fixed:    {}", fixed_csp2);

        // Should add script-src at the end
        assert!(fixed_csp2.contains("script-src 'sha256-abc123' 'nonce-xyz789'"));
        // Should preserve script-src-attr
        assert!(fixed_csp2.contains("script-src-attr 'none'"));

        // Test case 3: CSP with multiple script-src-* directives
        let original_csp3 = "script-src 'self'; script-src-attr 'none'; script-src-elem 'self'";
        let fixed_csp3 = fix_csp_script_src(original_csp3, &hashes);

        println!("\nTest 3:");
        println!("  Original: {}", original_csp3);
        println!("  Fixed:    {}", fixed_csp3);

        // Should only modify script-src
        assert!(fixed_csp3.contains("script-src 'self' 'sha256-abc123' 'nonce-xyz789'"));
        assert!(fixed_csp3.contains("script-src-attr 'none'"));
        assert!(fixed_csp3.contains("script-src-elem 'self'"));
        // Verify no cross-contamination
        assert!(!fixed_csp3.contains("script-src-attr 'none' 'sha256-abc123'"));
        assert!(!fixed_csp3.contains("script-src-elem 'self' 'sha256-abc123'"));
    }
}