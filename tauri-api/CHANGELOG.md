# Changelog

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
