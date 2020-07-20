# Changelog

## [0.9.0]

-   Fixes the AppImage bundling on containers.
    -   [53e8dc1](https://www.github.com/tauri-apps/tauri/commit/53e8dc1880b78994e17bf769d60e13f9e13dbf99) fix(bundler) support AppImage bundling on containers [#822](https://www.github.com/tauri-apps/tauri/pull/822) on 2020-07-13
    -   [bd0118f](https://www.github.com/tauri-apps/tauri/commit/bd0118f160360e588180de3f3518ef47a4d86a46) fix(changes) covector status pass on 2020-07-14
-   Bundler output refactor: move Windows artifacts to the `bundle/wix` folder and use a standard output name `${bundleName}_${version}_${arch}.${extension}`.
    -   [9130f1b](https://www.github.com/tauri-apps/tauri/commit/9130f1b1a422121fa9f3afbeeb87e851568e995f) refactor(bundler) standard output names and path ([#823](https://www.github.com/tauri-apps/tauri/pull/823)) on 2020-07-13

## [0.8.5]

-   Ignoring non UTF-8 characters on the loopback command output.
    -   [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## [0.8.4]

-   Properly run the loopback command on Windows.

## [0.8.3]

-   Fixes the unbound variable issue on the DMG bundler script.

## [0.8.2]

-   Fixes the AppImage bundler script.

## [0.8.1]

-   Improves the logging of child processes like bundle_appimage.sh and bundle_dmg.sh.

## [0.8.0]

-   The bundler now bundles all binaries from your project (undefined) and undefined.
    When multiple binaries are used, make sure to use the undefined config field.
-   Check if mksquashfs is installed before bundling AppImage

## [0.7.0]

-   Fixes AppImage bundler (appimagetool usage, build script running properly, proper AppRun and .desktop files).
