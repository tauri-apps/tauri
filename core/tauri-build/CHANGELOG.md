# Changelog

## \[1.2.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - Bumped due to a bump in tauri-utils.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[1.2.0]

- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28

## \[1.1.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.1.0]

- Rerun codegen if assets or icons change.
  - [ff8fd761](https://www.github.com/tauri-apps/tauri/commit/ff8fd7619ae894b70f149b192d8635d842827141) fix(tauri-build): rerun if assets or icons change ([#4910](https://www.github.com/tauri-apps/tauri/pull/4910)) on 2022-08-10
- Create the `desktop` and `mobile` cfg aliases.
  - [c04d0340](https://www.github.com/tauri-apps/tauri/commit/c04d0340e2f163483f3556c7aabe35322ee71e6a) feat(core): prepare build for mobile targets ([#4830](https://www.github.com/tauri-apps/tauri/pull/4830)) on 2022-08-02
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Enhance the dialog style on Windows via the manifest dependency `Microsoft.Windows.Common-Controls v6.0.0.0`.
  - [5c5c42ed](https://www.github.com/tauri-apps/tauri/commit/5c5c42edb64adf4250af6891d7d226fda7d4d783) feat(build): use modern dialog styles on Windows, closes [#4709](https://www.github.com/tauri-apps/tauri/pull/4709) ([#4840](https://www.github.com/tauri-apps/tauri/pull/4840)) on 2022-08-02
- Fix root of codegen output when using the `CodegenContext` API.
  - [ed581950](https://www.github.com/tauri-apps/tauri/commit/ed581950ea6fd0afee47aa73fb63083d2483429f) fix(tauri-build): use `::tauri` as root for the CodegenContext ([#4894](https://www.github.com/tauri-apps/tauri/pull/4894)) on 2022-08-08
- Return an error if a sidecar is configured with the same file name as the application.
  - [5cc1fd0f](https://www.github.com/tauri-apps/tauri/commit/5cc1fd0f7b01b257a461d4e867ddc8cd5072ff15) feat(tauri-build): validate sidecar name, closes [#4780](https://www.github.com/tauri-apps/tauri/pull/4780) closes [#4823](https://www.github.com/tauri-apps/tauri/pull/4823) ([#4814](https://www.github.com/tauri-apps/tauri/pull/4814)) on 2022-08-02
- Only rewrite temporary icon files when the content change, avoid needless rebuilds.
  - [f957cbb5](https://www.github.com/tauri-apps/tauri/commit/f957cbb56ccbd8d1c047a978b4579946252a6fd2) fix(codegen): write output file when contents change ([#4889](https://www.github.com/tauri-apps/tauri/pull/4889)) on 2022-08-09

## \[1.0.4]

- Reduce the amount of allocations when converting cases.
  - [bc370e32](https://www.github.com/tauri-apps/tauri/commit/bc370e326810446e15b1f50fb962b980114ba16b) feat: reduce the amount of `heck`-related allocations ([#4634](https://www.github.com/tauri-apps/tauri/pull/4634)) on 2022-07-11

## \[1.0.3]

- Improve configuration deserialization error messages.
  - [9170c920](https://www.github.com/tauri-apps/tauri/commit/9170c9207044fa561535f624916dfdbaa41ff79d) feat(core): improve config deserialization error messages ([#4607](https://www.github.com/tauri-apps/tauri/pull/4607)) on 2022-07-06
- The `TAURI_CONFIG` environment variable now represents the configuration to be merged instead of the entire JSON.
  - [fa028ebf](https://www.github.com/tauri-apps/tauri/commit/fa028ebf3c8ca7b43a70d283a01dbea86217594f) refactor: do not pass entire config from CLI to core, send patch instead ([#4598](https://www.github.com/tauri-apps/tauri/pull/4598)) on 2022-07-06

## \[1.0.2]

- Expose `platform::windows_version` function.
  - Bumped due to a bump in tauri-utils.
  - [bf764e83](https://www.github.com/tauri-apps/tauri/commit/bf764e83e01e7443e6cc54572001e1c98c357465) feat(utils): expose `windows_version` function ([#4534](https://www.github.com/tauri-apps/tauri/pull/4534)) on 2022-06-30

## \[1.0.1]

- Changed the `BundleConfig::targets` to a `BundleTarget` enum to enhance generated documentation.
  - Bumped due to a bump in tauri-utils.
  - [31c15cd2](https://www.github.com/tauri-apps/tauri/commit/31c15cd2bd94dbe39fb94982a15cbe02ac5d8925) docs(config): enhance documentation for bundle targets, closes [#3251](https://www.github.com/tauri-apps/tauri/pull/3251) ([#4418](https://www.github.com/tauri-apps/tauri/pull/4418)) on 2022-06-21
- Added `platform::is_windows_7`.
  - Bumped due to a bump in tauri-utils.
  - [57039fb2](https://www.github.com/tauri-apps/tauri/commit/57039fb2166571de85271b014a8711a29f06be1a) fix(core): add windows 7 notification support ([#4491](https://www.github.com/tauri-apps/tauri/pull/4491)) on 2022-06-28
- Suppress unused variable warning in release builds.
  - Bumped due to a bump in tauri-utils.
  - [45981851](https://www.github.com/tauri-apps/tauri/commit/45981851e35119266c1a079e1ff27a39f1fdfaed) chore(lint): unused variable warnings for release builds ([#4411](https://www.github.com/tauri-apps/tauri/pull/4411)) on 2022-06-22
- Added webview install mode options.
  - Bumped due to a bump in tauri-utils.
  - [2ca762d2](https://www.github.com/tauri-apps/tauri/commit/2ca762d207030a892a6d128b519e150e2d733468) feat(bundler): extend webview2 installation options, closes [#2882](https://www.github.com/tauri-apps/tauri/pull/2882) [#2452](https://www.github.com/tauri-apps/tauri/pull/2452) ([#4466](https://www.github.com/tauri-apps/tauri/pull/4466)) on 2022-06-26

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.15]

- Read the tray icon path relatively to the config directory.
  - Bumped due to a bump in tauri-codegen.
  - [562e8ca2](https://www.github.com/tauri-apps/tauri/commit/562e8ca23facf1a8e5fa6c8cdf872357d3523a78) fix(codegen): tray icon path is relative to the config directory on 2022-06-15

## \[1.0.0-rc.14]

- Do not copy the tray icon to the output directory on Linux since it is embedded in the binary.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.13]

- Copy `tauri.conf.json > tauri.bundle.windows.webview_fixed_runtime_path` as a resource to the target directory to fix development usage of a fixed Webview2 runtime path.
  - [8a634895](https://www.github.com/tauri-apps/tauri/commit/8a63489567b9fa86e404ad42b5b30c64361efe85) fix(build): fixed Webview2 runtime path in development, closes [#4308](https://www.github.com/tauri-apps/tauri/pull/4308) on 2022-06-10
- Improve usage of the GNU toolchain on Windows by copying the Webview2Loader.dll file to the target directory.
  - [58a6879b](https://www.github.com/tauri-apps/tauri/commit/58a6879b82e3a82027604cdd0913caacaaab5c76) feat(tauri-build): improve Windows GNU toolchain usage, closes [#4319](https://www.github.com/tauri-apps/tauri/pull/4319) ([#4323](https://www.github.com/tauri-apps/tauri/pull/4323)) on 2022-06-12
- Only statically link the VC runtime when the `STATIC_VCRUNTIME` environment variable is set to `true` (automatically done by the Tauri CLI).
  - [d703d27a](https://www.github.com/tauri-apps/tauri/commit/d703d27a707edc028f13b35603205da1133fcc2b) fix(build): statically link VC runtime only on `tauri build` ([#4292](https://www.github.com/tauri-apps/tauri/pull/4292)) on 2022-06-07

## \[1.0.0-rc.12]

- Statically link the Visual C++ runtime instead of using a merge module on the installer.
  - [bb061509](https://www.github.com/tauri-apps/tauri/commit/bb061509fb674bef86ecbc1de3aa8f3e367a9907) refactor(core): statically link vcruntime, closes [#4122](https://www.github.com/tauri-apps/tauri/pull/4122) ([#4227](https://www.github.com/tauri-apps/tauri/pull/4227)) on 2022-05-27

## \[1.0.0-rc.11]

- Create `dev` cfg alias.
  - [9cdcf9b3](https://www.github.com/tauri-apps/tauri/commit/9cdcf9b3a8fa27612b3156c1702a4e776627e869) feat(build): create `dev` alias ([#4212](https://www.github.com/tauri-apps/tauri/pull/4212)) on 2022-05-25

## \[1.0.0-rc.10]

- Delete existing sidecar before copying new one.
  - [a737f25c](https://www.github.com/tauri-apps/tauri/commit/a737f25c1078083e0b0f7f338f5c958b86914323) fix(tauri-build): delete existing sidecar file, closes [#4134](https://www.github.com/tauri-apps/tauri/pull/4134) ([#4167](https://www.github.com/tauri-apps/tauri/pull/4167)) on 2022-05-18

## \[1.0.0-rc.9]

- Search `tauri.conf.json > tauri > bundle > icons` for a `.ico` file for the window icon instead of simple default `icons/icon.ico` when `WindowsAttributes::window_icon_path` is not set.
  - [bad85a1f](https://www.github.com/tauri-apps/tauri/commit/bad85a1f11da03421401080531275ba201480cd1) feat(build): find .ico in config instead of default `icons/icon.ico` ([#4115](https://www.github.com/tauri-apps/tauri/pull/4115)) on 2022-05-13

## \[1.0.0-rc.8]

- Properly set file version information for the Windows executable.
  - [1ca2dd67](https://www.github.com/tauri-apps/tauri/commit/1ca2dd677d50b4c724c55b37060c3ba64b81c54a) fix(tauri-build): properly set executable version info on Windows ([#4045](https://www.github.com/tauri-apps/tauri/pull/4045)) on 2022-05-03

## \[1.0.0-rc.7]

- Rerun build script if `TAURI_CONFIG` environment variable change.
  - [7ae9e252](https://www.github.com/tauri-apps/tauri/commit/7ae9e25262376529637cd3b47b1cf84809efaec9) fix(tauri-build): rerun if `TAURI_CONFIG` env var changes on 2022-04-26

## \[1.0.0-rc.6]

- Copy system tray icon resource to the target directory on Linux.
  - [f2a30d8b](https://www.github.com/tauri-apps/tauri/commit/f2a30d8bc54fc3ba49e16f69a413eca5f61a9b1f) refactor(core): use ayatana appindicator by default, keep option to use gtk ([#3916](https://www.github.com/tauri-apps/tauri/pull/3916)) on 2022-04-19

## \[1.0.0-rc.5]

- Print error context on the `build` panic.
  - [49546c52](https://www.github.com/tauri-apps/tauri/commit/49546c5269080f38d57365788eb2592bff8f6d10) feat(build): print error context ([#3644](https://www.github.com/tauri-apps/tauri/pull/3644)) on 2022-03-09

## \[1.0.0-rc.4]

- Parse window icons at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [8c935872](https://www.github.com/tauri-apps/tauri/commit/8c9358725a17dcc2acaf4d10c3f654afdff586b0) refactor(core): move `png` and `ico` behind Cargo features ([#3588](https://www.github.com/tauri-apps/tauri/pull/3588)) on 2022-03-05

## \[1.0.0-rc.3]

- Automatically emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET` with the value set on `tauri.conf.json > tauri > bundle > macos > minimumSystemVersion`.
  - [4bacea5b](https://www.github.com/tauri-apps/tauri/commit/4bacea5bf48ea5ca6c9bdd10e28e85e67a0c6ef9) feat(core): set `MACOSX_DEPLOYMENT_TARGET` environment variable, closes [#2732](https://www.github.com/tauri-apps/tauri/pull/2732) ([#3496](https://www.github.com/tauri-apps/tauri/pull/3496)) on 2022-02-17

## \[1.0.0-rc.2]

- Rerun if sidecar or resource change.
  - [afcc3ec5](https://www.github.com/tauri-apps/tauri/commit/afcc3ec50148074293350aaa26a05812207716be) fix(build): rerun if resource or sidecar change ([#3460](https://www.github.com/tauri-apps/tauri/pull/3460)) on 2022-02-14

## \[1.0.0-rc.1]

- Remove `cargo:rerun-if-changed` check for non-existent file that caused projects to *always* rebuild.
  - [65287cd6](https://www.github.com/tauri-apps/tauri/commit/65287cd6148feeba91df86217b261770fed34608) remove non-existent cargo rerun check ([#3412](https://www.github.com/tauri-apps/tauri/pull/3412)) on 2022-02-11

## \[1.0.0-rc.0]

- Allow user to specify windows sdk path in build.rs.
  - [59b6ee87](https://www.github.com/tauri-apps/tauri/commit/59b6ee87932d341433032befe3babd897ed8f7d0) fix(tauri-build): allow user to specify win sdk path (fix: [#2871](https://www.github.com/tauri-apps/tauri/pull/2871)) ([#2893](https://www.github.com/tauri-apps/tauri/pull/2893)) on 2021-11-16
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
- Move the copying of resources and sidecars from `cli.rs` to `tauri-build` so using the Cargo CLI directly processes the files for the application execution in development.
  - [5eb72c24](https://www.github.com/tauri-apps/tauri/commit/5eb72c24deddf5a01093bea96b90c0d8806afc3f) refactor: copy resources and sidecars on the Cargo build script ([#3357](https://www.github.com/tauri-apps/tauri/pull/3357)) on 2022-02-08
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Validate `tauri` dependency `features` under `Cargo.toml` matching `tauri.conf.json`'s `allowlist`.
  - [4de285c3](https://www.github.com/tauri-apps/tauri/commit/4de285c3967d32250d73acdd5d171a6fd332d2b3) feat(core): validate Cargo features matching allowlist \[TRI-023] on 2022-01-09

## \[1.0.0-beta.4]

- Implement `Debug` on public API structs and enums.
  - [fa9341ba](https://www.github.com/tauri-apps/tauri/commit/fa9341ba18ba227735341530900714dba0f27291) feat(core): implement `Debug` on public API structs/enums, closes [#2292](https://www.github.com/tauri-apps/tauri/pull/2292) ([#2387](https://www.github.com/tauri-apps/tauri/pull/2387)) on 2021-08-11

## \[1.0.0-beta.3]

- Improve ESM detection with regexes.
  - Bumped due to a bump in tauri-codegen.
  - [4b0ec018](https://www.github.com/tauri-apps/tauri/commit/4b0ec0188078a8fefd4119fe5e19ebc30191f802) fix(core): improve JS ESM detection ([#2139](https://www.github.com/tauri-apps/tauri/pull/2139)) on 2021-07-02
- Inject invoke key on `script` tags with `type="module"`.
  - Bumped due to a bump in tauri-codegen.
  - [f03eea9c](https://www.github.com/tauri-apps/tauri/commit/f03eea9c9b964709532afbc4d1dd343b3fd96081) feat(core): inject invoke key on `<script type="module">` ([#2120](https://www.github.com/tauri-apps/tauri/pull/2120)) on 2021-06-29

## \[1.0.0-beta.2]

- Detect ESM scripts and inject the invoke key directly instead of using an IIFE.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27
- Improve invoke key code injection performance time rewriting code at compile time.
  - Bumped due to a bump in tauri-codegen.
  - [7765c7fa](https://www.github.com/tauri-apps/tauri/commit/7765c7fa281853ddfb26b6b17534df95eaede804) fix(core): invoke key injection on ES module, improve performance ([#2094](https://www.github.com/tauri-apps/tauri/pull/2094)) on 2021-06-27

## \[1.0.0-beta.1]

- Pull Windows resource information (`FileVersion`, `ProductVersion`, `ProductName` and `FileDescription`) from `tauri.conf.json > package` configuration.
  - [dc6b0d85](https://www.github.com/tauri-apps/tauri/commit/dc6b0d8522ca9f0962aa7c6fe446743889470b8c) feat(core): set .rc values from tauri.conf.json, closes [#1849](https://www.github.com/tauri-apps/tauri/pull/1849) ([#1951](https://www.github.com/tauri-apps/tauri/pull/1951)) on 2021-06-05

## \[1.0.0-beta.0]

- The `try_build` method now has a `Attributes` argument to allow specifying the window icon path used on Windows.
  - [c91105f](https://www.github.com/tauri-apps/tauri/commit/c91105ff965cceb2dfb402004c12799bf3b1c2f6) refactor(build): allow setting window icon path on `try_build` ([#1686](https://www.github.com/tauri-apps/tauri/pull/1686)) on 2021-05-03

## \[1.0.0-beta-rc.1]

- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - Bumped due to a bump in tauri-codegen.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.0]

- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
