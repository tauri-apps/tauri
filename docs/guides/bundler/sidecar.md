---
title: Sidecar (Embedding External Binaries)
sidebar_label: Sidecar
---

import Alert from '@theme/Alert'

You may need to embed depending binaries in order to make your application work or to prevent users having to install additional dependencies (e.g. Node.js, Python, etc).

To bundle the binaries of your choice, you can add the `externalBin` property to the `tauri > bundle` object in your `tauri.conf.json`.

See more about tauri.conf.json configuration <a href="/docs/api/config#tauri.bundle">here</a>.

`externalBin` expects a list of strings targeting binaries either with absolute or relative paths.

Here is a sample to illustrate the configuration, this is not a complete `tauri.conf.json` file:

```json
{
  "tauri": {
    "bundle": {
      "externalBin": ["/absolute/path/to/app", "relative/path/to/binary", "bin/python"]
    }
  }
}
```

A binary with the same name and a `-$TARGET_TRIPLE` suffix must exist on the specified path. For instance, `"externalBin": ["bin/python"]` requires a `src-tauri/bin/python-x86_64-unknown-linux-gnu` executable on Linux. You can find the current platform's target triple running the following command:

```bash
rustc -Vv | grep host | cut -f2 -d' '
```

Here's a Node.js script to append the target triple to a binary:

```javascript
const execa = require('execa')
const fs = require('fs')

let extension = ''
if (process.platform === 'win32') {
  extension = '.exe'
}

async function main() {
  const rustInfo = (await execa('rustc', ['-vV'])).stdout
  const targetTriple = /host: (\S+)/g.exec(rustInfo)[1]
  if (!targetTriple) {
    console.error('Failed to determine platform target triple')
  }
  fs.renameSync(
    `src-tauri/binaries/app${extension}`,
    `src-tauri/binaries/app-${targetTriple}${extension}`
  )
}

main().catch((e) => {
  throw e
})

```

## Running the sidecar binary on JavaScript

On the JavaScript code, import the `Command` class on the `shell` module and use the `sidecar` static method:

```javascript
import { Command } from '@tauri-apps/api/shell'
// alternatively, use `window.__TAURI__.shell.Command`
// `my-sidecar` is the value specified on `tauri.conf.json > tauri > bundle > externalBin`
const command = Command.sidecar('my-sidecar')
const output = await command.execute()
```

## Running the sidecar binary on Rust

On the Rust code, import the `Command` struct from the `tauri::api::process` module:

```rust
let (mut rx, mut child) = Command::new_sidecar("my-sidecar")
  .expect("failed to create `my-sidecar` binary command")
  .spawn()
  .expect("Failed to spawn sidecar");

tauri::async_runtime::spawn(async move {
  // read events such as stdout
  while let Some(event) = rx.recv().await {
    if let CommandEvent::Stdout(line) = event {
      window
        .emit("message", Some(format!("'{}'", line)))
        .expect("failed to emit event");
      // write to stdin
      child.write("message from Rust\n".as_bytes()).unwrap();
    }
  }
});
```

## Using Node.js on a sidecar

The Tauri [sidecar example](https://github.com/tauri-apps/tauri/tree/dev/examples/sidecar) demonstrates how to use the sidecar API to run a Node.js application on Tauri.
It compiles the Node.js code using [pkg](https://github.com/vercel/pkg) and uses the scripts above to run it.
