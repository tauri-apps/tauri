# 🔧 Fix CSP Android Bug: Enable script-src and script-src-attr Coexistence

## 🎯 **Problem Solved**
Fixes critical Android CSP bug where `script-src` and `script-src-attr` directives cannot coexist, causing corrupted security policies.

**Before (Broken):**
```
Input:  "script-src 'self'; script-src-attr 'none'"
Output: "script-src 'self' hash-attr 'none'"  ❌ CORRUPTED
```

**After (Fixed):**
```
Input:  "script-src 'self'; script-src-attr 'none'" 
Output: "script-src 'self' hash; script-src-attr 'none'"  ✅ CORRECT
```

## 🚀 **Solution**
- **New utility module**: `tauri_utils::csp_android_fix`
- **Regex-based fix**: Precisely targets `script-src` without affecting `script-src-attr`
- **Zero breaking changes**: Non-invasive workaround until WRY upstream fix

## 📦 **What's Included**
- ✅ Core fix implementation with comprehensive tests
- ✅ Easy-to-use API: `apply_csp_fix_to_header()`
- ✅ Complete documentation and usage examples
- ✅ Working example app demonstrating the fix

## 🔧 **Usage**
```rust
use tauri_utils::csp_android_fix::apply_csp_fix_to_header;

.on_web_resource_request(|request, response| {
    if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
        let hashes = vec!["'sha256-your-hash'".to_string()];
        apply_csp_fix_to_header(csp, &hashes)?;
    }
})
```

## 🧪 **Tested Scenarios**
- ✅ Both `script-src` and `script-src-attr` present
- ✅ Only `script-src-attr` (no `script-src`)
- ✅ Multiple `script-src-*` directives
- ✅ Edge cases and error handling

## 📋 **Files Changed**
- `crates/tauri-utils/src/csp_android_fix.rs` - Core implementation
- `crates/tauri-utils/src/lib.rs` - Module export
- `examples/csp-android-fix/` - Working example
- Documentation files

**Closes #14429**

---
**Impact**: 🔥 **High** - Enables secure Android apps with proper CSP policies  
**Risk**: 🟢 **Low** - Additive change, no existing functionality affected