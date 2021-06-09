---
title: Updating
---
import Alert from '@theme/Alert'

<Alert title="Please note" type="warning" icon="alert">
    Especially during the alpha and beta phases, you are expected to keep all Tauri dependencies and toolchains up to date. There is no support for any versions other than latest.
</Alert>

## Automatic updates

The Tauri JS CLI has a command to install and update all needed dependencies, just run `tauri deps install` or `tauri deps update`.

## Manual updates

### Update NPM Packages

If you are using the `tauri` package:
```bash
$ yarn upgrade @tauri-apps/cli @tauri-apps/api --latest
$ npm install @tauri-apps/cli@latest @tauri-apps/api@latest
```
You can also detect what the latest version of Tauri is on the command line, using:
- `npm outdated @tauri-apps/cli`
- `yarn outdated @tauri-apps/cli`

Alternatively, if you are using the `vue-cli-plugin-tauri` approach:
```bash
$ yarn upgrade vue-cli-plugin-tauri --latest
$ npm install vue-cli-plugin-tauri@latest
```

### Update Cargo Packages
Go to `src-tauri/Cargo.toml` and change `tauri` to
`tauri = { version = "%version%" }` where `%version%` is the version number shown above. (You can just use the `MAJOR.MINOR`) version, like `0.9`.

Then do the following:
```bash
$ cd src-tauri
$ cargo update -p tauri
```
You can also run `cargo outdated -r tauri` to get direct information about the core library's latest version.
