---
title: Create Rust Commands
---

Tauri provides a simple yet powerful "command" system for calling Rust functions from your web app. Commands can accept arguments and return values. They can also return errors and be `async`.

## Basic Example

Commands are defined in your `src-tauri/src/main.rs` file. To create a command, just add a function and annotate it with `#[tauri::command]`:

```rust
#[tauri::command]
fn my_custom_command() {
  println!("I was invoked from JS!");
}
```

You will have to provide a list of your commands to the builder function like so:

```rust
// Also in main.rs
fn main() {
  tauri::Builder::default()
    // This is where you pass in your commands
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .run(tauri::generate_context!())
    .expect("failed to run app");
}
```

Now, you can invoke the command from your JS code:

```js
// With the Tauri API npm package:
import { invoke } from '@tauri-apps/api/tauri'
// With the Tauri global script:
const invoke = window.__TAURI__.invoke

// Invoke the command
invoke('my_custom_command')
```

## Passing Arguments

Your command handlers can take arguments:

```rust
#[tauri::command]
fn my_custom_command(invoke_message: String) {
  println!("I was invoked from JS, with this message: {}", invoke_message);
}
```

Arguments should be passed as a JSON object with camelCase keys:

```js
invoke('my_custom_command', { invokeMessage: 'Hello!' })
```

Arguments can be of any type, as long as they implement [serde::Deserialize](https://serde.rs/derive.html).

## Returning Data

Command handlers can return data as well:

```rust
#[tauri::command]
fn my_custom_command() -> String {
  "Hello from Rust!".into()
}
```

The `invoke` function returns a promise that resolves with the returned value:

```js
invoke('my_custom_command').then((message) => console.log(message))
```

Returned data can be of any type, as long as it implements [Serde::Serialize](https://serde.rs/derive.html).

## Error Handling

If your handler could fail and needs to be able to return an error, have the function return a `Result`:

```rust
#[tauri::command]
fn my_custom_command() -> Result<String, String> {
  // If something fails
  Err("This failed!".into())
  // If it worked
  Ok("This worked!".into())
}
```

If the command returns an error, the promise will reject, otherwise it resolves:

```js
invoke('my_custom_command')
  .then((message) => console.log(message))
  .catch((error) => console.error(error))
```

## Async Commands

If your command needs to run asynchronously, simply declare it as `async`:

```rust
#[tauri::command]
async fn my_custom_command() {
  // Call another async function and wait for it to finish
  let result = some_async_function().await;
  println!("Result: {}", result);
}
```

Since invoking the command from JS already returns a promise, it works just like any other command:

```js
invoke('my_custom_command').then(() => console.log('Completed!'))
```

## Accessing the Window in Commands

If your command needs access to the Window (TODO: add link), add `with_window` to the `command` annotation:

```rust
#[tauri::command(with_window)]
async fn my_custom_command<M: tauri::Params>(window: tauri::Window<M>) {
  println!("Window: {}", window.label());
}
```

## Complete Example

Any or all of the above features can be combined:

```rust title=main.rs
// Definition in main.rs

#[derive(serde::Serialize)]
struct CustomResponse {
  message: String,
  other_val: usize,
}

#[tauri::command(with_window)]
async fn my_custom_command<M: tauri::Params>(
  window: tauri::Window<M>,
  number: usize,
) -> Result<CustomResponse, String> {
  println!("Called from {}", window.label());
  let result: Option<String> = some_other_function().await;
  if let Some(message) = result {
    Ok(CustomResponse {
      message,
      other_val: 42 + number,
    })
  } else {
    Err("No result".into())
  }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

```js
// Invocation from JS

invoke('my_custom_command', {
  message: 'Hi',
  number: 42,
})
  .then((res) =>
    console.log(`Message: ${res.message}, Other Val: ${res.other_val}`)
  )
  .catch((e) => console.error(e))
```
