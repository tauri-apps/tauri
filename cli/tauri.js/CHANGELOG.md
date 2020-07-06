# Changelog

## [0.8.1]

-   Transpile the TS API to ES5.
    Expose CJS as .js and ESM as .mjs.
-   Fixes the assets embedding into the binary.

## [0.8.0]

-   Create UMD, ESM and CJS artifacts for the JavaScript API entry point from TS source using rollup.
-   Renaming window.tauri to window.\_\_TAURI\_\_, closing #435.
    The **Tauri** object now follows the TypeScript API structure (e.g. window.tauri.readTextFile is now window.\_\_TAURI\_\_.fs.readTextFile).
    If you want to keep the `window.tauri` object for a while, you can add a [mapping object](https://gist.github.com/lucasfernog/8f7b29cadd91d92ee2cf816a20c2ef01) to your code.
