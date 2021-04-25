# Changelog

## \[1.0.0-beta-rc.3]

- Fixes distribution of the `@tauri-apps/api` package for older bundlers.
  - [7f998d0](https://www.github.com/tauri-apps/tauri/commit/7f998d08e3ab8823c99190fa283bdfa2c4f2749b) fix(api): distribution ([#1582](https://www.github.com/tauri-apps/tauri/pull/1582)) on 2021-04-22
- Update minimum Node.js version to v12.13.0
  - [1f089fb](https://www.github.com/tauri-apps/tauri/commit/1f089fb4f964c673dcab5784bdf1da2833487a7c) chore: update minimum nodejs version to 12.13.0 ([#1562](https://www.github.com/tauri-apps/tauri/pull/1562)) on 2021-04-21

## \[1.0.0-beta-rc.2]

- TS was wrongly re-exporting the module.
  - [fcb3b48](https://www.github.com/tauri-apps/tauri/commit/fcb3b4857efa17d2a3717f32457e88b24520cc9b) fix: [#1512](https://www.github.com/tauri-apps/tauri/pull/1512) ([#1517](https://www.github.com/tauri-apps/tauri/pull/1517)) on 2021-04-19
  - [ae14a3f](https://www.github.com/tauri-apps/tauri/commit/ae14a3ff51a742b6ab6f76bbfc21f385310f1dc6) fix: [#1517](https://www.github.com/tauri-apps/tauri/pull/1517) had the wrong package reference in the changefile ([#1538](https://www.github.com/tauri-apps/tauri/pull/1538)) on 2021-04-19

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- Add current working directory to the path api module.
  - [52c2baf](https://www.github.com/tauri-apps/tauri/commit/52c2baf940773cf7c51647fb6f20d0f7df126115) feat: add current working directory to path api module ([#1375](https://www.github.com/tauri-apps/tauri/pull/1375)) on 2021-03-23
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The shell process spawning API was rewritten and now includes stream access.
  - [3713066](https://www.github.com/tauri-apps/tauri/commit/3713066e451bd30d0cc6f57bb437f08276f4c4ad) refactor(core): rewrite shell execute API, closes [#1229](https://www.github.com/tauri-apps/tauri/pull/1229) ([#1408](https://www.github.com/tauri-apps/tauri/pull/1408)) on 2021-03-31
- The file dialog API now uses [rfd](https://github.com/PolyMeilex/rfd). The filter option is now an array of `{ name: string, extensions: string[] }`.
  - [2326bcd](https://www.github.com/tauri-apps/tauri/commit/2326bcd399411f7f0eabdb7ade910be473adadae) refactor(core): use `nfd` for file dialogs, closes [#1251](https://www.github.com/tauri-apps/tauri/pull/1251) ([#1257](https://www.github.com/tauri-apps/tauri/pull/1257)) on 2021-02-18
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The HTTP API was improved with client caching and better payload and response types.
  - [a7bc472](https://www.github.com/tauri-apps/tauri/commit/a7bc472e994730071f960d09a12ac85296a080ae) refactor(core): improve HTTP API, closes [#1098](https://www.github.com/tauri-apps/tauri/pull/1098) ([#1237](https://www.github.com/tauri-apps/tauri/pull/1237)) on 2021-02-15
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Use secure RNG on callback function names.
  - [c8992bb](https://www.github.com/tauri-apps/tauri/commit/c8992bb0bfb8eaeae8ebed444719f9c9372d39d4) refactor(api): use secure RNG, closes [#1356](https://www.github.com/tauri-apps/tauri/pull/1356) ([#1398](https://www.github.com/tauri-apps/tauri/pull/1398)) on 2021-03-30
- The invoke function can now be called with the cmd as the first parameter and the args as the second.
  - [427d170](https://www.github.com/tauri-apps/tauri/commit/427d170930ab711fd0ca82f7a73b524d6fdc222f) feat(api/invoke): separate cmd arg ([#1321](https://www.github.com/tauri-apps/tauri/pull/1321)) on 2021-03-04
- Adds a global shortcut API.
  - [855effa](https://www.github.com/tauri-apps/tauri/commit/855effadd9ebfb6bc1a3555ac7fc733f6f766b7a) feat(core): globalShortcut API ([#1232](https://www.github.com/tauri-apps/tauri/pull/1232)) on 2021-02-14
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added window management and window creation APIs.
  - [a3d6dff](https://www.github.com/tauri-apps/tauri/commit/a3d6dff2163c7a45842253edd81dbc62248dc65d) feat(core): window API ([#1225](https://www.github.com/tauri-apps/tauri/pull/1225)) on 2021-02-13
  - [641374b](https://www.github.com/tauri-apps/tauri/commit/641374b15343518cd835bd5ada811941c65dcf2e) feat(core): window creation at runtime ([#1249](https://www.github.com/tauri-apps/tauri/pull/1249)) on 2021-02-17
