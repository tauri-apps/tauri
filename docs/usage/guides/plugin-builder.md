---
title: Write Tauri Plugins
---

import Alert from '@theme/Alert'

<Alert title="Note" icon="info-alt">
This guide uses the `plugin::Builder` introduced in `v1`, if you're looking for the old version using trait implementations [see here](./plugin.md)
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
use tauri::plugin::{Builder, TauriPlugin};

// the plugin custom command handlers if you choose to extend the API.
#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|initialize')`.
// where `awesome` is the plugin name.
fn initialize() {}

#[tauri::command]
// this will be accessible with `invoke('plugin:awesome|do_something')`.
fn do_something() {}

pub fn init() -> TauriPlugin {
  Builder::new("awesome")
    .invoke_handler(tauri::generate_handler![initialize, do_something])
    .build()
}
```

Plugins can setup and maintain state, just like your app can:

```rust
  use tauri::{
    AppHandle, Runtime, State
    plugin::{Builder, TauriPlugin}
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
2. Plugin names start with `tauri-plugin-`.

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

## Official Tauri Plugins

- [Stronghold](https://github.com/tauri-apps/tauri-plugin-stronghold)
- [Authenticator](https://github.com/tauri-apps/tauri-plugin-authenticator)
- [Logging](https://github.com/tauri-apps/tauri-plugin-log)
- [SQL](https://github.com/tauri-apps/tauri-plugin-sql)
- [WebSocket](https://github.com/tauri-apps/tauri-plugin-websocket)
- [Restoring window state](https://github.com/tauri-apps/tauri-plugin-window-state)
- [Store](https://github.com/tauri-apps/tauri-plugin-store)
