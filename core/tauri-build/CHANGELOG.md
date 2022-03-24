# Changelog

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
