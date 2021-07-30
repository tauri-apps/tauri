# Changelog

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
