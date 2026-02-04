# Tauri Asset Gstreamer Plugin

This crate provides a GStreamer plugin that allows you to handle `asset://` URIs in Tauri applications.

## Compilation

```sh
cargo build --release
```

To be able to use it you need to set the `GST_PLUGIN_PATH` environment variable to point to the directory containing the compiled plugin and then you can run your application. For example:

```sh
export GST_PLUGIN_PATH=$(pwd)/target/release
./your-tauri-app
```

Or put the plugin in one of the standard GStreamer plugin directories (e.g. `/usr/lib64/gstreamer-1.0` on Linux).

## Testing

```sh
cargo test
```

Or with verbose output:

```sh
GST_DEBUG=4 cargo test
```

or only information from the plugin:

```sh
GST_DEBUG=tauri_asset:7 cargo test
```
