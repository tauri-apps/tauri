# Changelog

## \[1.0.0-beta-rc.3]

- `tauri::error::CreateWebview` now has the error string message attached.
  - [7471e34](https://www.github.com/tauri-apps/tauri/commit/7471e347d3b23b7604c19040b0d989da8f48cb91) feat(core): add error message on `error::CreateWebview` ([#1602](https://www.github.com/tauri-apps/tauri/pull/1602)) on 2021-04-23
- If the dialog `defaultPath` is a file, use it as starting file path.
  - [aa7e273](https://www.github.com/tauri-apps/tauri/commit/aa7e2738ccafd8e4f5df866206f12888f6db8973) feat: use `rfd::FileDialog#set_file_name` if `default_path` is a file ([#1598](https://www.github.com/tauri-apps/tauri/pull/1598)) on 2021-04-23
- Validate dialog option `defaultPath` - it must exists.
  - [cfa74eb](https://www.github.com/tauri-apps/tauri/commit/cfa74ebf68de96cf46bc9471c61f9a84dd0be9ee) feat(core): validate dialog `default_path` (it must exist) ([#1599](https://www.github.com/tauri-apps/tauri/pull/1599)) on 2021-04-23
- Expose `async_runtime` module.
  - [d3fdeb4](https://www.github.com/tauri-apps/tauri/commit/d3fdeb45184d9aed8405ded53607a7cca979275e) feat(core): expose `async_runtime` module ([#1576](https://www.github.com/tauri-apps/tauri/pull/1576)) on 2021-04-21
- Expose `PageLoadPayload` struct.
  - [5e65b76](https://www.github.com/tauri-apps/tauri/commit/5e65b768e5930708695512260faf8c12d679c04e) fix(core): expose `PageLoadPayload` struct ([#1590](https://www.github.com/tauri-apps/tauri/pull/1590)) on 2021-04-22
- Fixes the Message `command` name value on plugin invoke handler.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
- Allow `window.__TAURI__.invoke` to be moved to another variable.
  - [be65f04](https://www.github.com/tauri-apps/tauri/commit/be65f04db7d2fce23477156ebba368a897ceee3c) fix(core): make window.**TAURI**.invoke context free, fixes [#1547](https://www.github.com/tauri-apps/tauri/pull/1547) ([#1565](https://www.github.com/tauri-apps/tauri/pull/1565)) on 2021-04-21
- Make sure custom protocol is treated as secure content on macOS.
  - [5909c1e](https://www.github.com/tauri-apps/tauri/commit/5909c1e01437e10c45694c24f9037f4b176a03ec) Make sure custom protocol is handled as secure context on macOS ([#1551](https://www.github.com/tauri-apps/tauri/pull/1551)) on 2021-04-22
- Fixes macOS shortcut modifiers keycodes.
  - [ceadf2f](https://www.github.com/tauri-apps/tauri/commit/ceadf2f556f5f327b34f8fdd01e5e07969182b13) fix(core): macos shortcut modifiers, closes [#1542](https://www.github.com/tauri-apps/tauri/pull/1542) ([#1560](https://www.github.com/tauri-apps/tauri/pull/1560)) on 2021-04-21
- Adds APIs to determine global and webview-specific URI scheme handlers.
  - [938fb62](https://www.github.com/tauri-apps/tauri/commit/938fb624f5cc0f2a4499ea67cd30b014a18a6526) feat(core): expose custom protocol handler APIs ([#1553](https://www.github.com/tauri-apps/tauri/pull/1553)) on 2021-04-21
  - [a868cb7](https://www.github.com/tauri-apps/tauri/commit/a868cb71762268aa7b78af26622c900bddf3344c) refactor(core): clear `uri_scheme_protocol` registration function names ([#1617](https://www.github.com/tauri-apps/tauri/pull/1617)) on 2021-04-25
- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
- Change plugin trait `initialization` return type to `std::result::Result<(), Box<dyn std::error::Error>>`.
  - [508eddc](https://www.github.com/tauri-apps/tauri/commit/508eddc78458cd7ff51259ed733fe8e6f206e293) refactor(core): plugin initialization return value ([#1575](https://www.github.com/tauri-apps/tauri/pull/1575)) on 2021-04-21
- Fixes `sidecar` Command API.
  - [99307d0](https://www.github.com/tauri-apps/tauri/commit/99307d02c3c28ce10ba418873ac02ce267af4f4f) fix(core): sidecar command path ([#1584](https://www.github.com/tauri-apps/tauri/pull/1584)) on 2021-04-22
- Set LocalStorage and IndexedDB files path on Linux to `$HOME/.local/${bundleIdentifier}`.
  - [5f033db](https://www.github.com/tauri-apps/tauri/commit/5f033db41cf6b043d9d2b4debe8b10bdc4633c58) feat(core): use bundle identifier on user data path ([#1580](https://www.github.com/tauri-apps/tauri/pull/1580)) on 2021-04-22
- Use bundle identifier instead of `Tauri` for user data path on Windows.
  - [5f033db](https://www.github.com/tauri-apps/tauri/commit/5f033db41cf6b043d9d2b4debe8b10bdc4633c58) feat(core): use bundle identifier on user data path ([#1580](https://www.github.com/tauri-apps/tauri/pull/1580)) on 2021-04-22

## \[1.0.0-beta-rc.2]

- Prevent "once" events from being able to be called multiple times.

- `Window::trigger(/*...*/)` is now properly `pub` instead of `pub(crate)`.

- `Manager::once_global(/*...*/)` now returns an `EventHandler`.

- `Window::once(/*...*/)` now returns an `EventHandler`.

- (internal) `event::Listeners::trigger(/*...*/)` now handles removing "once" events.

- [ece243d](https://www.github.com/tauri-apps/tauri/commit/ece243d17c9c3ef8d2ba8e3b25b872aa6ea0b6ab) don't remove once listener on new thread ([#1506](https://www.github.com/tauri-apps/tauri/pull/1506)) on 2021-04-14

- Window and global events can now be nested inside event handlers. They will run as soon
  as the event handler closure is finished in the order they were called. Previously, calling
  events inside an event handler would produce a deadlock.

Note: The order that event handlers are called when triggered is still non-deterministic.

- [e447b8e](https://www.github.com/tauri-apps/tauri/commit/e447b8e0e6198c8972bae25625bb409850cb686b) allow event listeners to be nested ([#1513](https://www.github.com/tauri-apps/tauri/pull/1513)) on 2021-04-15

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - Bumped due to a bump in api.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- internal refactoring of `Params` to allow for easier usage without a private trait with only 1 implementor.
  `ParamsPrivate` -> `ParamsBase`
  `ManagerPrivate` -> `ManagerBase`
  (new) `Args`, crate only. Now implements `Params`/`ParamsBase`.
  `App` and `Window` use `WindowManager` directly
- [ec27ca8](https://www.github.com/tauri-apps/tauri/commit/ec27ca81fe721e0b08ed574073547250b7a8153a) refactor(tauri): remove private params trait methods ([#1484](https://www.github.com/tauri-apps/tauri/pull/1484)) on 2021-04-14
- Updated `wry`, fixing an issue with the draggable region.
  - [f2d24ef](https://www.github.com/tauri-apps/tauri/commit/f2d24ef2fbd95ec7d3433ba651964f4aa3b7f48c) chore(deps): update wry ([#1482](https://www.github.com/tauri-apps/tauri/pull/1482)) on 2021-04-14
- Now Tauri commands always returns Promise<T>.
  - [ea73325](https://www.github.com/tauri-apps/tauri/commit/ea7332539d100bd63f93396101ffa01ff73c924b) refactor(core): all API are now promise based ([#1239](https://www.github.com/tauri-apps/tauri/pull/1239)) on 2021-02-16
- Rename macOS bundle settings from `osx` to `macOS`.
  - [080f639](https://www.github.com/tauri-apps/tauri/commit/080f6391bac4fd59e9e71b9785d7a2f835703805) refactor(bundler): specific settings on dedicated structs, update README ([#1380](https://www.github.com/tauri-apps/tauri/pull/1380)) on 2021-03-25
- The shell process spawning API was rewritten and now includes stream access.
  - [3713066](https://www.github.com/tauri-apps/tauri/commit/3713066e451bd30d0cc6f57bb437f08276f4c4ad) refactor(core): rewrite shell execute API, closes [#1229](https://www.github.com/tauri-apps/tauri/pull/1229) ([#1408](https://www.github.com/tauri-apps/tauri/pull/1408)) on 2021-03-31
- The Tauri files are now read on the app space instead of the `tauri` create.
  Also, the `AppBuilder` `build` function now returns a Result.
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Tauri now uses explicit Error variants with `thiserror` instead of relying on `anyhow`.
  - [156a0ad](https://www.github.com/tauri-apps/tauri/commit/156a0ad5cb0a152eaa0dd038a6b3dba68f03eb21) refactor(tauri): use explicit error types instead of anyhow ([#1209](https://www.github.com/tauri-apps/tauri/pull/1209)) on 2021-02-10
- Align HTTP API types with the [documentation](https://tauri.studio/en/docs/api/js#http).
  - [2fc39fc](https://www.github.com/tauri-apps/tauri/commit/2fc39fc341771431078c20a95fa6b2affe5155c9) fix(api/http): correct types ([#1360](https://www.github.com/tauri-apps/tauri/pull/1360)) on 2021-03-17
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Replace `\` with `\\` in css assets that are lazy loaded. Since these are injected in a template literal, backslashes must be escaped. Backslashes are sometimes used for octal sequences in CSS.
  - [4491c70](https://www.github.com/tauri-apps/tauri/commit/4491c707907a6a931fd8c057c2baeb0b9e6db1d8) fix(tauri/asset): escape octal sequences in css ([#1166](https://www.github.com/tauri-apps/tauri/pull/1166)) on 2021-01-30
- Replaces the embedded-server mode with Wry's custom protocol feature. This allows assets to be transferred to the webview directly, instead of through a localhost server.
  - [0c691f4](https://www.github.com/tauri-apps/tauri/commit/0c691f40a338be184a4dd2c84d6e5d0b0ed6ee4b) feat(core): Use Wry custom protocol instead of embedded server ([#1296](https://www.github.com/tauri-apps/tauri/pull/1296)) on 2021-02-25
- The `message` and `ask` dialogs now use `tinyfiledialogs-rs` instead of `tauri-dialog-rs`.
  - [6eee355](https://www.github.com/tauri-apps/tauri/commit/6eee355a12ead3ac9cb4be0c98c1cfe5c0611291) refactor(core): use tinyfiledialogs-rs for message/confirmation dialogs ([#1255](https://www.github.com/tauri-apps/tauri/pull/1255)) on 2021-02-17
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Refactor the event callback payload and return an unlisten function on the `listen` API.
  - [b670ec5](https://www.github.com/tauri-apps/tauri/commit/b670ec55f2b7389b8a2f8c965d4fe1e0cb46e6dc) refactor(core): add `unlisten`, `once` APIs to the event system ([#1359](https://www.github.com/tauri-apps/tauri/pull/1359)) on 2021-03-16
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Adds `unlisten` and `once` APIs on the Rust event system.
  - [b670ec5](https://www.github.com/tauri-apps/tauri/commit/b670ec55f2b7389b8a2f8c965d4fe1e0cb46e6dc) refactor(core): add `unlisten`, `once` APIs to the event system ([#1359](https://www.github.com/tauri-apps/tauri/pull/1359)) on 2021-03-16
- The `tauri::event` module has been moved to a Webview manager API.
  - [07208df](https://www.github.com/tauri-apps/tauri/commit/07208dff6c1e8cff7c10780f4f7f8cee9de44a2e) feat(core): add mult-window support ([#1217](https://www.github.com/tauri-apps/tauri/pull/1217)) on 2021-02-11
- The file dialog API now uses [rfd](https://github.com/PolyMeilex/rfd). The filter option is now an array of `{ name: string, extensions: string[] }`.
  - [2326bcd](https://www.github.com/tauri-apps/tauri/commit/2326bcd399411f7f0eabdb7ade910be473adadae) refactor(core): use `nfd` for file dialogs, closes [#1251](https://www.github.com/tauri-apps/tauri/pull/1251) ([#1257](https://www.github.com/tauri-apps/tauri/pull/1257)) on 2021-02-18
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Tauri now emits events on file drops on the webview window.
  - [2db901e](https://www.github.com/tauri-apps/tauri/commit/2db901e744f51cd4296ddec4352d7a51c859b85b) feat(core): add file drop handler ([#1352](https://www.github.com/tauri-apps/tauri/pull/1352)) on 2021-03-12
- Fixes `resource_dir` resolution on AppImage.
  - [bd1df5d](https://www.github.com/tauri-apps/tauri/commit/bd1df5d80431f5de4cd905ffaf7f3f2628d6b8ab) fix: get correct resource dir in AppImge (fix [#1308](https://www.github.com/tauri-apps/tauri/pull/1308)) ([#1333](https://www.github.com/tauri-apps/tauri/pull/1333)) on 2021-03-12
- Fixed missing 'App' variant & string promise instead of void promise.
  - [44fc65c](https://www.github.com/tauri-apps/tauri/commit/44fc65c723f638f2a1b2ecafb79b32d509ed2f35) Fixing TS API typings ([#1451](https://www.github.com/tauri-apps/tauri/pull/1451)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The HTTP API was improved with client caching and better payload and response types.
  - [a7bc472](https://www.github.com/tauri-apps/tauri/commit/a7bc472e994730071f960d09a12ac85296a080ae) refactor(core): improve HTTP API, closes [#1098](https://www.github.com/tauri-apps/tauri/pull/1098) ([#1237](https://www.github.com/tauri-apps/tauri/pull/1237)) on 2021-02-15
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added new Javascript API to extract `name`, `version`, `tauri version` from the running application. We exposed `relaunch` and `exit` as well to control your application state.
  - [e511d39](https://www.github.com/tauri-apps/tauri/commit/e511d3991041a974273a2674a9bf60230b7519ee) feat(api): Expose application metadata and functions to JS api - fix [#1387](https://www.github.com/tauri-apps/tauri/pull/1387) ([#1445](https://www.github.com/tauri-apps/tauri/pull/1445)) on 2021-04-08
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- The event listener `once` kind was moved to a dedicated function.
  - [372036c](https://www.github.com/tauri-apps/tauri/commit/372036ce20ac7f103dea05bae7e8686858d096a4) refactor(api): move event's `once` to its own function ([#1276](https://www.github.com/tauri-apps/tauri/pull/1276)) on 2021-02-23
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Use ``JSON.parse(String.raw`{arg}`)`` for communicating serialized JSON objects and arrays < 1 GB to the Webview from Rust.

https://github.com/GoogleChromeLabs/json-parse-benchmark

- [eeb2030](https://www.github.com/tauri-apps/tauri/commit/eeb20308acdd83029abb6ce94fb5d0c896759060) Use JSON.parse instead of literal JS for callback formatting ([#1370](https://www.github.com/tauri-apps/tauri/pull/1370)) on 2021-04-07
- [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Added support to multiple windows.
  - [07208df](https://www.github.com/tauri-apps/tauri/commit/07208dff6c1e8cff7c10780f4f7f8cee9de44a2e) feat(core): add mult-window support ([#1217](https://www.github.com/tauri-apps/tauri/pull/1217)) on 2021-02-11
- Adds `productName` and `version` configs on `tauri.conf.json > package`.
  - [5b3d9b2](https://www.github.com/tauri-apps/tauri/commit/5b3d9b2c07da766f81981ba7c4961cd354d51340) feat(config): allow setting product name and version on tauri.conf.json ([#1358](https://www.github.com/tauri-apps/tauri/pull/1358)) on 2021-03-22
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Plugins are now configurable through a `tauri.conf.json > "plugins" > $pluginName` object.
  - [2058cc3](https://www.github.com/tauri-apps/tauri/commit/2058cc39c9ac9d9d442802db2c2f3be34a85acc4) feat(tauri): add plugin `initialize` (with config) API, run in parallel ([#1194](https://www.github.com/tauri-apps/tauri/pull/1194)) on 2021-02-10
- Renamed the `Plugin` trait `init_script` to `initialization_script`.
  - [5c5d8f8](https://www.github.com/tauri-apps/tauri/commit/5c5d8f811fc094cca1b441ff966f15c7bf5d2e90) refactor(tauri): rename `init_script` to `initialization_script` ([#1200](https://www.github.com/tauri-apps/tauri/pull/1200)) on 2021-02-10
- The plugin instance is now mutable and must be `Send`.
  - [fb607ee](https://www.github.com/tauri-apps/tauri/commit/fb607ee97a912d1e23f6d7dd6dd3c28aac9b4527) refactor(tauri): plugin trait with mutable references ([#1197](https://www.github.com/tauri-apps/tauri/pull/1197)) on 2021-02-10
  - [1318ffb](https://www.github.com/tauri-apps/tauri/commit/1318ffb47c5f2fb696d6323fbcee4f840396c6b3) refactor(core): remove async from app hooks, add InvokeMessage type ([#1392](https://www.github.com/tauri-apps/tauri/pull/1392)) on 2021-03-26
- Fixes the event system usage on the plugin `ready` hook.
  - [23132ac](https://www.github.com/tauri-apps/tauri/commit/23132acf765ab8b6a37b74151a4c175b68390657) fix(tauri): run plugin::ready without webview.dispatch ([#1164](https://www.github.com/tauri-apps/tauri/pull/1164)) on 2021-01-29
- The `allowlist` configuration now has one object per module.
  - [e0be59e](https://www.github.com/tauri-apps/tauri/commit/e0be59ea26df17fe2e31224759f21fb1d0cbdfd3) refactor(core): split allowlist configuration per module ([#1263](https://www.github.com/tauri-apps/tauri/pull/1263)) on 2021-02-20
- The Tauri script is now injected with the webview `init` API, so it is available after page changes.
  - [4412b7c](https://www.github.com/tauri-apps/tauri/commit/4412b7c438c2b10e519bf8b696e3ef827e9091f2) refactor(tauri): inject script with webview init API ([#1186](https://www.github.com/tauri-apps/tauri/pull/1186)) on 2021-02-05
  - [8bdd894](https://www.github.com/tauri-apps/tauri/commit/8bdd8949254d63bfc57ad67ce2592d40a0b44bf8) refactor(core): move bundle script to /tauri crate ([#1377](https://www.github.com/tauri-apps/tauri/pull/1377)) on 2021-03-23
- Removed the `no-server` mode, the `inliner`, the `dev` server proxy and the `loadAsset` API.
  - [84d7cda](https://www.github.com/tauri-apps/tauri/commit/84d7cdae632eeb02a66f8d1d7577adfa65917a34) refactor(core): remove `no-server` and its APIs ([#1215](https://www.github.com/tauri-apps/tauri/pull/1215)) on 2021-02-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Adds a global shortcut API.
  - [855effa](https://www.github.com/tauri-apps/tauri/commit/855effadd9ebfb6bc1a3555ac7fc733f6f766b7a) feat(core): globalShortcut API ([#1232](https://www.github.com/tauri-apps/tauri/pull/1232)) on 2021-02-14
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Added `async` support to the Tauri Rust core on commit [#a169b67](https://github.com/tauri-apps/tauri/commit/a169b67ef0277b958bdac97e33c6e4c41b6844c3).
  - [2bf55f8](https://www.github.com/tauri-apps/tauri/commit/2bf55f80564f5c31d89384bd4a82dd55307f8c75) chore: add changefile on 2021-02-03
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The Tauri integration with Webview was refactored to use traits, which allows custom implementations by developers and simplifies changes on the webview implementation.
  - [b9ce7b9](https://www.github.com/tauri-apps/tauri/commit/b9ce7b94c4eb027bcbbd4ee600b75a7407f108ca) refactor(tauri): Webview traits ([#1183](https://www.github.com/tauri-apps/tauri/pull/1183)) on 2021-02-05
- Added window management and window creation APIs.
  - [a3d6dff](https://www.github.com/tauri-apps/tauri/commit/a3d6dff2163c7a45842253edd81dbc62248dc65d) feat(core): window API ([#1225](https://www.github.com/tauri-apps/tauri/pull/1225)) on 2021-02-13
  - [641374b](https://www.github.com/tauri-apps/tauri/commit/641374b15343518cd835bd5ada811941c65dcf2e) feat(core): window creation at runtime ([#1249](https://www.github.com/tauri-apps/tauri/pull/1249)) on 2021-02-17
- Use [WRY](https://github.com/tauri-apps/wry) as Webview interface, thanks to @wusyong.
  - [99ecf7b](https://www.github.com/tauri-apps/tauri/commit/99ecf7bb3e8da6e611b57c6680a14b65179f8a35) feat(tauri): use WRY as webview engine ([#1190](https://www.github.com/tauri-apps/tauri/pull/1190)) on 2021-02-08

## \[0.11.1]

- Update webview-official dependency which fix compatibility on macOS.
  - [692312a](https://www.github.com/tauri-apps/tauri/commit/692312a0f51a05dd418d9cad553a695f3347b943) chore(deps) Update webview-official ([#1152](https://www.github.com/tauri-apps/tauri/pull/1152)) on 2021-01-24

## \[0.11.0]

- Match writeBinaryFile command name between js and rust
  - [486bd92](https://www.github.com/tauri-apps/tauri/commit/486bd920f899905bec0f690092aa1e3cac2c78f3) Fix: writeBinaryFile to call the correct command (fix [#1133](https://www.github.com/tauri-apps/tauri/pull/1133)) ([#1136](https://www.github.com/tauri-apps/tauri/pull/1136)) on 2021-01-06

## \[0.10.0]

- Adds missing APIs features from `allowlist` to the tauri crate's manifest file.
  - [2c0f09c](https://www.github.com/tauri-apps/tauri/commit/2c0f09c85c8a60c2fa304fb25174d5020663f0d7) fix(tauri) add missing API features, closes [#1023](https://www.github.com/tauri-apps/tauri/pull/1023) ([#1052](https://www.github.com/tauri-apps/tauri/pull/1052)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Adds a path resolution API (e.g. getting the download directory or resolving a path to the home directory).
  - [771e401](https://www.github.com/tauri-apps/tauri/commit/771e4019b8cfd1973015ffa632c9d6c6b82c5657) feat: Port path api to js ([#1006](https://www.github.com/tauri-apps/tauri/pull/1006)) on 2020-09-24
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Update minimum Rust version to 1.42.0 due to a dependency update.
  - [d13dcd9](https://www.github.com/tauri-apps/tauri/commit/d13dcd9fd8d30b1db147a78cecb878e924382274) chore(deps) Update Tauri Bundler ([#1045](https://www.github.com/tauri-apps/tauri/pull/1045)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Minimum Rust version updated to 1.47.0. Run `$ rustup update` to update to the latest version.
  - [b4544b6](https://www.github.com/tauri-apps/tauri/commit/b4544b63f268dc6f3f47a4bfbad5d72cceee8698) chore(deps) Update Tauri API ([#1072](https://www.github.com/tauri-apps/tauri/pull/1072)) on 2020-11-07

## \[0.9.2]

- Bump all deps as noted in #975, #976, #977, #978, and #979.
  - [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20

## \[0.9.1]

- Adjust payload formatting to handle multibyte characters in front-end message
  payloads.
  \- [df70ca5](https://www.github.com/tauri-apps/tauri/commit/df70ca51965665952a74161cc6eb1ff19eae45e2) Fix [#912](https://www.github.com/tauri-apps/tauri/pull/912) multibyte character breaks message ([#914](https://www.github.com/tauri-apps/tauri/pull/914)) on 2020-08-01

## \[0.9.0]

- Make sure CSS content loaded with the `loadAsset` API is inside a template string and not injected raw.
  - [e3e2e39](https://www.github.com/tauri-apps/tauri/commit/e3e2e3920833627400ee7a5b000dc6e51d8d332b) fix(tauri) ensure css content is loaded inside a string ([#884](https://www.github.com/tauri-apps/tauri/pull/884)) on 2020-07-22
  - [b96b1fb](https://www.github.com/tauri-apps/tauri/commit/b96b1fb6b8a4f565fb946847bb9a29d9d939e2cb) inject css with template string to allow for line breaks ([#894](https://www.github.com/tauri-apps/tauri/pull/894)) on 2020-07-25
- Pin the `tauri-api` dependency on the `tauri` crate so updates doesn't crash the build.
  - [ad717c6](https://www.github.com/tauri-apps/tauri/commit/ad717c6f33b4d6e20fbb13cbe30e06946dbb74f6) chore(tauri) pin tauri-api dep version ([#872](https://www.github.com/tauri-apps/tauri/pull/872)) on 2020-07-21
- Fixes the Webview initialization on Windows.
  - [4abd12c](https://www.github.com/tauri-apps/tauri/commit/4abd12c2a42b5ace8527114ab64da38f4486754f) fix(tauri) webview initialization on windows, fixes [#879](https://www.github.com/tauri-apps/tauri/pull/879) ([#885](https://www.github.com/tauri-apps/tauri/pull/885)) on 2020-07-23

## \[0.8.0]

- Use native dialog on `window.alert` and `window.confirm`.
  Since every communication with the webview is asynchronous, the `window.confirm` returns a Promise resolving to a boolean value.
  \- [0245833](https://www.github.com/tauri-apps/tauri/commit/0245833bb56d5462a4e1249eb1e2f9f5e477592d) feat(tauri) make `window.alert` and `window.confirm` available, fix [#848](https://www.github.com/tauri-apps/tauri/pull/848) ([#854](https://www.github.com/tauri-apps/tauri/pull/854)) on 2020-07-18
  \- [dac0ae9](https://www.github.com/tauri-apps/tauri/commit/dac0ae976ea1b419ed5af078d00106b1476dee04) chore(changes) add tauri-api to JS dialogs changefile on 2020-07-19
- The notification's `body` is now optional, closes #793.
  - [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
- Fixes a regression on the storage of requestPermission response.
  ß
  \- [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
- Plugin system added. You can hook into the webview lifecycle (`created`, `ready`) and extend the API adding logic to the `invoke_handler` by implementing the `tauri::plugin::Plugin` trait.
  - [78afee9](https://www.github.com/tauri-apps/tauri/commit/78afee9725e0e372f9de7edeaac523011a1c02a3) feat(tauri) add plugin system for rust ([#494](https://www.github.com/tauri-apps/tauri/pull/494)) on 2020-07-12
- Renaming `whitelist` to `allowlist` (see #645).
  - [a6bb3b5](https://www.github.com/tauri-apps/tauri/commit/a6bb3b59059e08a844d7bb2b43f3d1192954d890) refactor(tauri) rename `whitelist` to `allowlist`, ref [#645](https://www.github.com/tauri-apps/tauri/pull/645) ([#858](https://www.github.com/tauri-apps/tauri/pull/858)) on 2020-07-19
- Moving the webview implementation to [webview](https://github.com/webview/webview), with the [official Rust binding](https://github.com/webview/webview_rust).
  This is a breaking change.
  IE support has been dropped, so the `edge` object on `tauri.conf.json > tauri` no longer exists and you need to remove it.
  `webview.handle()` has been replaced with `webview.as_mut()`.
  \- [cd5b401](https://www.github.com/tauri-apps/tauri/commit/cd5b401707d709bf8212b0a4c34623f902ae40f9) feature: import official webview rust binding ([#846](https://www.github.com/tauri-apps/tauri/pull/846)) on 2020-07-18

## \[0.7.5]

- Fixes Edge blank screen on Windows when running tauri dev (Tauri crashing window due to Edge reloading app because of missing Content-Type header).
  - Bumped due to a bump in tauri-api.
  - [fedee83](https://www.github.com/tauri-apps/tauri/commit/fedee835e36daa4363b91aabd43143e8033c9a5c) fix(tauri.js) windows Edge blank screen on tauri dev ([#808](https://www.github.com/tauri-apps/tauri/pull/808)) on 2020-07-11

## \[0.7.4]

- Ignoring non UTF-8 characters on the loopback command output.
  - [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## \[0.7.3]

- Properly run the loopback command on Windows.
- Properly ignore the ${distDir}/index.html asset from the asset embbeding. Previously every asset with name matching /(.+)index.html$/g were ignored.

## \[0.7.2]

Bumped due to dependency.

## \[0.7.1]

- Fixes the assets embedding into the binary.

## \[0.7.0]

- The execute_promise and execute_promise_sync helpers now accepts any tauri::Result<T> where T: impl Serialize.
  This means that you do not need to serialize your response manually or deal with String quotes anymore.
  As part of this refactor, the event::emit function also supports impl Serialize instead of String.

## \[0.6.2]

- Fixes the Windows build with the latest Windows SDK.

## \[0.6.1] - (Not Published)

## \[0.6.0]

- Adds a command line interface option to tauri apps, configurable under tauri.conf.json > tauri > cli.
- Fixes no-server mode not running on another machine due to fs::read_to_string usage instead of the include_str macro.
  Build no longer fails when compiling without environment variables, now the app will show an error.
- Adds desktop notifications API.
- Properly reflect tauri.conf.json changes on app when running tauri dev.
