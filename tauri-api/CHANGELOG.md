# Changelog

## [0.7.2]

-   Fixes Edge blank screen on Windows when running tauri dev (Tauri crashing window due to Edge reloading app because of missing Content-Type header).
    -   Bumped due to a bump in tauri-api.
    -   [fedee83](https://www.github.com/tauri-apps/tauri/commit/fedee835e36daa4363b91aabd43143e8033c9a5c) fix(tauri.js) windows Edge blank screen on tauri dev ([#808](https://www.github.com/tauri-apps/tauri/pull/808)) on 2020-07-11

## [0.7.1]

-   Fixes the config reloading when tauri.conf.json changes.

## [0.7.0]

-   The execute_promise and execute_promise_sync helpers now accepts any tauri::Result<T> where T: impl Serialize.
    This means that you do not need to serialize your response manually or deal with String quotes anymore.
    As part of this refactor, the event::emit function also supports impl Serialize instead of String.
-   readDir API refactor. Now returns path, name and children. 

## [0.6.1]

-   Fixes the httpRequest headers usage. It now accepts Strings instead of serde_json::Value.

## [0.6.0]

-   This adds HttpRequestBuilder, described at "alternatives you've considered" section in undefined.
-   Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
-   Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
    Build no longer fails when compiling without environment variables, now the app will show an error.
-   Adds desktop notifications API.
