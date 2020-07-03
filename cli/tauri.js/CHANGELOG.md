# Changelog

## [0.8.0]

-   Create UMD, ESM and CJS artifacts for the JavaScript API entry point from TS source using rollup.
-   Renaming window.tauri to window.**TAURI**, closing #435.
    The **TAURI** object now follows the TypeScript API structure (e.g. window.**TAURI**.readTextFile is now window.**TAURI**.fs.readTextFile).
