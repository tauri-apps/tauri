# Changelog

## \[1.0.0-rc.3]

- Parse window icons at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05

## \[1.0.0-rc.2]

- Changed the default value for `tauri > bundle > macOS > minimumSystemVersion` to `10.13`.
  - Bumped due to a bump in tauri-utils.
  - [fce344b9](https://www.github.com/tauri-apps/tauri/commit/fce344b90b7227f8f5514853c2f885fb24d3648e) feat(core): set default value for `minimum_system_version` to 10.13 ([#3497](https://www.github.com/tauri-apps/tauri/pull/3497)) on 2022-02-17

## \[1.0.0-rc.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.0]

- Adds support for using JSON5 format for the `tauri.conf.json` file, along with also supporting the `.json5` extension.

Here is the logic flow that determines if JSON or JSON5 will be used to parse the config:

1. Check if `tauri.conf.json` exists
   a. Parse it with `serde_json`
   b. Parse it with `json5` if `serde_json` fails
   c. Return original `serde_json` error if all above steps failed
2. Check if `tauri.conf.json5` exists
   a. Parse it with `json5`
   b. Return error if all above steps failed
3. Return error if all above steps failed

- [995de57a](https://www.github.com/tauri-apps/tauri/commit/995de57a76cf51215277673e526d7ec32b86b564) Add seamless support for using JSON5 in the config file ([#47](https://www.github.com/tauri-apps/tauri/pull/47)) on 2022-02-03
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22

## \[1.0.0-beta.5]

- Embed Info.plist file contents on binary on dev.
  - Bumped due to a bump in tauri-codegen.
  - [537ab1b6](https://www.github.com/tauri-apps/tauri/commit/537ab1b6d5a792c550a535619965c9e4126292e6) feat(core): inject src-tauri/Info.plist file on dev and merge on bundle, closes [#1570](https://www.github.com/tauri-apps/tauri/pull/1570) [#2338](https://www.github.com/tauri-apps/tauri/pull/2338) ([#2444](https://www.github.com/tauri-apps/tauri/pull/2444)) on 2021-08-15
- Fix ES Module detection for default imports with relative paths or scoped packages and exporting of async functions.
  - Bumped due to a bump in tauri-codegen.
  - [b2b36cfe](https://www.github.com/tauri-apps/tauri/commit/b2b36cfe8dfcccb341638a4cb6dc23a514c54148) fix(core): fixes ES Module detection for default imports with relative paths or scoped packages ([#2380](https://www.github.com/tauri-apps/tauri/pull/2380)) on 2021-08-10
  - [fbf8caf5](https://www.github.com/tauri-apps/tauri/commit/fbf8caf5c419cb4fc3d123be910e094a8e8c4bef) fix(core): ESM detection when using `export async function` ([#2425](https://www.github.com/tauri-apps/tauri/pull/2425)) on 2021-08-14

## \[1.0.0-beta.4]

- `Params` has been removed, along with all the associated types on it. Functions that previously accepted those
  associated types now accept strings instead. Type that used a generic parameter `Params` now use `Runtime` instead. If
  you use the `wry` feature, then types with a `Runtime` generic parameter should default to `Wry`, letting you omit the
  explicit type and let the compiler infer it instead.

`tauri`:

- See `Params` note
- If you were using `Params` inside a function parameter or definition, all references to it have been replaced with a
  simple runtime that defaults to `Wry`. If you are not using a custom runtime, just remove `Params` from the definition
  of functions/items that previously took it. If you are using a custom runtime, you *may* need to pass the runtime type
  to these functions.
- If you were using custom types for `Params` (uncommon and if you don't understand you probably were not using it), all
  methods that were previously taking the custom type now takes an `Into<String>` or a `&str`. The types were already
  required to be string-able, so just make sure to convert it into a string before passing it in if this breaking change
  affects you.

`tauri-macros`:

- (internal) Added private `default_runtime` proc macro to allow us to give item definitions a custom runtime only when
  the specified feature is enabled.

`tauri-runtime`:

- See `Params` note
- Removed `Params`, `MenuId`, `Tag`, `TagRef`.
- Added `menu::{MenuHash, MenuId, MenuIdRef}` as type aliases for the internal type that menu types now use.
  - All previous menu items that had a `MenuId` generic now use the underlying `MenuId` type without a generic.
- `Runtime`, `RuntimeHandle`, and `Dispatch` have no more generic parameter on `create_window(...)` and instead use the
  `Runtime` type directly
- `Runtime::system_tray` has no more `MenuId` generic and uses the string based `SystemTray` type directly.
- (internal) `CustomMenuItem::id_value()` is now hashed on creation and exposed as the `id` field with type `MenuHash`.

`tauri-runtime-wry`:

- See `Params` note
- update menu and runtime related types to the ones changed in `tauri-runtime`.

`tauri-utils`:

- `Assets::get` signature has changed to take a `&AssetKey` instead of `impl Into<AssetKey>` to become trait object
  safe.
- [fd8fab50](https://www.github.com/tauri-apps/tauri/commit/fd8fab507c8fa1b113b841af14c6693eb3955f6b) refactor(core): remove `Params` and replace with strings ([#2191](https://www.github.com/tauri-apps/tauri/pull/2191)) on 2021-07-15

## \[1.0.0-beta.3]

- Detect ESM scripts and inject the invoke key directly instead of using an IIFE.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Improve invoke key code injection performance time rewriting code at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27

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
