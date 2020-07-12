# Changelog

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
