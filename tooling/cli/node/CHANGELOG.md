# Changelog

## \[1.0.0-rc.1]

- Include `vswhere.exe` on the published package.
  - Bumped due to a bump in cli.rs.
  - [3227502e](https://www.github.com/tauri-apps/tauri/commit/3227502e8c9f137e5783cba2e0c692473cc5456d) fix(cli.rs): package `vswhere.exe` on 2022-02-10

## \[1.0.0-rc.0]

- Do not force Tauri application code on `src-tauri` folder and use a glob pattern to look for a subfolder with a `tauri.conf.json` file.
  - [a8cff6b3](https://www.github.com/tauri-apps/tauri/commit/a8cff6b3bc3288a53d7cdc5b3cb95d371309d2d6) feat(cli): do not enforce `src-tauri` folder structure, closes [#2643](https://www.github.com/tauri-apps/tauri/pull/2643) ([#2654](https://www.github.com/tauri-apps/tauri/pull/2654)) on 2021-09-27
- Added CommonJS output to the `dist` folder.
  - [205b0dc8](https://www.github.com/tauri-apps/tauri/commit/205b0dc8f30bf70902979a2c0a08c8bc8c8e5360) feat(cli.js): add CommonJS dist files ([#2646](https://www.github.com/tauri-apps/tauri/pull/2646)) on 2021-09-23
- Fixes `.ico` icon generation.
  - [11db96e4](https://www.github.com/tauri-apps/tauri/commit/11db96e440e6cadc1c70992d07bfea3c448208b1) fix(cli.js): `.ico` icon generation, closes [#2692](https://www.github.com/tauri-apps/tauri/pull/2692) ([#2694](https://www.github.com/tauri-apps/tauri/pull/2694)) on 2021-10-02
- Automatically unplug `@tauri-apps/cli` in yarn 2+ installations to fix the download of the rust-cli.
  - [1e336b68](https://www.github.com/tauri-apps/tauri/commit/1e336b6872c3b78caf7c2c6e71e03016c6abdacf) fix(cli.js): Fix package installation on yarn 2+ ([#3012](https://www.github.com/tauri-apps/tauri/pull/3012)) on 2021-12-09
- Read `package.json` and check for a `tauri` object containing the `appPath` string, which points to the tauri crate path.
  - [fb2b9a52](https://www.github.com/tauri-apps/tauri/commit/fb2b9a52f594830c0a68ea40ea429a09892f7ba7) feat(cli.js): allow configuring tauri app path on package.json [#2752](https://www.github.com/tauri-apps/tauri/pull/2752) ([#3035](https://www.github.com/tauri-apps/tauri/pull/3035)) on 2021-12-09
- Removed the `icon` command, now exposed as a separate package, see https://github.com/tauri-apps/tauricon.
  - [58030172](https://www.github.com/tauri-apps/tauri/commit/58030172eddb2403a84b56a21b5bdcebca42c265) feat(tauricon): remove from cli ([#3293](https://www.github.com/tauri-apps/tauri/pull/3293)) on 2022-02-07
