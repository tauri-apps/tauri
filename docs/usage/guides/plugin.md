---
title: Write Tauri Plugins
---

import Alert from '@theme/Alert'

<Alert title="Note" icon="info-alt">
This guide uses the `plugin::Builder` introduced in `v1`, if you're looking for the old version using trait implementations [see here](#advanced)
</Alert>

Plugins allow you to hook into the Tauri application lifecycle and introduce new commands.

## Getting started

Plugins are reusable extensions to the Tauri API that solve common problems. They are also a very convenient way to structure your own code base!

If you intend to share you plugin with others, we provide a ready-made template! With the tauri-cli installed just run:

```sh
tauri init plugin --name awesome
```

### API package

By default consumers of your plugin can call provided commands like this:

```ts
import { invoke } from '@tauri-apps/api'

invoke('plugin:awesome|do_something')
```
where `awesome` will be replaced by your plugin name.

This isn't very convenient however, so it's common for plugins to prive a so called *API package*, a JavaScript package that provides convenient access to your commands.

> An example of this is the [tauri-plugin-store](https://github.com/tauri-apps/tauri-plugin-store), that provides a convenient class structure to accessing a store.

You can scaffold a plugin with attached API package like this:

```sh
tauri init plugin --name awesome --api
```

## Writing a Plugin

Using the `tauri::plugin::Builder` you can define plugins similar to how you define your app:

```rust
use tauri::plugin::{Builder, GenericPlugin};

// the plugin custom command handlers if you choose to extend the API.
#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|initialize')`.
// where `awesome` is the plugin name.
fn initialize() {}

#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|do_something')`.
fn do_something() {}

pub fn init() -> GenericPlugin {
  Builder::new("awesome")
    .invoke_handler(tauri::generate_handler![initialize, do_something])
    .build()
}
```

Plugins can setup and maintain state, just like your app can:

```rust
  use tauri::{
    AppHandle, Runtime, State
    plugin::{Builder, GenericPlugin}
  };

  #[derive(Default)]
  struct MyState {

  }

  #[tauri::command]
  // this will be accessible with `invoke('plugin:awesome|do_something')`.
  fn do_something<R: Runtime>(_app: AppHandle<R>, state: State<'_, MyState>) {
    // you can access `MyState` here!
  }

  Builder::new("awesome")
    .invoke_handler(tauri::generate_handler![initialize, do_something])
    .setup(|app_handle| {
      // setup plugin specific state here

      app.manage(MyState::default())

      Ok(())
    })
    .build()
```

### Conventions

1. Your crate exports an `init` function that returns the plugin.
2. Plugin names start with `tauri-plugin-`

## Using a Plugin

To use a plugin, just pass the plugin instance to the App
's `plugin` method:

```rust
fn main() {
  tauri::Builder::default()
    .plugin(my_awesome_plugin::init())
    .run(tauri::generate_context!())
    .expect("failed to run app");
}
```

## Distributing a Plugin

Plugins can be published on [crates.io](https://crates.io) and [npm](https://npmjs.com) or downloaded from github directly. While official plugins provide both options, you are free to choose the distribution mechanism. Just make sure to document how your plugin can be installed!

## Advanced

Under the hood `GenericPlugin` implements the `Plugin` trait and while it's recommended to use the `Builder` to create plugins, you can implement this trait and structs yourself:

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

## Official Tauri Plugins

- [Stronghold](https://github.com/tauri-apps/tauri-plugin-stronghold)
- [Authenticator](https://github.com/tauri-apps/tauri-plugin-authenticator)
- [Logging](https://github.com/tauri-apps/tauri-plugin-log)
- [SQL](https://github.com/tauri-apps/tauri-plugin-sql)
- [WebSocket](https://github.com/tauri-apps/tauri-plugin-websocket)
- [Restoring window state](https://github.com/tauri-apps/tauri-plugin-window-state)
- [Store](https://github.com/tauri-apps/tauri-plugin-store)
