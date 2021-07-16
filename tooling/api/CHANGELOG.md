# Changelog

## \[1.0.0-beta.5]

- Adds `convertFileSrc` helper to the `tauri` module, simplifying the process of using file paths as webview source (`img`, `video`, etc).
  - [51a5cfe4](https://www.github.com/tauri-apps/tauri/commit/51a5cfe4b5e9890fb6f639c9c929657fd747a595) feat(api): add `convertFileSrc` helper ([#2138](https://www.github.com/tauri-apps/tauri/pull/2138)) on 2021-07-02
- You can now use `emit`, `listen` and `once` using the `appWindow` exported by the window module.
  - [5d7626f8](https://www.github.com/tauri-apps/tauri/commit/5d7626f89781a6ebccceb9ab3b2e8335aa7a0392) feat(api): WindowManager extends WebviewWindowHandle, add events docs ([#2146](https://www.github.com/tauri-apps/tauri/pull/2146)) on 2021-07-03
- Allow manipulating a spawned window directly using `WebviewWindow`, which now extends `WindowManager`.
  - [d69b1cf6](https://www.github.com/tauri-apps/tauri/commit/d69b1cf6d7c13297073073d753e30fe1a22a09cb) feat(api): allow mananing windows created on JS ([#2154](https://www.github.com/tauri-apps/tauri/pull/2154)) on 2021-07-05

## \[1.0.0-beta.4]

- Add asset custom protocol to access local file system.
  - [ee60e424](https://www.github.com/tauri-apps/tauri/commit/ee60e424221559d3d725716b0003c5566ef2b5cd) feat: asset custom protocol to access local file system ([#2104](https://www.github.com/tauri-apps/tauri/pull/2104)) on 2021-06-28

## \[1.0.0-beta.3]

- Export `Response` and `ResponseType` as value instead of type.
  - [394b6e05](https://www.github.com/tauri-apps/tauri/commit/394b6e0572e7a0a92e103e462a7f603f7d569319) fix(api): http  `ResponseType` export type error ([#2065](https://www.github.com/tauri-apps/tauri/pull/2065)) on 2021-06-24

## \[1.0.0-beta.2]

- Export `BaseDirectory` in `path` module
  - [277f5ca5](https://www.github.com/tauri-apps/tauri/commit/277f5ca5a8ae227bbdccee1ad52bdd88b4a5b11b) feat(api): export `BaseDirectory` in `path` module ([#1885](https://www.github.com/tauri-apps/tauri/pull/1885)) on 2021-05-30
- Use `export type` to export TS types, enums and interfaces.
  - [9a662d26](https://www.github.com/tauri-apps/tauri/commit/9a662d2601b01d712c6bd205f8db1b674f56dfa7) fix: Monitor if --isolatedModules is enabled ([#1825](https://www.github.com/tauri-apps/tauri/pull/1825)) on 2021-05-13
  - [612cd8ec](https://www.github.com/tauri-apps/tauri/commit/612cd8ecb8e02954f3696b9e138cbc7d2c228fad) feat(api): finalize `export type` usage ([#1847](https://www.github.com/tauri-apps/tauri/pull/1847)) on 2021-05-17
- Adds `focus?: boolean` to the WindowOptions interface.
  - [5f351622](https://www.github.com/tauri-apps/tauri/commit/5f351622c7812ad1bb56ddb37364ccaa4124c24b) feat(core): add focus API to the WindowBuilder and WindowOptions, [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
- Adds `isDecorated` getter on the window API.
  - [f58a2114](https://www.github.com/tauri-apps/tauri/commit/f58a2114fbfd5307c349f05c88f2e08fd8baa8aa) feat(core): add `is_decorated` Window getter on 2021-05-30
- Adds `isResizable` getter on the window API.
  - [1e8af280](https://www.github.com/tauri-apps/tauri/commit/1e8af280c27f381828d6209722b10e889082fa00) feat(core): add `is_resizable` Window getter on 2021-05-30
- Adds `isVisible` getter on the window API.
  - [36506c96](https://www.github.com/tauri-apps/tauri/commit/36506c967de82bc7ff453d11e6104ecf66d7a588) feat(core): add `is_visible` API on 2021-05-30
- Adds `requestUserAttention` API to the `window` module.
  - [7dcca6e9](https://www.github.com/tauri-apps/tauri/commit/7dcca6e9281182b11ad3d4a79871f09b30b9b419) feat(core): add `request_user_attention` API, closes [#2023](https://www.github.com/tauri-apps/tauri/pull/2023) ([#2026](https://www.github.com/tauri-apps/tauri/pull/2026)) on 2021-06-20
- Adds `setFocus` to the window API.
  - [bb6992f8](https://www.github.com/tauri-apps/tauri/commit/bb6992f888196ca7c87bb2fe74ad2bd8bf393e05) feat(core): add `set_focus` window API, fixes [#1737](https://www.github.com/tauri-apps/tauri/pull/1737) on 2021-05-30
- Adds `setSkipTaskbar` to the window API.
  - [e06aa277](https://www.github.com/tauri-apps/tauri/commit/e06aa277384450cfef617c0e57b0d5d403bb1e7f) feat(core): add `set_skip_taskbar` API on 2021-05-30
- Adds `skipTaskbar?: boolean` to the WindowOptions interface.
  - [5525b03a](https://www.github.com/tauri-apps/tauri/commit/5525b03a78a2232c650043fbd9894ce1553cad41) feat(core): add `skip_taskbar` API to the WindowBuilder/WindowOptions on 2021-05-30
- Adds `center?: boolean` to `WindowOptions` and `center()` API to the `appWindow`.
  - [5cba6eb4](https://www.github.com/tauri-apps/tauri/commit/5cba6eb4d28d53f06855d60d4d0eae6b95233ccf) feat(core): add window `center` API, closes [#1822](https://www.github.com/tauri-apps/tauri/pull/1822) ([#1954](https://www.github.com/tauri-apps/tauri/pull/1954)) on 2021-06-05
- Adds `clipboard` APIs (write and read text).
  - [285bf64b](https://www.github.com/tauri-apps/tauri/commit/285bf64bf9569efb2df904c69c6df405ff0d62e2) feat(core): add clipboard writeText and readText APIs ([#2035](https://www.github.com/tauri-apps/tauri/pull/2035)) on 2021-06-21
  - [dee71ad5](https://www.github.com/tauri-apps/tauri/commit/dee71ad58349f699995cc9077b79032bacc6afcb) fix(workflows): update docs workflow syntax ([#2054](https://www.github.com/tauri-apps/tauri/pull/2054)) on 2021-06-23
- The `http` APIs now resolve the returned promise when the API call finishes with an error status code.
  - [47f75584](https://www.github.com/tauri-apps/tauri/commit/47f7558417cc654bdb1d018127e8900bc4eac622) fix(core): resolve HTTP API on non-ok status code, fix binary response, closes [#2046](https://www.github.com/tauri-apps/tauri/pull/2046) ([#2053](https://www.github.com/tauri-apps/tauri/pull/2053)) on 2021-06-23
- Improve RPC security by requiring a numeric code to invoke commands. The codes are generated by the Rust side and injected into the app's code using a closure, so external scripts can't access the backend. This change doesn't protect `withGlobalTauri` (`window.__TAURI__`) usage.
  - [160fb052](https://www.github.com/tauri-apps/tauri/commit/160fb0529fd31d755574ae30fbdf01fa221a2acb) feat(core): improve RPC security, closes [#814](https://www.github.com/tauri-apps/tauri/pull/814) ([#2047](https://www.github.com/tauri-apps/tauri/pull/2047)) on 2021-06-22
- Mark the `WebviewWindow` constructor as public.
  - [4aeb936e](https://www.github.com/tauri-apps/tauri/commit/4aeb936e9b60b895d383597dc698ee5d638436f9) fix(api): `WebviewWindow` constructor is public ([#1888](https://www.github.com/tauri-apps/tauri/pull/1888)) on 2021-05-21
- Validate arguments on the window `setLocation`, `setSize`, `setMinSize` and `setMaxSize` API.
  - [7616e6cc](https://www.github.com/tauri-apps/tauri/commit/7616e6cc7bcd49f688b0d00fdc33c94b7b93713d) feat(api): validate window API `size` and `location` arguments ([#1846](https://www.github.com/tauri-apps/tauri/pull/1846)) on 2021-05-17

## \[1.0.0-beta.1]

- Adds `package.json` to the `exports` object.
  - [ab1ea96](https://www.github.com/tauri-apps/tauri/commit/ab1ea964786e1781c922582b059c555b6072f1a0) chore(api): add `package.json` to the `exports` field ([#1807](https://www.github.com/tauri-apps/tauri/pull/1807)) on 2021-05-12

## \[1.0.0-beta.0]

- CommonJS chunks are now properly exported with `.cjs` extension
  - [ddcd923](https://www.github.com/tauri-apps/tauri/commit/ddcd9233bd6f499aa7f22484d6c151b01778bc1b) fix(api): export commonjs chunks with `.cjs` extension, fix [#1625](https://www.github.com/tauri-apps/tauri/pull/1625) ([#1627](https://www.github.com/tauri-apps/tauri/pull/1627)) on 2021-04-26
- Adds `transparent?: boolean` to the `WindowOptions` interface.
  - [08c1c5c](https://www.github.com/tauri-apps/tauri/commit/08c1c5ca5c0ebe17ea98689a5fe3b7e47a98e955) fix(api): missing `transparent` flag on `WindowOptions` ([#1764](https://www.github.com/tauri-apps/tauri/pull/1764)) on 2021-05-10
- Adds `options` argument to the shell command API (`env` and `cwd` configuration).
  - [721e98f](https://www.github.com/tauri-apps/tauri/commit/721e98f175567b360c86f30565ab1b9d08e7cf85) feat(core): add env, cwd to the command API, closes [#1634](https://www.github.com/tauri-apps/tauri/pull/1634) ([#1635](https://www.github.com/tauri-apps/tauri/pull/1635)) on 2021-04-28
- Adds `startDragging` API on the window module.
  - [c31f097](https://www.github.com/tauri-apps/tauri/commit/c31f0978c535f794fffb75a121e69a323e70b06e) refactor: update to wry 0.9 ([#1630](https://www.github.com/tauri-apps/tauri/pull/1630)) on 2021-04-28
- Move `exit` and `relaunch` APIs from `app` to `process` module.
  - [b0bb796](https://www.github.com/tauri-apps/tauri/commit/b0bb796a42e2560233aea47ce6ced54ac238eb53) refactor: rename `command` mod to `process`, move restart_application ([#1667](https://www.github.com/tauri-apps/tauri/pull/1667)) on 2021-04-30
- The window management API was refactored: removed `setX`, `setY`, `setWidth`, `setHeight` APIs, renamed `resize` to `setSize` and the size and position APIs now allow defining both logical and physical values.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05
- Adds window getters.
  - [6bfac86](https://www.github.com/tauri-apps/tauri/commit/6bfac866a703f1499a64237fb29b2625703f4e22) refactor(core): add window getters, physical & logical sizes/positions ([#1723](https://www.github.com/tauri-apps/tauri/pull/1723)) on 2021-05-05

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
