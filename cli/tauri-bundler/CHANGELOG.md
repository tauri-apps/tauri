# Changelog

## [0.9.3]

-   Improve checking for Xcode command line tools to allow builds on mac
    -   [7a788fd](https://www.github.com/tauri-apps/tauri/commit/7a788fdceebc2bf6b7b46ebe54e98597d4a71529) fix: improve checking for Rez (fix [#994](https://www.github.com/tauri-apps/tauri/pull/994)) ([#995](https://www.github.com/tauri-apps/tauri/pull/995)) on 2020-08-28
-   add a newline after Categories in deb .desktop file generation to fix issues #899 and #925.
    -   [37bcf5f](https://www.github.com/tauri-apps/tauri/commit/37bcf5fea154bdefbca2692b69aafaabba8c23e2) fix(bundler) missing newline in deb desktop file generation (fix: [#899](https://www.github.com/tauri-apps/tauri/pull/899), [#925](https://www.github.com/tauri-apps/tauri/pull/925)) ([#998](https://www.github.com/tauri-apps/tauri/pull/998)) on 2020-08-27

## [0.9.2]

-   Bump all deps as noted in #975, #976, #977, #978, and #979.
    -   [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20

## [0.9.1]

-   Hide external scripts output unless `--verbose` is passed.
    -   [78add1e](https://www.github.com/tauri-apps/tauri/commit/78add1e79ef42ed61e988a0012be87b428439332) feat(bundler): hide output from shell scripts unless --verbose is passed (fixes [#888](https://www.github.com/tauri-apps/tauri/pull/888)) ([#893](https://www.github.com/tauri-apps/tauri/pull/893)) on 2020-07-26
-   Fixes the target directory detection, respecting the `CARGO_TARGET_DIR` and `.cargo/config (build.target-dir)` options to set the Cargo output directory.
    -   [63b9c64](https://www.github.com/tauri-apps/tauri/commit/63b9c6457233d777b698b53cd6661c6cd9a0d67b) fix(bundler) properly detect the target directory ([#895](https://www.github.com/tauri-apps/tauri/pull/895)) on 2020-07-25
-   Bundling every DLL file on the binary directory.
    -   [a00ac02](https://www.github.com/tauri-apps/tauri/commit/a00ac023eef9f7b3a508ca9acd664470382d7d06) fix(bundler) webview dll not being bundled, fixes [#875](https://www.github.com/tauri-apps/tauri/pull/875) ([#889](https://www.github.com/tauri-apps/tauri/pull/889)) on 2020-07-24

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
