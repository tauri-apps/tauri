# Changelog

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - Bumped due to a bump in cli.rs.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- Fixed missing 'App' variant & string promise instead of void promise.
  - [44fc65c](https://www.github.com/tauri-apps/tauri/commit/44fc65c723f638f2a1b2ecafb79b32d509ed2f35) Fixing TS API typings ([#1451](https://www.github.com/tauri-apps/tauri/pull/1451)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- JS CLI now downloads prebuilt Rust CLI.
  - [df305b2](https://www.github.com/tauri-apps/tauri/commit/df305b234bfe487f429949df90e522c1b0ce85c9) feat(cli/tauri.js): download prebuilt cli ([#1452](https://www.github.com/tauri-apps/tauri/pull/1452)) on 2021-04-13
- The `info` command was rewritten in Rust.
  - [c3e06ee](https://www.github.com/tauri-apps/tauri/commit/c3e06ee9e88b3631da6eeb17d61ddd41cd5c6fe9) refactor(cli): rewrite info in Rust ([#1389](https://www.github.com/tauri-apps/tauri/pull/1389)) on 2021-03-25
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `init` command was rewritten in Rust.
  - [f72b93b](https://www.github.com/tauri-apps/tauri/commit/f72b93b676ba8c48fd9273c187de3dbbc410fa0f) refactor(cli): rewrite init command in Rust ([#1382](https://www.github.com/tauri-apps/tauri/pull/1382)) on 2021-03-24
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Removed the `no-server` mode, the `inliner`, the `dev` server proxy and the `loadAsset` API.
  - [84d7cda](https://www.github.com/tauri-apps/tauri/commit/84d7cdae632eeb02a66f8d1d7577adfa65917a34) refactor(core): remove `no-server` and its APIs ([#1215](https://www.github.com/tauri-apps/tauri/pull/1215)) on 2021-02-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Revert `tauri create` deletion and shift remaining pieces that weren't deleted to `create-tauri-app`.
  - [4ec20a4](https://www.github.com/tauri-apps/tauri/commit/4ec20a4a28823614186365c5a90512d77170cff2) feat: shift tauri create \[not wired up] ([#1330](https://www.github.com/tauri-apps/tauri/pull/1330)) on 2021-03-07
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The Tauri API interface is now shipped with the `@tauri-apps/api` package instead of the deprecated `tauri` package.
  To use the new API package, delete the old `tauri` from your `package.json` and install the new package:
  `$ yarn remove tauri && yarn add @tauri-apps/api` or `$ npm uninstall tauri && npm install @tauri-apps/api`.
  And change all `import { someApi } from 'tauri/api` to `import { someApi } from '@tauri-apps/api'`.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The Tauri Node.js CLI package is now `@tauri-apps/cli`.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- All the arguments passed after `tauri dev --` are now propagated to the binary.
  - [4e9d31c](https://www.github.com/tauri-apps/tauri/commit/4e9d31c70ba13f1cabe830c6519a1b5f4789fd7b) feat(cli): propagate args passed after `dev --`, closes [#1406](https://www.github.com/tauri-apps/tauri/pull/1406) ([#1407](https://www.github.com/tauri-apps/tauri/pull/1407)) on 2021-03-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Eliminate the dead code in cli.js. Also removed some unused dependencies and updated the publish config that we are not publishing the Typescript files (since this is now primarily a CLI).
  - [c718bd2](https://www.github.com/tauri-apps/tauri/commit/c718bd2382a289666cd665bb356a39193c06deb6) chore: tauri.js dead code elim ([#1422](https://www.github.com/tauri-apps/tauri/pull/1422)) on 2021-04-01
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12

## \[0.14.1]

- Fixed a TypeScript issue where it didn't allow you to put the "recursive" option in the directory functions.
  - [2fd1067](https://www.github.com/tauri-apps/tauri/commit/2fd1067a4c7ef86dda074867b6a6702527962829) Fix: add recursive option to directory APIs ([#1141](https://www.github.com/tauri-apps/tauri/pull/1141)) on 2021-01-12

## \[0.14.0]

- Update the tauri template to properly set the app icon id on Windows so the webview can load the executable icon.
  To use it on old projects, update your `src-tauri/src/build.rs` file, replacing `res.set_icon("icons/icon.ico");` with `res.set_icon_with_id("icons/icon.ico", "32512");`.
  \- [f887320](https://www.github.com/tauri-apps/tauri/commit/f887320df35e44b56e437355ee0ff05507a83173) fix(template) default windows icon id should be 32512, fixes [#1099](https://www.github.com/tauri-apps/tauri/pull/1099) ([#1107](https://www.github.com/tauri-apps/tauri/pull/1107)) on 2020-12-05
- Fixes `tauri deps` command usage when `npm` is not installed.
  - [8da495f](https://www.github.com/tauri-apps/tauri/commit/8da495f78c34780b76d0b1201f622edcbba00229) fix(tauri.js) `deps` cmd usage when `npm` is not installed, closes [#1037](https://www.github.com/tauri-apps/tauri/pull/1037) ([#1053](https://www.github.com/tauri-apps/tauri/pull/1053)) on 2020-12-05
- Match writeBinaryFile command name between js and rust
  - [486bd92](https://www.github.com/tauri-apps/tauri/commit/486bd920f899905bec0f690092aa1e3cac2c78f3) Fix: writeBinaryFile to call the correct command (fix [#1133](https://www.github.com/tauri-apps/tauri/pull/1133)) ([#1136](https://www.github.com/tauri-apps/tauri/pull/1136)) on 2021-01-06

## \[0.13.0]

- Fixes `Reflect.deleteProperty` on promisified API calls failing with `Unable to delete property` by making it configurable.
  - [c8b167a](https://www.github.com/tauri-apps/tauri/commit/c8b167adb3561db182bc8f6e4d8753ce1ae3f450) fix(tauri.js) promisified API fails on Reflect.deleteProperty, fix [#1038](https://www.github.com/tauri-apps/tauri/pull/1038) ([#1056](https://www.github.com/tauri-apps/tauri/pull/1056)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Adds a path resolution API (e.g. getting the download directory or resolving a path to the home directory).
  - [771e401](https://www.github.com/tauri-apps/tauri/commit/771e4019b8cfd1973015ffa632c9d6c6b82c5657) feat: Port path api to js ([#1006](https://www.github.com/tauri-apps/tauri/pull/1006)) on 2020-09-24
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21

## \[0.12.0]

- Break out TauriBuildConfig interface from TauriConfig build property
  - [43a8c4d](https://www.github.com/tauri-apps/tauri/commit/43a8c4d2bcc2461232e2ddfdf2506d3b4d68471d) fix [#920](https://www.github.com/tauri-apps/tauri/pull/920): Create recipes ([#930](https://www.github.com/tauri-apps/tauri/pull/930)) on 2020-08-17
- Create recipes. A recipe:
  - Updates the TauriBuildConfig during the init process
  - Specifies npm dev and production dependencies to be installed
  - Runs extra installation scripts
  - [43a8c4d](https://www.github.com/tauri-apps/tauri/commit/43a8c4d2bcc2461232e2ddfdf2506d3b4d68471d) fix [#920](https://www.github.com/tauri-apps/tauri/pull/920): Create recipes ([#930](https://www.github.com/tauri-apps/tauri/pull/930)) on 2020-08-17
- Create React JS and React TS recipes
  - [43a8c4d](https://www.github.com/tauri-apps/tauri/commit/43a8c4d2bcc2461232e2ddfdf2506d3b4d68471d) fix [#920](https://www.github.com/tauri-apps/tauri/pull/920): Create recipes ([#930](https://www.github.com/tauri-apps/tauri/pull/930)) on 2020-08-17
- Add new top level command `create`, which accepts a recipe as a CLI, or runs interactively, prompting for a recipe out of a menu of choices defined by `api/recipes/index`
  - [43a8c4d](https://www.github.com/tauri-apps/tauri/commit/43a8c4d2bcc2461232e2ddfdf2506d3b4d68471d) fix [#920](https://www.github.com/tauri-apps/tauri/pull/920): Create recipes ([#930](https://www.github.com/tauri-apps/tauri/pull/930)) on 2020-08-17
- Refactor `init` command so that it is just an alias for `create` with no recipe
  - [43a8c4d](https://www.github.com/tauri-apps/tauri/commit/43a8c4d2bcc2461232e2ddfdf2506d3b4d68471d) fix [#920](https://www.github.com/tauri-apps/tauri/pull/920): Create recipes ([#930](https://www.github.com/tauri-apps/tauri/pull/930)) on 2020-08-17
- Bump all deps as noted in #975, #976, #977, #978, and #979.
  - [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20
- Make interactive prompt not ask for app name supplied as cli arg
  - [59e0de7](https://www.github.com/tauri-apps/tauri/commit/59e0de765046a240d6c9ff3ddcd7a98e8f765512) Fix cli no prompt for app-name cli arg ([#980](https://www.github.com/tauri-apps/tauri/pull/980)) on 2020-08-19
- Change `String` to `string` type for `open` and `save` methods
  - [0a5bac1](https://www.github.com/tauri-apps/tauri/commit/0a5bac1dd641792a64f79ec90e2a357f18280776) fix(tauri.js): fix typings for open and save dialogs ([#926](https://www.github.com/tauri-apps/tauri/pull/926)) on 2020-08-08
- Format all code with prettier. This technically should only affect code styles, but noting for posterity.
  - [6a21965](https://www.github.com/tauri-apps/tauri/commit/6a21965ff302940bcbdefa16490249ec7d0c1f2e) chore: add prettier for js formatting ([#937](https://www.github.com/tauri-apps/tauri/pull/937)) on 2020-08-18
- Set correct promise resolve type which returns from `readBinaryFile`
  - [f98d4b9](https://www.github.com/tauri-apps/tauri/commit/f98d4b9076b51a7fc9eca12b4bed2cd3b466c6bc) fix(tauri.js): fix return type for `readBinaryFile` api method ([#927](https://www.github.com/tauri-apps/tauri/pull/927)) on 2020-08-08
- Add types to JSDoc annotations
  - [f98d4b9](https://www.github.com/tauri-apps/tauri/commit/f98d4b9076b51a7fc9eca12b4bed2cd3b466c6bc) fix(tauri.js): fix return type for `readBinaryFile` api method ([#927](https://www.github.com/tauri-apps/tauri/pull/927)) on 2020-08-08

## \[0.11.1]

- Fix command line arguments -W (window title) and -P (dev server uri) to work as intended.
  - [e1fd626](https://www.github.com/tauri-apps/tauri/commit/e1fd626453bf6310b18e48472aa831c367212290) Fix typos referring to CLI args in init command ([#921](https://www.github.com/tauri-apps/tauri/pull/921)) on 2020-08-03

## \[0.11.0]

- Fixes the Webview initialization on Windows.
  - [4abd12c](https://www.github.com/tauri-apps/tauri/commit/4abd12c2a42b5ace8527114ab64da38f4486754f) fix(tauri) webview initialization on windows, fixes [#879](https://www.github.com/tauri-apps/tauri/pull/879) ([#885](https://www.github.com/tauri-apps/tauri/pull/885)) on 2020-07-23

## \[0.10.0]

- Fixes the `writeFile` and `writeBinaryFile` usage.
  - [cbd14c3](https://www.github.com/tauri-apps/tauri/commit/cbd14c307753449d2d8a9cd4d4b29d30af6a7097) fix(tauri.js) `writeFile` and `writeBinaryFile` API ([#857](https://www.github.com/tauri-apps/tauri/pull/857)) on 2020-07-19
- The notification's `body` is now optional, closes #793.
  - [dac1db3](https://www.github.com/tauri-apps/tauri/commit/dac1db39831ecbcf23c630351d5753af01ccd500) fix(tauri) notification body optional, requestPermission() regression, closes [#793](https://www.github.com/tauri-apps/tauri/pull/793) ([#844](https://www.github.com/tauri-apps/tauri/pull/844)) on 2020-07-16
- Fixes a memory leak on the `promisified` helper usage.
  - [42a8bb0](https://www.github.com/tauri-apps/tauri/commit/42a8bb0e096548f2f9d6da2ba3699260e6cda18e) fix(api) `promisified` not cleaning up transformed callbacks, fixes [#852](https://www.github.com/tauri-apps/tauri/pull/852) ([#853](https://www.github.com/tauri-apps/tauri/pull/853)) on 2020-07-18
- Prevent running the `dev` pipeline when running with administrator privileges.
  - [1780057](https://www.github.com/tauri-apps/tauri/commit/17800571fe417b5250aa1bd7052340a1c93918a8) fix(tauri.js) exit dev when running as admin, fixes [#781](https://www.github.com/tauri-apps/tauri/pull/781) ([#839](https://www.github.com/tauri-apps/tauri/pull/839)) on 2020-07-15
- Print outdated dependencies information on `tauri info`.
  - [f0ce94f](https://www.github.com/tauri-apps/tauri/commit/f0ce94fc8e38642f2ba479311370dc1ca54799c7) feat(tauri.js) print outdated deps information on `tauri info` ([#841](https://www.github.com/tauri-apps/tauri/pull/841)) on 2020-07-15
- Convert the `--app-name` value to kebab case.
  - [da99f63](https://www.github.com/tauri-apps/tauri/commit/da99f632f0c8a6b3b7fc5dfecaffb04b74537f0f) fix(tauri.js) app name as kebab case ([#856](https://www.github.com/tauri-apps/tauri/pull/856)) on 2020-07-19
- Do not require a `package.json` file on the app root.
  - [45d3de6](https://www.github.com/tauri-apps/tauri/commit/45d3de6d97f060659e72e0cc0dc56d4f33f4a2f9) fix(tauri.js) do not require a package.json ([#855](https://www.github.com/tauri-apps/tauri/pull/855)) on 2020-07-19
- Adds a dependency manager command to the Node.js CLI (`tauri deps`). The manager is able to install and update Rust and the Tauri ecosystem dependencies (npm package, crates, cargo subcommands).
  Usage: `tauri deps install` and `tauri deps update`. - [77282c1](https://www.github.com/tauri-apps/tauri/commit/77282c1e513227fe379f916cd21249b44faa8756) feat(tauri.js) add dependency manager command ([#829](https://www.github.com/tauri-apps/tauri/pull/829)) on 2020-07-15
- Run the dependency manager's install script after `tauri init` succeeds.
  - [0591f1f](https://www.github.com/tauri-apps/tauri/commit/0591f1f945420ec4bc53919d05a8f8de014b3823) feat(tauri.js) run `deps install` after `tauri init` ([#842](https://www.github.com/tauri-apps/tauri/pull/842)) on 2020-07-15
- Move types exported in the `tauri` js api into the modules that use them. For
  example, `Event` is now available from `tauri/api/event` instead of
  `tauri/api/types/event`. - [660a2d8](https://www.github.com/tauri-apps/tauri/commit/660a2d87d6acf0abf6be70c01e6402cb5aba96c7) feat(tauri.js) move exported api types into api modules (fix [#807](https://www.github.com/tauri-apps/tauri/pull/807)) ([#809](https://www.github.com/tauri-apps/tauri/pull/809)) on 2020-07-12

## \[0.9.1]

- Fixes Edge blank screen on Windows when running tauri dev (Tauri crashing window due to Edge reloading app because of missing Content-Type header).
  - Bumped due to a bump in tauri-api.
  - [fedee83](https://www.github.com/tauri-apps/tauri/commit/fedee835e36daa4363b91aabd43143e8033c9a5c) fix(tauri.js) windows Edge blank screen on tauri dev ([#808](https://www.github.com/tauri-apps/tauri/pull/808)) on 2020-07-11
- Improve the tauri info output on Windows, including the Microsoft Edge version.
  - [0d6235e](https://www.github.com/tauri-apps/tauri/commit/0d6235e427c0f8241d1068bdd1e34903eb9298f9) feat(tauri.js) add microsoft edge version to the info output ([#810](https://www.github.com/tauri-apps/tauri/pull/810)) on 2020-07-12

## \[0.9.0]

- Fixes a race condition on the beforeDevCommand usage (starting Tauri before the devServer is ready).
  - [a26cffc](https://www.github.com/tauri-apps/tauri/commit/a26cffc575bee224a6beb5b7b0565d5583c0131f) fix(tauri.js) beforeDevCommand race condition ([#801](https://www.github.com/tauri-apps/tauri/pull/801)) on 2020-07-10
- Revert a nullish coalescing operator that changed embedded server/inliner behavior.
  - [e7b4951](https://www.github.com/tauri-apps/tauri/commit/e7b495133fe9f4e9f576bb9805bec98b967783eb) fix(tauri.js) revert nullish coalesce addition ([#799](https://www.github.com/tauri-apps/tauri/pull/799)) on 2020-07-10
- Fixes tauri init not generating tauri.conf.json on the Vue CLI Plugin.
  - [f208a68](https://www.github.com/tauri-apps/tauri/commit/f208a68e40c804daf41d54539d3a5951679e8a64) fix(tauri.js) do not swallow init errors, fix conf inject ([#802](https://www.github.com/tauri-apps/tauri/pull/802)) on 2020-07-10
- tauri init now prompt for default values such as window title, app name, dist dir and dev path. You can use --ci to skip the prompts.
  - [ee8724b](https://www.github.com/tauri-apps/tauri/commit/ee8724b90a63f281292c6eb174773b905ba52e32) feat(tauri.js/init): prompt for default values (fix [#422](https://www.github.com/tauri-apps/tauri/pull/422)/[#162](https://www.github.com/tauri-apps/tauri/pull/162)) ([#472](https://www.github.com/tauri-apps/tauri/pull/472)) on 2020-07-10

## \[0.8.4]

- Bump lodash to 4.17.19

## \[0.8.3]

- Fixes the wrong cli value on the template that's used by tauri init.
  Also fixes the template test.
- Fixes the tauri icon usage with the --icon flag. Previously, only the -i flag worked.

## \[0.8.2]

- Adds tauri.conf.json schema validation to the CLI.

## \[0.8.1]

- Transpile the TS API to ES5.
  Expose CJS as .js and ESM as .mjs.
- Fixes the assets embedding into the binary.

## \[0.8.0]

- Create UMD, ESM and CJS artifacts for the JavaScript API entry point from TS source using rollup.
- Renaming window.tauri to window.\__TAURI\_\_, closing #435.
  The **Tauri** object now follows the TypeScript API structure (e.g. window.tauri.readTextFile is now window.\__TAURI\_\_.fs.readTextFile).
  If you want to keep the `window.tauri` object for a while, you can add a [mapping object](https://gist.github.com/lucasfernog/8f7b29cadd91d92ee2cf816a20c2ef01) to your code.
