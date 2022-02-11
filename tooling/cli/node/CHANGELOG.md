# Changelog

## \[1.0.0-rc.2]

- Fixes Tauri path resolution on projects without Git or a `.gitignore` file.
  - [d8acbe11](https://www.github.com/tauri-apps/tauri/commit/d8acbe11492bd990e6983c7e63e0f1a8f1ea5c7c) fix(cli.rs): app path resolution on projects without git, closes [#3409](https://www.github.com/tauri-apps/tauri/pull/3409) ([#3410](https://www.github.com/tauri-apps/tauri/pull/3410)) on 2022-02-11

## \[1.0.0-rc.1]

- Fix `init` command prompting for values even if the argument has been provided on the command line.
  - [def76840](https://www.github.com/tauri-apps/tauri/commit/def76840257a1447723ecda13c807cf0c881f083) fix(cli.rs): do not prompt for `init` values if arg set ([#3400](https://www.github.com/tauri-apps/tauri/pull/3400)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11
- Fixes CLI freezing when running `light.exe` on Windows without the `--verbose` flag.
  - [8beab636](https://www.github.com/tauri-apps/tauri/commit/8beab6363491e2a8757cc9fc0fa1eccc98ece916) fix(cli): build freezing on Windows, closes [#3399](https://www.github.com/tauri-apps/tauri/pull/3399) ([#3402](https://www.github.com/tauri-apps/tauri/pull/3402)) on 2022-02-11
- Respect `.gitignore` configuration when looking for the folder with the `tauri.conf.json` file.
  - [9c6c5a8c](https://www.github.com/tauri-apps/tauri/commit/9c6c5a8c52c6460d0b0a1a55300e1828262994ba) perf(cli.rs): improve performance of tauri dir lookup reading .gitignore ([#3405](https://www.github.com/tauri-apps/tauri/pull/3405)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11

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
