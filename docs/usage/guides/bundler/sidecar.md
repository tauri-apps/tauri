---
title: Sidecar (Embedding External Binaries)
sidebar_label: Sidecar
---

import Alert from '@theme/Alert'

You may need to embed depending binaries in order to make your application work or to prevent users having to install additional dependencies (e.g. Node.js, Python, etc).

To bundle the binaries of your choice, you can add the `externalBin` property to the `tauri` namespace in your `tauri.conf.json`.

See more about tauri.conf.json configuration <a href="/docs/api/config#build">here</a>.

`externalBin` expects a list of strings targeting binaries either with absolute or relative paths.

Here is a sample to illustrate the configuration, this is not a complete `tauri.conf.json` file:

```json
{
  "tauri": {
    "bundle": {
      "externalBin": ["/absolute/path/to/bin1", "relative/path/to/bin2"]
    }
  }
}
```

This way, you may [execute commands with Rust](https://doc.rust-lang.org/std/process/struct.Command.html) in your Tauri application.

<Alert title="Note">
Tauri provides some functions to handle standard cases (like loading platform specific binaries), such as:

- `tauri::api::command::binary_command`, which will append the current environment triplet to the input (useful for cross-environments). If you're creating your own binary, you'll _have to_ provide a binary **for each platform you're targeting** by specifying the target triplets, e.g. "binaryname-x86_64-apple-darwin".

Target triplets can be found by executing the `rustup target list` command.

- `tauri::api::command::relative_command` that will relatively resolve the path to the binary.

</Alert>
