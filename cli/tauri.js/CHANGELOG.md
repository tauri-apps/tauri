# Changelog

## [0.9.0]

-   Fixes a race condition on the beforeDevCommand usage (starting Tauri before the devServer is ready).
    -   [a26cffc](https://www.github.com/tauri-apps/tauri/commit/a26cffc575bee224a6beb5b7b0565d5583c0131f) fix(tauri.js) beforeDevCommand race condition ([#801](https://www.github.com/tauri-apps/tauri/pull/801)) on 2020-07-10
-   Revert a nullish coalescing operator that changed embedded server/inliner behavior.
    -   [e7b4951](https://www.github.com/tauri-apps/tauri/commit/e7b495133fe9f4e9f576bb9805bec98b967783eb) fix(tauri.js) revert nullish coalesce addition ([#79](https://www.github.com/tauri-apps/tauri/pull/79)9) on 2020-07-10
-   Fixes tauri init not generating tauri.conf.json on the Vue CLI Plugin.
    -   [f208a68](https://www.github.com/tauri-apps/tauri/commit/f208a68e40c804daf41d54539d3a5951679e8a64) fix(tauri.js) do not swallow init errors, fix conf inject ([#80](https://www.github.com/tauri-apps/tauri/pull/80)2) on 2020-07-10
-   tauri init now prompt for default values such as window title, app name, dist dir and dev path. You can use --ci to skip the prompts.
    -   [ee8724b](https://www.github.com/tauri-apps/tauri/commit/ee8724b90a63f281292c6eb174773b905ba52e32) feat(tauri.js/init): prompt for default values (fix [#42](https://www.github.com/tauri-apps/tauri/pull/42)2/[#16](https://www.github.com/tauri-apps/tauri/pull/16)2) ([#47](https://www.github.com/tauri-apps/tauri/pull/47)2) on 2020-07-10

## [0.8.4]

-   Bump lodash to 4.17.19

## [0.8.3]

-   Fixes the wrong cli value on the template that's used by tauri init.
    Also fixes the template test.
-   Fixes the tauri icon usage with the --icon flag. Previously, only the -i flag worked.

## [0.8.2]

-   Adds tauri.conf.json schema validation to the CLI.

## [0.8.1]

-   Transpile the TS API to ES5.
    Expose CJS as .js and ESM as .mjs.
-   Fixes the assets embedding into the binary.

## [0.8.0]

-   Create UMD, ESM and CJS artifacts for the JavaScript API entry point from TS source using rollup.
-   Renaming window.tauri to window.\_\_TAURI\_\_, closing #435.
    The **Tauri** object now follows the TypeScript API structure (e.g. window.tauri.readTextFile is now window.\_\_TAURI\_\_.fs.readTextFile).
    If you want to keep the `window.tauri` object for a while, you can add a [mapping object](https://gist.github.com/lucasfernog/8f7b29cadd91d92ee2cf816a20c2ef01) to your code.
