---
title: Debian packages
---

import Alert from '@theme/Alert'

Tauri allows your app to be packaged as a `.deb` (Debian package) file.

# Bootstrapper

Instead of launching the app directly, you can configure the bundled app to run a script that tries to expose the environment variables to the app; without that you'll have trouble using system programs because the `PATH` environment variable isn't correct. Enable it with the <a href="/docs/api/config#tauri.bundle.deb.useBootstrapper">`useBootstrapper`</a> config.

# Custom files

To include custom files to the debian package, you can configure a mapping on `tauri.conf.json > tauri > bundle > deb > files` as follows:

```json
{
  "tauri": {
    "bundle": {
      "deb": {
        "files": {
          "/usr/lib/README.md": "../README.md", // copies the README.md file to /usr/lib/README.md
          "usr/lib/assets": "../public/" // copies the entire public directory to /usr/lib/assets
        }
      }
    }
  }
}
```

<Alert title="Note" icon="info-alt">
Each `files` object key is the path on the debian package, and the value is a path to a file or directory relative to the `tauri.conf.json` file.
</Alert>
