---
title: Write Tauri Plugins
---

import Alert from '@theme/Alert'

<Alert title="Note" icon="info-alt">
The Tauri CLI can bootstrap a Plugin project with the `$ tauri init plugin --name your-plugin-name` command.
It setups the recommended folder structure, optionally adding a TypeScript API wrapper with the `--api` flag.
</Alert>

Plugins allow you to hook into the Tauri application lifecycle and introduce new commands.

## Writing a Plugin

To write a plugin you just need to implement the `tauri::plugin::Plugin` trait:

```rust
use tauri::{plugin::{Plugin, Result as PluginResult}, Runtime, PageLoadPayload, Window, Invoke, AppHandle};

struct MyAwesomePlugin<R: Runtime> {
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
  // plugin state, configuration fields
}

// the plugin custom command handlers if you choose to extend the API.
#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|initialize')`.
// where `awesome` is the plugin name.
fn initialize() {}

#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|do_something')`.
fn do_something() {}

impl<R: Runtime> MyAwesomePlugin<R> {
  // you can add configuration fields here,
  // see https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
  pub fn new() -> Self {
    Self {
      invoke_handler: Box::new(tauri::generate_handler![initialize, do_something]),
    }
  }
}

impl<R: Runtime> Plugin<R> for MyAwesomePlugin<R> {
  /// The plugin name. Must be defined and used on the `invoke` calls.
  fn name(&self) -> &'static str {
    "awesome"
  }

  /// The JS script to evaluate on initialization.
  /// Useful when your plugin is accessible through `window`
  /// or needs to perform a JS task on app initialization
  /// e.g. "window.awesomePlugin = { ... the plugin interface }"
  fn initialization_script(&self) -> Option<String> {
    None
  }

  /// initialize plugin with the config provided on `tauri.conf.json > plugins > $yourPluginName` or the default value.
  fn initialize(&mut self, app: &AppHandle<R>, config: serde_json::Value) -> PluginResult<()> {
    Ok(())
  }

  /// Callback invoked when the Window is created.
  fn created(&mut self, window: Window<R>) {}

  /// Callback invoked when the webview performs a navigation.
  fn on_page_load(&mut self, window: Window<R>, payload: PageLoadPayload) {}

  /// Extend the invoke handler.
  fn extend_api(&mut self, message: Invoke<R>) {
    (self.invoke_handler)(message)
  }
}
```

Note that each function on the `Plugin` trait is optional, except the `name` function.

## Using a plugin

To use a plugin, just pass an instance of the `MyAwesomePlugin` struct to the App's `plugin` method:

```rust
fn main() {
  let awesome_plugin = MyAwesomePlugin::new();
  tauri::Builder::default()
    .plugin(awesome_plugin)
    .run(tauri::generate_context!())
    .expect("failed to run app");
}
```

## Official Tauri Plugins

- [Stronghold](https://github.com/tauri-apps/tauri-plugin-stronghold)
- [Authenticator](https://github.com/tauri-apps/tauri-plugin-authenticator)
- [Logging](https://github.com/tauri-apps/tauri-plugin-log)
- [SQL](https://github.com/tauri-apps/tauri-plugin-sql)
- [WebSocket](https://github.com/tauri-apps/tauri-plugin-websocket)
- [Restoring window state](https://github.com/tauri-apps/tauri-plugin-window-state)
- [Store](https://github.com/tauri-apps/tauri-plugin-store)
