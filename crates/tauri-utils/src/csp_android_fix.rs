// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! CSP Android Fix Module
//! 
//! This module provides utilities to fix the CSP (Content Security Policy) issue
//! on Android devices where both `script-src` and `script-src-attr` directives
//! cannot be used simultaneously due to a bug in WRY's Android implementation.
//! 
//! Issue: https://github.com/tauri-apps/tauri/issues/14429

use regex::Regex;
use std::sync::OnceLock;

/// Regex pattern to match script-src directive as a complete word
static SCRIPT_SRC_REGEX: OnceLock<Regex> = OnceLock::new();

/// Get the compiled regex for matching script-src directive
fn get_script_src_regex() -> &'static Regex {
    SCRIPT_SRC_REGEX.get_or_init(|| {
        Regex::new(r"(?P<prefix>^|;|\s)script-src(?P<suffix>\s|;|$)")
            .expect("Failed to compile script-src regex")
    })
}

/// Fixes CSP string by properly handling script-src directive injection
/// 
/// This function addresses the Android CSP bug where simple string replacement
/// of "script-src" would incorrectly modify "script-src-attr" as well.
/// 
/// # Arguments
/// 
/// * `csp_string` - The original CSP string
/// * `hashes` - Vector of hash values to inject into script-src directive
/// 
/// # Returns
/// 
/// Fixed CSP string with hashes properly injected into script-src directive
/// 
/// # Example
/// 
/// ```rust
/// use tauri_utils::csp_android_fix::fix_csp_script_src;
/// 
/// let original_csp = "script-src 'self'; script-src-attr 'none'";
/// let hashes = vec!["'sha256-abc123'", "'sha256-def456'"];
/// let fixed_csp = fix_csp_script_src(original_csp, &hashes);
/// 
/// assert_eq!(fixed_csp, "script-src 'self' 'sha256-abc123' 'sha256-def456'; script-src-attr 'none'");
/// ```
pub fn fix_csp_script_src(csp_string: &str, hashes: &[String]) -> String {
    let re = get_script_src_regex();
    
    if re.is_match(csp_string) {
        re.replace_all(csp_string, |caps: &regex::Captures| {
            let prefix = &caps["prefix"];
            let suffix = &caps["suffix"];
            format!("{}script-src {}{}", prefix, hashes.join(" "), suffix)
        }).into_owned()
    } else {
        format!("{} script-src {}", csp_string, hashes.join(" "))
    }
}

/// Applies the CSP fix to an HTTP header value
/// 
/// This is a convenience function for use with HTTP responses in web resource handlers.
/// 
/// # Arguments
/// 
/// * `csp_header` - Mutable reference to the CSP header value
/// * `hashes` - Vector of hash values to inject
/// 
/// # Returns
/// 
/// Result indicating success or failure of the header update
/// 
/// # Example
/// 
/// ```rust
/// use http::HeaderValue;
/// use tauri_utils::csp_android_fix::apply_csp_fix_to_header;
/// 
/// let mut csp_header = HeaderValue::from_static("script-src 'self'; script-src-attr 'none'");
/// let hashes = vec!["'sha256-abc123'".to_string()];
/// 
/// apply_csp_fix_to_header(&mut csp_header, &hashes).unwrap();
/// ```
pub fn apply_csp_fix_to_header(
    csp_header: &mut http::HeaderValue,
    hashes: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let csp_string = csp_header.to_str()?.to_string();
    let fixed_csp = fix_csp_script_src(&csp_string, hashes);
    *csp_header = http::HeaderValue::from_str(&fixed_csp)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_csp_with_both_directives() {
        let original = "script-src 'self'; script-src-attr 'none'";
        let hashes = vec!["'sha256-abc123'".to_string(), "'sha256-def456'".to_string()];
        let result = fix_csp_script_src(original, &hashes);
        
        assert_eq!(result, "script-src 'self' 'sha256-abc123' 'sha256-def456'; script-src-attr 'none'");
    }

    #[test]
    fn test_fix_csp_only_script_src() {
        let original = "script-src 'self'";
        let hashes = vec!["'sha256-abc123'".to_string()];
        let result = fix_csp_script_src(original, &hashes);
        
        assert_eq!(result, "script-src 'self' 'sha256-abc123'");
    }

    #[test]
    fn test_fix_csp_no_script_src() {
        let original = "style-src 'self'; script-src-attr 'none'";
        let hashes = vec!["'sha256-abc123'".to_string()];
        let result = fix_csp_script_src(original, &hashes);
        
        assert_eq!(result, "style-src 'self'; script-src-attr 'none' script-src 'sha256-abc123'");
    }

    #[test]
    fn test_fix_csp_multiple_script_src_attr() {
        let original = "script-src 'self'; script-src-attr 'none'; script-src-elem 'self'";
        let hashes = vec!["'sha256-abc123'".to_string()];
        let result = fix_csp_script_src(original, &hashes);
        
        // Should only modify script-src, not script-src-attr or script-src-elem
        assert_eq!(result, "script-src 'self' 'sha256-abc123'; script-src-attr 'none'; script-src-elem 'self'");
    }

    #[test]
    fn test_fix_csp_empty_hashes() {
        let original = "script-src 'self'; script-src-attr 'none'";
        let hashes: Vec<String> = vec![];
        let result = fix_csp_script_src(original, &hashes);
        
        assert_eq!(result, "script-src 'self' ; script-src-attr 'none'");
    }

    #[test]
    fn test_apply_csp_fix_to_header() {
        let mut header = http::HeaderValue::from_static("script-src 'self'; script-src-attr 'none'");
        let hashes = vec!["'sha256-abc123'".to_string()];
        
        apply_csp_fix_to_header(&mut header, &hashes).unwrap();
        
        assert_eq!(
            header.to_str().unwrap(),
            "script-src 'self' 'sha256-abc123'; script-src-attr 'none'"
        );
    }
}