# Changelog

## \[1.0.0-beta.2]

- internal: Refactor all macro code that expects specific bindings to be passed Idents
  - [39f8f269](https://www.github.com/tauri-apps/tauri/commit/39f8f269164d2fda3d5b614a193b12bb266e4b4b) refactor(macros): explicitly pass idents ([#1812](https://www.github.com/tauri-apps/tauri/pull/1812)) on 2021-05-13

## \[1.0.0-beta.1]

- Fixes a name collision when the command function is named `invoke`.
  - [7862ec5](https://www.github.com/tauri-apps/tauri/commit/7862ec562fa70e3733263ce1f690d6cd2943c0b4) fix(macros): change invoke binding in generate handler ([#1804](https://www.github.com/tauri-apps/tauri/pull/1804)) on 2021-05-12
- Fixes a name collision when the command function is named `message` or `resolver`.
  - [0b87532](https://www.github.com/tauri-apps/tauri/commit/0b875327067ca825ff6f6f26c9b2ce6fcb001e79) fix(macros): fix rest of command collisons ([#1805](https://www.github.com/tauri-apps/tauri/pull/1805)) on 2021-05-12
- Fixes a name collision when the command function is named `cmd`.
  - [d36b726](https://www.github.com/tauri-apps/tauri/commit/d36b7269261d329dd7d7fcd4d5098f3fca167364) fix(macros): collision when command is named `cmd` ([#1802](https://www.github.com/tauri-apps/tauri/pull/1802)) on 2021-05-12

## \[1.0.0-beta.0]

- Only commands with a `async fn` are executed on a separate task. `#[command] fn command_name` runs on the main thread.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09
- `#[command]` now generates a macro instead of a function to allow passing through `Params` and other generics.
  `generate_handler!` has been changed to consume the generated `#[command]` macro
  - [1453d4b](https://www.github.com/tauri-apps/tauri/commit/1453d4bf842ed6891ec604e0635344c930282189) feat(core): support generics (especially Param) in #\[command] ([#1622](https://www.github.com/tauri-apps/tauri/pull/1622)) on 2021-05-05
- Improves support for commands returning `Result`.
  - [bb8dafb](https://www.github.com/tauri-apps/tauri/commit/bb8dafbe1ea6edde7385631d41ac05e96a3309ef) feat(core): #\[command] return with autoref specialization workaround fix [#1672](https://www.github.com/tauri-apps/tauri/pull/1672) ([#1734](https://www.github.com/tauri-apps/tauri/pull/1734)) on 2021-05-09
- Adds support to command state, triggered when a command argument is `arg: State<'_, StateType>`.
  - [8b6f3de](https://www.github.com/tauri-apps/tauri/commit/8b6f3de0ad47684e72a2ae5f884d8675acfaeeac) feat(core): add state management, closes [#1655](https://www.github.com/tauri-apps/tauri/pull/1655) ([#1665](https://www.github.com/tauri-apps/tauri/pull/1665)) on 2021-05-02

## \[1.0.0-beta-rc.1]

- Fixes the Message `command` name value on plugin invoke handler.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.0]

- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Added new macros to simplify the creation of commands that can be called by the webview.
  - [1f2e7a3](https://www.github.com/tauri-apps/tauri/commit/1f2e7a3226ccf0ee3e30ae0cba3c67f7e219d1f2) feat(core): improved command matching with macros, fixes [#1157](https://www.github.com/tauri-apps/tauri/pull/1157) ([#1301](https://www.github.com/tauri-apps/tauri/pull/1301)) on 2021-02-28
