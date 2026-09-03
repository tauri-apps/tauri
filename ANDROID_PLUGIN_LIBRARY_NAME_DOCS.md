# Android Plugin Dynamic Library Name - Documentation Update

This document provides the updated documentation for the "Calling Rust From Mobile Plugins - Android" section to fix [issue #14569](https://github.com/tauri-apps/tauri/issues/14569).

## Current Problem

The current documentation at https://tauri.app/develop/plugins/develop-mobile/#android-1 shows:

```kotlin
System.loadLibrary("app_lib")
```

This hardcoded library name makes plugins non-portable, as developers must manually change `"app_lib"` to match their specific plugin's library name.

## Proposed Documentation Update

Replace the current section with the following:

---

### Android

In your plugin's `Cargo.toml`, add the jni crate as a dependency:

**Cargo.toml**
```toml
[target.'cfg(target_os = "android")'.dependencies]
jni = "0.21"
```

#### Understanding the Library Name

When Cargo builds your plugin for Android, it generates a native library (`.so` file) based on your `Cargo.toml` package name. The library name follows this pattern:

- **Package name in Cargo.toml**: `tauri-plugin-sample`
- **Generated library file**: `libtauri_plugin_sample.so`
- **Name for `System.loadLibrary()`**: `tauri_plugin_sample` (without `lib` prefix or `.so` extension)

Note: Hyphens (`-`) in the package name are converted to underscores (`_`) for the library name.

#### Loading the Library and Defining Native Functions

Load your plugin's native library and define external functions in your Kotlin code. In this example, the Kotlin class is `com.example.HelloWorld`, which requires referencing the full package name from the Rust side.

**Important**: Replace `"your_plugin_lib_name"` with your actual library name derived from your `Cargo.toml` package name.

**Example.kt**
```kotlin
package com.example

import android.util.Log

class HelloWorld {
    companion object {
        private const val TAG = "HelloWorld"
        
        // Define your plugin's library name based on Cargo.toml package.name
        // Example: if package name is "tauri-plugin-sample", use "tauri_plugin_sample"
        private const val LIBRARY_NAME = "your_plugin_lib_name"
        
        init {
            try {
                System.loadLibrary(LIBRARY_NAME)
                Log.d(TAG, "Successfully loaded lib$LIBRARY_NAME.so")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load lib$LIBRARY_NAME.so", e)
                throw e
            }
        }
    }
    
    external fun helloWorld(name: String): String?
}
```

**Real-world example**: For a plugin with `package.name = "tauri-plugin-sample"` in `Cargo.toml`:

```kotlin
package com.plugin.sample

import android.util.Log

class Example {
    companion object {
        private const val TAG = "Example"
        private const val LIBRARY_NAME = "tauri_plugin_sample" // Derived from "tauri-plugin-sample"
        
        init {
            try {
                System.loadLibrary(LIBRARY_NAME)
                Log.d(TAG, "Successfully loaded libtauri_plugin_sample.so")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load libtauri_plugin_sample.so", e)
                throw e
            }
        }
    }
    
    external fun helloWorld(name: String): String?
}
```

#### Implementing the Rust JNI Function

In your plugin's Rust code, define the function JNI will look for. The function name format is `Java_package_class_method`, so for our class above this becomes `Java_com_example_HelloWorld_helloWorld` to get called by our `helloWorld` method:

**lib.rs**
```rust
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_example_HelloWorld_helloWorld(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jstring {
    log::debug!("Calling JNI Hello World!");
    let result = format!("Hello, {}!", name);
    match env.new_string(result) {
        Ok(jstr) => jstr.into_raw(),
        Err(e) => {
            log::error!("Failed to create JString: {}", e);
            std::ptr::null_mut()
        }
    }
}
```

#### Quick Reference

To determine your library name:
1. Check your `Cargo.toml` `package.name` field
2. Replace hyphens with underscores
3. Use this as your `LIBRARY_NAME` constant in Kotlin

| Cargo.toml package.name | Library file | System.loadLibrary() argument |
|------------------------|--------------|------------------------------|
| `tauri-plugin-sample` | `libtauri_plugin_sample.so` | `"tauri_plugin_sample"` |
| `my-awesome-plugin` | `libmy_awesome_plugin.so` | `"my_awesome_plugin"` |
| `plugin_name` | `libplugin_name.so` | `"plugin_name"` |

---

## Files to Update in tauri-docs Repository

This content should be updated in the `tauri-apps/tauri-docs` repository, specifically in the file that generates the "Calling Rust From Mobile Plugins" section at https://tauri.app/develop/plugins/develop-mobile/#android-1.

The exact file path will be in the docs repository, likely something like:
- `src/content/docs/develop/plugins/develop-mobile.mdx` or similar

## Benefits of This Change

1. **Clear naming convention**: Developers understand how the library name is derived
2. **Portable code**: Using a constant makes it easy to update in one place
3. **Better error messages**: The example includes helpful logging
4. **Quick reference table**: Easy lookup for common patterns
5. **Real-world example**: Shows actual plugin code structure
