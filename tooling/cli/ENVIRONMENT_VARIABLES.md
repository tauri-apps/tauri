<!-- TODO: v2 rename all vars with consistency and grouping -->

### Tauri's Environment Variables

This is a documentation of all environment variables used by tauri core crates and tauri CLI.

### Tauri CLI

These environment variables are inputs to the CLI which may have an equivalent CLI flag.

> if both environment variable and CLI flag are used, the CLI flag will have priority.

- `TAURI_PATH_DEPTH` — Number of levels to traverse and find tauri configuration file.
- `TAURI_DEV_SERVER_PORT` — Port to use for the CLI built-in dev server.
- `TAURI_DEV_WATCHER_IGNORE_FILE` — A `.gitignore`-style file to control which files should be watched by the CLI in `dev` command.
- `TAURI_SKIP_DEVSERVER_CHECK` — Skip waiting for the frontend dev server to start before building the tauri application.
- `TAURI_SKIP_UPDATE_CHECK` — Skip checking for a newer CLI version before exiting the CLI process.
- `TAURI_TRAY` — Set this var to `ayatana` to use `libayatana-appindicator` for system tray on Linux or set it `appindicator` to use `libappindicator`.
  - For `deb` bundle target, the CLI will add the appropriate package as a dependency, if unset, the CLI will default to `ayatana`.
  - For `appimage` bundle target, the CLI will copy the appropriate package to the appimage. if unset, the CLI will make a guess based on what package is installed on the developer system.
- `TAURI_FIPS_COMPLIANT` — Specify WiX `FipsCompliant` option
- `TAURI_PRIVATE_KEY` — Private key used to sign your app bundles
- `TAURI_KEY_PASSWORD` — The private key password, see `TAURI_PRIVATE_KEY`
- `APPLE_CERTIFICATE` —
- `APPLE_CERTIFICATE_PASSWORD` —
- `APPLE_ID` —
- `APPLE_PASSWORD` —
- `APPLE_API_KEY` —
- `APPLE_API_ISSUER` —
- `APPLE_SIGNING_IDENTITY` —
- `APPLE_PROVIDER_SHORT_NAME` —
- `CI` — If set, the CLI will run in CI mode and won't require any user interaction.

### Tauri CLI Hook Commands

These environment variables are set for each hook command (`beforeDevCommand`, `beforeBuildCommand`, ...etc) which could be useful to conditionally build your frontend or execute a specific action.

- `TAURI_ARCH` — Target arch, `x86_64`, `aarch64`...etc.
- `TAURI_PLATFORM` — Target platform, `windows`, `macos`, `linux`...etc.
- `TAURI_FAMILY` — Target platform family `unix` or `windows`.
- `TAURI_PLATFORM_TYPE` — Target platform type `Linux`, `Windows_NT` or `Darwin`
- `TAURI_PLATFORM_VERSION` — Build platform version
- `TAURI_DEBUG` — `true` for `dev` command, `false` for `build` command.
- `TAURI_TARGET_TRIPLE` — Target triple the CLI is building.
