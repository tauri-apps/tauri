# Changelog

## \[0.10.0]

- Append app version and OS architecture on AppImage output filename.
  - [ae76c60](https://www.github.com/tauri-apps/tauri/commit/ae76c60a615602fcb8c1dd824a6ad9fa8f48fe69) fix(bundler): appimage paths and filename ([#1227](https://www.github.com/tauri-apps/tauri/pull/1227)) on 2021-02-13
- The Tauri bundler is now a general purpose library instead of a Cargo custom subcommand.
  - [b1e6b74](https://www.github.com/tauri-apps/tauri/commit/b1e6b74a4f624b623a840686fb1abe1d23593867) refactor(cli): decouple bundler from cargo ([#1269](https://www.github.com/tauri-apps/tauri/pull/1269)) on 2021-02-21
- Rename macOS bundle settings from `osx` to `macOS`.
  - [080f639](https://www.github.com/tauri-apps/tauri/commit/080f6391bac4fd59e9e71b9785d7a2f835703805) refactor(bundler): specific settings on dedicated structs, update README ([#1380](https://www.github.com/tauri-apps/tauri/pull/1380)) on 2021-03-25
- The `dev` and `build` pipeline is now written in Rust.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Bundle Visual C++ redistributable files with VC142\_CRT merge modules.
  - [3047a18](https://www.github.com/tauri-apps/tauri/commit/3047a18975db07abdf49985f531c3925f68a0db9) feat(bundler): add visual c++ redistributable files with MSM ([#1368](https://www.github.com/tauri-apps/tauri/pull/1368)) on 2021-03-22
- Automatically install Webview2 runtime alongside app.
  - [8e9752b](https://www.github.com/tauri-apps/tauri/commit/8e9752bb8bad5c56b55a3750876e0073efdc6d39) feat(bundler/wix): install webview2 runtime ([#1329](https://www.github.com/tauri-apps/tauri/pull/1329)) on 2021-03-07
- Fixes the bundler workspace detection.
  - [e34ee4c](https://www.github.com/tauri-apps/tauri/commit/e34ee4c29c7fde02e09685a3100f0b2ef6380c98) fix(bundler): workspace detection, closes [#1007](https://www.github.com/tauri-apps/tauri/pull/1007) ([#1235](https://www.github.com/tauri-apps/tauri/pull/1235)) on 2021-02-14

## \[0.9.4]

- `dirs` crate is unmaintained, now using `dirs-next` instead.
  - [82cda98](https://www.github.com/tauri-apps/tauri/commit/82cda98532975c6d4b1c93bf2f326173f39e0964) chore(tauri) `dirs` crate is unmaintained, use `dirst-next` instead ([#1057](https://www.github.com/tauri-apps/tauri/pull/1057)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Force IPv4 on `wget` so AppImage bundling doesn't hang.
  - [6f5667b](https://www.github.com/tauri-apps/tauri/commit/6f5667bf72d58972b8d05ee2e42a031c85f95ed4) fix: [#1018](https://www.github.com/tauri-apps/tauri/pull/1018) Force IPv4 on wget requests ([#1019](https://www.github.com/tauri-apps/tauri/pull/1019)) on 2020-10-11
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Set the Windows installer (WiX) `WorkingDirectory` field to `INSTALLDIR` so the app can read paths relatively (previously resolving to `C:\Windows\System32`).
  - [5cf3402](https://www.github.com/tauri-apps/tauri/commit/5cf3402735ac2e88fc4aae5fe39fc0a41262b6fa) fix: add working directory to wix's shortcut ([#1021](https://www.github.com/tauri-apps/tauri/pull/1021)) on 2020-09-24
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21

## \[0.9.3]

- Improve checking for Xcode command line tools to allow builds on mac
  - [7a788fd](https://www.github.com/tauri-apps/tauri/commit/7a788fdceebc2bf6b7b46ebe54e98597d4a71529) fix: improve checking for Rez (fix [#994](https://www.github.com/tauri-apps/tauri/pull/994)) ([#995](https://www.github.com/tauri-apps/tauri/pull/995)) on 2020-08-28
- add a newline after Categories in deb .desktop file generation to fix issues #899 and #925.
  - [37bcf5f](https://www.github.com/tauri-apps/tauri/commit/37bcf5fea154bdefbca2692b69aafaabba8c23e2) fix(bundler) missing newline in deb desktop file generation (fix: [#899](https://www.github.com/tauri-apps/tauri/pull/899), [#925](https://www.github.com/tauri-apps/tauri/pull/925)) ([#998](https://www.github.com/tauri-apps/tauri/pull/998)) on 2020-08-27

## \[0.9.2]

- Bump all deps as noted in #975, #976, #977, #978, and #979.
  - [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20

## \[0.9.1]

- Hide external scripts output unless `--verbose` is passed.
  - [78add1e](https://www.github.com/tauri-apps/tauri/commit/78add1e79ef42ed61e988a0012be87b428439332) feat(bundler): hide output from shell scripts unless --verbose is passed (fixes [#888](https://www.github.com/tauri-apps/tauri/pull/888)) ([#893](https://www.github.com/tauri-apps/tauri/pull/893)) on 2020-07-26
- Fixes the target directory detection, respecting the `CARGO_TARGET_DIR` and `.cargo/config (build.target-dir)` options to set the Cargo output directory.
  - [63b9c64](https://www.github.com/tauri-apps/tauri/commit/63b9c6457233d777b698b53cd6661c6cd9a0d67b) fix(bundler) properly detect the target directory ([#895](https://www.github.com/tauri-apps/tauri/pull/895)) on 2020-07-25
- Bundling every DLL file on the binary directory.
  - [a00ac02](https://www.github.com/tauri-apps/tauri/commit/a00ac023eef9f7b3a508ca9acd664470382d7d06) fix(bundler) webview dll not being bundled, fixes [#875](https://www.github.com/tauri-apps/tauri/pull/875) ([#889](https://www.github.com/tauri-apps/tauri/pull/889)) on 2020-07-24

## \[0.9.0]

- Fixes the AppImage bundling on containers.
  - [53e8dc1](https://www.github.com/tauri-apps/tauri/commit/53e8dc1880b78994e17bf769d60e13f9e13dbf99) fix(bundler) support AppImage bundling on containers [#822](https://www.github.com/tauri-apps/tauri/pull/822) on 2020-07-13
  - [bd0118f](https://www.github.com/tauri-apps/tauri/commit/bd0118f160360e588180de3f3518ef47a4d86a46) fix(changes) covector status pass on 2020-07-14
- Bundler output refactor: move Windows artifacts to the `bundle/wix` folder and use a standard output name `${bundleName}_${version}_${arch}.${extension}`.
  - [9130f1b](https://www.github.com/tauri-apps/tauri/commit/9130f1b1a422121fa9f3afbeeb87e851568e995f) refactor(bundler) standard output names and path ([#823](https://www.github.com/tauri-apps/tauri/pull/823)) on 2020-07-13

## \[0.8.5]

- Ignoring non UTF-8 characters on the loopback command output.
  - [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## \[0.8.4]

- Properly run the loopback command on Windows.

## \[0.8.3]

- Fixes the unbound variable issue on the DMG bundler script.

## \[0.8.2]

- Fixes the AppImage bundler script.

## \[0.8.1]

- Improves the logging of child processes like bundle_appimage.sh and bundle_dmg.sh.

## \[0.8.0]

- The bundler now bundles all binaries from your project (undefined) and undefined.
  When multiple binaries are used, make sure to use the undefined config field.
- Check if mksquashfs is installed before bundling AppImage

## \[0.7.0]

- Fixes AppImage bundler (appimagetool usage, build script running properly, proper AppRun and .desktop files).
