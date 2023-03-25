---
"cli.rs": patch
"cli.js": patch
---

Add `--ci` flag and respect the `CI` environment variable on the `signer generate` command. In this case the default password will be an empty string and the CLI will not prompt for a value.
