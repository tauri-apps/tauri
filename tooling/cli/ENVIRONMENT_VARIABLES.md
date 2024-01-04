### Tauri's Environment Variables

This is a documentation of all environment variables used by tauri core crates and tauri CLI.

### Tauri CLI

These environment variables are inputs to the CLI which may have an equivalent CLI flag.

> if both environment variable and CLI flag are used, the CLI flag will have priority.

- `CI` — If set, the CLI will run in CI mode and won't require any user interaction.
- `TAURI_CLI_CONFIG_DEPTH` — Number of levels to traverse and find tauri configuration file.
- `TAURI_CLI_PORT` — Port to use for the CLI built-in dev server.
- `TAURI_CLI_WATCHER_IGNORE_FILENAME` — Name of a `.gitignore`-style file to control which files should be watched by the CLI in `dev` command. The CLI will look for this file name in each directory.
- `TAURI_CLI_NO_DEV_SERVER_WAIT` — Skip waiting for the frontend dev server to start before building the tauri application.
- `TAURI_LINUX_AYATANA_APPINDICATOR` — Set this var to `true` or `1` to force usage of `libayatana-appindicator` for system tray on Linux.
- `TAURI_BUNDLER_WIX_FIPS_COMPLIANT` — Specify the bundler's WiX `FipsCompliant` option.
- `TAURI_SIGNING_PRIVATE_KEY` — Private key used to sign your app bundles, can be either a string or a path to the file.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — The signing private key password, see `TAURI_SIGNING_PRIVATE_KEY`.
- `APPLE_CERTIFICATE` — Base64 encoded of the `.p12` certificate for code signing. To get this value, run `openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt`.
- `APPLE_CERTIFICATE_PASSWORD` — The password you used to export the certificate.
- `APPLE_ID` — The Apple ID used to notarize the application. If this environment variable is provided, `APPLE_PASSWORD` and `APPLE_TEAM_ID` must also be set. Alternatively, `APPLE_API_KEY` and `APPLE_API_ISSUER` can be used to authenticate.
- `APPLE_PASSWORD` — The Apple password used to authenticate for application notarization. Required if `APPLE_ID` is specified. An app-specific password can be used. Alternatively to entering the password in plaintext, it may also be specified using a '@keychain:' or '@env:' prefix followed by a keychain password item name or environment variable name.
- `APPLE_TEAM_ID`: Developer team ID. To find your Team ID, go to the [Account](https://developer.apple.com/account) page on the Apple Developer website, and check your membership details.
- `APPLE_API_KEY` — Alternative to `APPLE_ID` and `APPLE_PASSWORD` for notarization authentication using JWT.
  - See [creating API keys](https://developer.apple.com/documentation/appstoreconnectapi/creating_api_keys_for_app_store_connect_api) for more information.
- `API_PRIVATE_KEYS_DIR` — Specify the directory where your AuthKey file is located. See `APPLE_API_KEY`.
- `APPLE_API_ISSUER` — Issuer ID. Required if `APPLE_API_KEY` is specified.
- `APPLE_API_KEY_PATH` - path to the API key `.p8` file. If not specified, the bundler searches the following directories in sequence for a private key file with the name of 'AuthKey_<api_key>.p8': './private_keys', '~/private_keys', '~/.private_keys', and '~/.appstoreconnect/private_keys'.
- `APPLE_SIGNING_IDENTITY` — The identity used to code sign. Overwrites `tauri.conf.json > tauri > bundle > macOS > signingIdentity`.
- `APPLE_PROVIDER_SHORT_NAME` — If your Apple ID is connected to multiple teams, you have to specify the provider short name of the team you want to use to notarize your app. Overwrites `tauri.conf.json > tauri > bundle > macOS > providerShortName`.
- `APPLE_DEVELOPMENT_TEAM` — TODO
- `TAURI_WEBVIEW_AUTOMATION` — Enables webview automation (Linux Only).
- `TAURI_ANDROID_PROJECT_PATH` — Path of the tauri android project, usually will be `<project>/src-tauri/gen/android`.
- `TAURI_IOS_PROJECT_PATH` — Path of the tauri iOS project, usually will be `<project>/src-tauri/gen/ios`.
- `RPM_SIGN_KEY` — The private GPG key used to sign the RPM bundle, exported to its ASCII-armored format.
- `RPM_SIGN_KEY_PASSPHRASE` — The GPG key passphrase for `RPM_SIGN_KEY`, if needed.

### Tauri CLI Hook Commands

These environment variables are set for each hook command (`beforeDevCommand`, `beforeBuildCommand`, ...etc) which could be useful to conditionally build your frontend or execute a specific action.

- `TAURI_ENV_DEBUG` — `true` for `dev` command or `build --debug`, `false` otherwise.
- `TAURI_ENV_TARGET_TRIPLE` — Target triple the CLI is building.
- `TAURI_ENV_ARCH` — Target arch, `x86_64`, `aarch64`...etc.
- `TAURI_ENV_PLATFORM` — Target platform, `windows`, `darwin`, `linux`...etc.
- `TAURI_ENV_PLATFORM_VERSION` — Build platform version
- `TAURI_ENV_FAMILY` — Target platform family `unix` or `windows`.
