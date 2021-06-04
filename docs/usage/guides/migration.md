---
title: Migrating from 0.x
---

First of all if you still have `tauri` as dependency in your `package.json`
replace it with a recent version of `@tauri-apps/cli` (make sure to also change
the import path in your JavaScript/TypeScript files, see [JavaScript](#javascript)).

For example:

```diff
- "tauri": "^0.14.1"
+ "@tauri-apps/cli": "^1.0.0-beta-rc.4"
```

Next update your `Cargo.toml`:

- add `tauri-build` as a new build-dependency and remove `winres`, e.g.:

  ```diff
  + [build-dependencies]
  + tauri-build = { version = "1.0.0-beta-rc.0" }

  - [target."cfg(windows)".build-dependencies]
  - winres = "0.1"
  ```

- update the version of `tauri` to e.g. `1.0.0-beta-rc.4`
- remove all old features of the `tauri` dependency
- remove all features, that tauri added and add `custom-protocol` as a new one:
  
  ```diff
  [features]
  - embedded-server = [ "tauri/embedded-server" ]
  - no-server = [ "tauri/no-server" ]
  + custom-protocol = [ "tauri/custom-protocol" ]
  + default = [ "custom-protocol" ]
  ```

Update your `tauri.conf.json` like this:

- remove `ctx`
- remove the `embeddedServer`
- rename `osx` to `macOS` and add some fields:
  - `"exceptionDomain": ""`
  - `"signingIdentity": null`
  - `"entitlements": null`
- remove the `exceptionDomain`
- add a configuration for `windows`:
  - `"certificateThumbprint": null`
  - `"digestAlgorithm": "sha256"`
  - `"timestampUrl": ""`
- make the `window` definition into an array and call it `windows`
- remove `inliner`

> for more information about the config see [here](../../api/config.md)

```diff
  {
-   "ctx": {},
    "tauri": {
-     "embeddedServer": {
-       "active": true
-     },
      "bundle": {
-       "osx": {
+       "macOS": {
          "frameworks": [],
          "minimumSystemVersion": "",
-         "useBootstrapper": false
+         "useBootstrapper": false,
+         "exceptionDomain": "",
+         "signingIdentity": null,
+         "entitlements": null
        },
-       "exceptionDomain": ""
+       "windows": {
+         "certificateThumbprint": null,
+         "digestAlgorithm": "sha256",
+         "timestampUrl": ""
+       }
      },
+     "updater": {
+       "active": false
+     },
-     "window": {
+     "windows": [
        {
          "title": "Calciumdibromid",
          "width": 800,
          "height": 600,
          "resizable": true,
          "fullscreen": false
        }
+     ],
-     "inliner": {
-       "active": true
-     }
    }
  }
```

## Commands

The following example is taken from the previous documentation.

In the new version of Tauri there is no distinction between synchronous and
asynchronous commands, the only difference in your code is a call of
`tauri::execute_promise()`, that isn't there in a synchronous command.

### Rust

Here is the complete example code of the "old" version:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct DoSomethingPayload {
  state: String,
  data: u64,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
enum Cmd {
  DoSomething {
    count: u64,
    payload: DoSomethingPayload,
    callback: String,
    error: String,
  },
}

#[derive(Serialize)]
struct Response<'a> {
  value: u64,
  message: &'a str,
}

#[derive(Debug, Clone)]
struct CommandError<'a> {
  message: &'a str,
}

impl<'a> CommandError<'a> {
  fn new(message: &'a str) -> Self {
    Self { message }
  }
}

impl<'a> std::fmt::Display for CommandError<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl<'a> std::error::Error for CommandError<'a> {}

fn main() {
  tauri::AppBuilder::new()
    .invoke_handler(|_webview, arg| {
      use Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            DoSomething { count, payload, callback, error } => tauri::execute_promise(
              _webview,
              move || {
                if count > 5 {
                  let response = Response {
                    value: 5,
                    message: "async response!",
                  };
                  Ok(response)
                } else {
                  Err(CommandError::new("count should be > 5").into())
                }
              },
              callback,
              error,
            ),
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
```

Complete the following steps to migrate your code:

- create a new function for every `Cmd` enum variant
- wrap the new function with the `#[tauri::command]` macro
- use the fields of the enum as arguments (`callback` and `error` can be deleted)
- as function body use the code inside the `match` block of the enum variant
- add a return type
- rename `AppBuilder` to `Builder` in `main()`
- replace the big `invoke_handler` with the new syntax

The old example code should look like this now:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct DoSomethingPayload {
  state: String,
  data: u64,
}

#[derive(Serialize)]
struct Response<'a> {
  value: u64,
  message: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct CommandError<'a> {
  message: &'a str,
}

impl<'a> CommandError<'a> {
  fn new(message: &'a str) -> Self {
    Self { message }
  }
}

impl<'a> std::fmt::Display for CommandError<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl<'a> std::error::Error for CommandError<'a> {}

#[tauri::command]
fn do_something(count: u64, payload: DoSomethingPayload) -> Result<Response, CommandError> {
  if count > 5 {
    let response = Response {
      value: 5,
      message: "async response!",
    };
    Ok(response)
  } else {
    Err(CommandError::new("count should be > 5").into())
  }
}

fn main() {
  tauri::Builder::new()
    .invoke_handler(tauri::generate_handler![do_something])
    .run(tauri::generate_context!());
}
```

### JavaScript

Like mentioned above there is also no distinction between synchronous and
asynchronous commands in JavaScript.  
You only have to use `invoke` and optionally use the results.

Here is an example of the "old" code:

```js
invoke({
  cmd: 'doSomething',
  count: 5,
  payload: {
    state: 'some string data',
    data: 17
  }
});

promisified({
  cmd: 'doSomething',
  count: 5,
  payload: {
    state: 'some string data',
    data: 17
  }
}).then(response => {
  console.log(response);
}).catch(error => {
  console.error(error);
});
```

Complete the following steps to migrate your code:

- replace all `promisified`-calls with `invoke`-calls
- extract the `cmd` attribute of the argument object as first parameter  
  (you may have to rename it to `snake_case` as the `cmd` parameter is now the
  name of the function in Rust)
- if you import parts of the tauri-api with `tauri/api/*` replace it with `@tauri-apps/api/*`, e.g.:

  ```diff
  - import { invoke } from 'tauri/api/tauri';
  + import { invoke } from '@tauri-apps/api/tauri';
  ```

The old example code should look like this now:

```js
invoke(
  'do_something',
  {
    count: 5,
    payload: {
      state: 'some string data',
      data: 17
    }
  }
);

invoke(
  'do_something',
  {
    count: 5,
    payload: {
      state: 'some string data',
      data: 17
    }
  }
).then(response => {
  console.log(response);
}).catch(error => {
  console.error(error);
});
```

For more information on commands read [Create Rust Commands](command.md).
