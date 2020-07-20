# Changelog

## [0.8.0]

-   Use native dialog on `window.alert` and `window.confirm`.
    Since every communication with the webview is asynchronous, the `window.confirm` returns a Promise resolving to a boolean value.
        - [0245833](https://www.github.com/tauri-apps/tauri/commit/0245833bb56d5462a4e1249eb1e2f9f5e477592d) feat(tauri) make `window.alert` and `window.confirm` available, fix [#848](https://www.github.com/tauri-apps/tauri/pull/848) ([#854](https://www.github.com/tauri-apps/tauri/pull/854)) on 2020-07-18
        - [dac0ae9](https://www.github.com/tauri-apps/tauri/commit/dac0ae976ea1b419ed5af078d00106b1476dee04) chore(changes) add tauri-api to JS dialogs changefile on 2020-07-19
-   The notification's `body` is now optional, closes #793.
    -   [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
-   Fixes a regression on the storage of requestPermission response.
    ß
        - [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
-   Plugin system added. You can hook into the webview lifecycle (`created`, `ready`) and extend the API adding logic to the `invoke_handler` by implementing the `tauri::plugin::Plugin` trait.
    -   [78afee9](https://www.github.com/tauri-apps/tauri/commit/78afee9725e0e372f9de7edeaac523011a1c02a3) feat(tauri) add plugin system for rust ([#494](https://www.github.com/tauri-apps/tauri/pull/494)) on 2020-07-12
-   Renaming `whitelist` to `allowlist` (see #645).
    -   [a6bb3b5](https://www.github.com/tauri-apps/tauri/commit/a6bb3b59059e08a844d7bb2b43f3d1192954d890) refactor(tauri) rename `whitelist` to `allowlist`, ref [#645](https://www.github.com/tauri-apps/tauri/pull/645) ([#858](https://www.github.com/tauri-apps/tauri/pull/858)) on 2020-07-19
-   Moving the webview implementation to [webview](https://github.com/webview/webview), with the [official Rust binding](https://github.com/webview/webview_rust).
    This is a breaking change.
    IE support has been dropped, so the `edge` object on `tauri.conf.json > tauri` no longer exists and you need to remove it.
    `webview.handle()` has been replaced with `webview.as_mut()`.
        - [cd5b401](https://www.github.com/tauri-apps/tauri/commit/cd5b401707d709bf8212b0a4c34623f902ae40f9) feature: import official webview rust binding ([#846](https://www.github.com/tauri-apps/tauri/pull/846)) on 2020-07-18

## [0.7.5]

-   Fixes Edge blank screen on Windows when running tauri dev (Tauri crashing window due to Edge reloading app because of missing Content-Type header).
    -   Bumped due to a bump in tauri-api.
    -   [fedee83](https://www.github.com/tauri-apps/tauri/commit/fedee835e36daa4363b91aabd43143e8033c9a5c) fix(tauri.js) windows Edge blank screen on tauri dev ([#808](https://www.github.com/tauri-apps/tauri/pull/808)) on 2020-07-11

## [0.7.4]

-   Ignoring non UTF-8 characters on the loopback command output.
    -   [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## [0.7.3]

-   Properly run the loopback command on Windows.
-   Properly ignore the ${distDir}/index.html asset from the asset embbeding. Previously every asset with name matching /(.+)index.html$/g were ignored.

## [0.7.2]

Bumped due to dependency.

## [0.7.1]

-   Fixes the assets embedding into the binary.

## [0.7.0]

-   The execute_promise and execute_promise_sync helpers now accepts any tauri::Result<T> where T: impl Serialize.
    This means that you do not need to serialize your response manually or deal with String quotes anymore.
    As part of this refactor, the event::emit function also supports impl Serialize instead of String.

## [0.6.2]

-   Fixes the Windows build with the latest Windows SDK.

## [0.6.1] - (Not Published)

## [0.6.0]

-   Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
-   Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
    Build no longer fails when compiling without environment variables, now the app will show an error.
-   Adds desktop notifications API.
-   Properly reflect tauri.conf.json changes on app when running tauri dev.
