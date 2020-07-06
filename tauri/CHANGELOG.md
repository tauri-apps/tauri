# Changelog

## [0.7.1]

-   Fixes the assets embedding into the binary.

## [0.7.0]

-   The execute_promise and execute_promise_sync helpers now accepts any tauri::Result<T> where T: impl Serialize.
    This means that you do not need to serialize your response manually or deal with String quotes anymore.
    As part of this refactor, the event::emit function also supports impl Serialize instead of String.

## [0.7.1]

-   Fixes the Windows build with the latest Windows SDK.

## [0.7.0]

-   Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
-   Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
    Build no longer fails when compiling without environment variables, now the app will show an error.
-   Adds desktop notifications API.
-   Properly reflect tauri.conf.json changes on app when running tauri dev.
