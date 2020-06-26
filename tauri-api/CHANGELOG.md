# Changelog

## [0.6.1]

-   Fixes the httpRequest headers usage. It now accepts Strings instead of serde_json::Value.

## [0.6.0]

-   This adds HttpRequestBuilder, described at "alternatives you've considered" section in undefined.
-   Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
-   Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
    Build no longer fails when compiling without environment variables, now the app will show an error.
-   Adds desktop notifications API.
