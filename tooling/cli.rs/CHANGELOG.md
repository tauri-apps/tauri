# Changelog

## \[1.0.0-beta.5]

- Run powershell commands with `-NoProfile` flag
  - [3e6f3416](https://www.github.com/tauri-apps/tauri/commit/3e6f34160deab4f774d90aba28122e5b6b6f9db2) fix(cli.rs): run powershell kill command without profile ([#2130](https://www.github.com/tauri-apps/tauri/pull/2130)) on 2021-06-30
- Adds `release` argument to the `dev` command. Allowing to run the backend in release mode during development.
  - [7ee2dc8b](https://www.github.com/tauri-apps/tauri/commit/7ee2dc8b690703f509ab2d6ecdf9dafd6b72cd0b) feat(cli.rs): add release argument to the dev command ([#2192](https://www.github.com/tauri-apps/tauri/pull/2192)) on 2021-07-12
- Fixes `center` and `focus` not being allowed in `tauri.conf.json > tauri > windows` and ignored in `WindowBuilderWrapper`.
  - [bc2c331d](https://www.github.com/tauri-apps/tauri/commit/bc2c331dec3dec44c79e659b082b5fb6b65cc5ea) fix: center and focus not being allowed in config ([#2199](https://www.github.com/tauri-apps/tauri/pull/2199)) on 2021-07-12

## \[1.0.0-beta.4]

- Improve error message when the product name is invalid.
  - [1a41e9f0](https://www.github.com/tauri-apps/tauri/commit/1a41e9f040cfa18b6cc1380dfe21251d56e3f973) feat(cli.rs): improve error message on app rename, closes [#2101](https://www.github.com/tauri-apps/tauri/pull/2101) ([#2114](https://www.github.com/tauri-apps/tauri/pull/2114)) on 2021-06-28

## \[1.0.0-beta.3]

- Properly detect target platform's architecture.
  - [628a53eb](https://www.github.com/tauri-apps/tauri/commit/628a53eb6176f811d22d7730f08a99e5c370dbf4) fix(cli): properly detect target architecture, closes [#2040](https://www.github.com/tauri-apps/tauri/pull/2040) ([#2102](https://www.github.com/tauri-apps/tauri/pull/2102)) on 2021-06-28
- Fixes `build` command when the `target` arg is set.
  - [8e238701](https://www.github.com/tauri-apps/tauri/commit/8e2387018940e9e1421948d74a82156661ce2e4b) fix(cli.rs): fix out dir detection when target arg is set, closes [#2040](https://www.github.com/tauri-apps/tauri/pull/2040) ([#2098](https://www.github.com/tauri-apps/tauri/pull/2098)) on 2021-06-27

## \[1.0.0-beta.2]

- Support `cargo tauri build` on Apple M1 chip.
  - [3bf853d7](https://www.github.com/tauri-apps/tauri/commit/3bf853d782b491ad4965a1da25d19337eeac161f) feat(cli.rs): support tauri build on M1 chip ([#1915](https://www.github.com/tauri-apps/tauri/pull/1915)) on 2021-05-29
- Infer `app name` and `window title` from `package.json > productName` or `package.json > name`.
  Infer `distDir` and `devPath` by reading the package.json and trying to determine the UI framework (Vue.js, Angular, React, Svelte and some UI frameworks).
  - [21a971c3](https://www.github.com/tauri-apps/tauri/commit/21a971c3b76bf0c26d00b2520b4976fa526738f5) feat(cli.rs): infer devPath/distDir/appName from package.json ([#1930](https://www.github.com/tauri-apps/tauri/pull/1930)) on 2021-05-31
- Watch workspace crates on `dev` command.
  - [86a23ff3](https://www.github.com/tauri-apps/tauri/commit/86a23ff30b4f18effa39c87b7cae6b7e324d131c) added support for cargo workspaces for `dev` command ([#1827](https://www.github.com/tauri-apps/tauri/pull/1827)) on 2021-05-13
- Adds `features` argument to the `dev` and `build` commands.
  - [6ec8e84d](https://www.github.com/tauri-apps/tauri/commit/6ec8e84d9172c090ee1549db56c98c66f12436ff) feat(cli.rs): add `features` arg to dev/build ([#1828](https://www.github.com/tauri-apps/tauri/pull/1828)) on 2021-05-13
- Fixes the libwebkit2gtk package name.
  - [e08065d7](https://www.github.com/tauri-apps/tauri/commit/e08065d7fe8398b41180b3a64854ec8e71174d42) fix: deb installation error ([#1844](https://www.github.com/tauri-apps/tauri/pull/1844)) on 2021-05-18
- Properly keep all `tauri` features that are not managed by the CLI.
  - [17c7c439](https://www.github.com/tauri-apps/tauri/commit/17c7c4396ff2d5e13fc8726c2965b4e810fad6b9) refactor(core): use `attohttpc` by default ([#1861](https://www.github.com/tauri-apps/tauri/pull/1861)) on 2021-05-19
- Copy resources and binaries to `OUT_DIR` on `tauri dev` command.
  - [8f29a260](https://www.github.com/tauri-apps/tauri/commit/8f29a260e67aa111f6aeb262bd846a46d2858ce9) fix(cli.rs): copy resources and binaries on dev, closes [#1298](https://www.github.com/tauri-apps/tauri/pull/1298) ([#1946](https://www.github.com/tauri-apps/tauri/pull/1946)) on 2021-06-04
- Read cargo features from `tauri.conf.json > build > features` and propagate them on `dev` and `build`.
  - [2b814e9c](https://www.github.com/tauri-apps/tauri/commit/2b814e9c937489af0acb56051bd01c0d7fca2413) added cargo features to tauri config ([#1824](https://www.github.com/tauri-apps/tauri/pull/1824)) on 2021-05-13
- Fixes `tauri.conf.json > tauri > bundle > targets` not applying to the bundler.
  - [8be35ced](https://www.github.com/tauri-apps/tauri/commit/8be35ced78658de732360e3b20d7d70108c9b32d) fix(cli.rs): `tauri.conf.json > tauri > bundle > targets` being ignored ([#1945](https://www.github.com/tauri-apps/tauri/pull/1945)) on 2021-06-04
- Fixes `info` command not striping `\r` from child process version output.
  - [6a95d7ac](https://www.github.com/tauri-apps/tauri/commit/6a95d7acc378b40230bab18d00ea32de40a5818c) fix(cli.rs): `info` version checks not striping `\r` on Windows ([#1952](https://www.github.com/tauri-apps/tauri/pull/1952)) on 2021-06-05
- Allow setting a path to a license file for the Windows Installer (`tauri.conf.json > bundle > windows > wix > license`).
  - [b769c7f7](https://www.github.com/tauri-apps/tauri/commit/b769c7f7da4064b6133bf39a82127863d0d35531) feat(bundler): windows installer license, closes [#2009](https://www.github.com/tauri-apps/tauri/pull/2009) ([#2027](https://www.github.com/tauri-apps/tauri/pull/2027)) on 2021-06-21
- Change the `csp` value on the template to include `wss:` and `tauri:` to the `default-src` attribute.
  - [463fd00d](https://www.github.com/tauri-apps/tauri/commit/463fd00d06241c734994fe8e1882788dc30cc993) fix(csp): add wss and tauri to conf template ([#1974](https://www.github.com/tauri-apps/tauri/pull/1974)) on 2021-06-15
- Adds `tauri > bundle > windows > wix > language` config option. See https://docs.microsoft.com/en-us/windows/win32/msi/localizing-the-error-and-actiontext-tables.
  - [47919619](https://www.github.com/tauri-apps/tauri/commit/47919619815900fc3af47ec5873e31afb778b0ad) feat(bundler): allow setting wix language, closes [#1976](https://www.github.com/tauri-apps/tauri/pull/1976) ([#1988](https://www.github.com/tauri-apps/tauri/pull/1988)) on 2021-06-15

## \[1.0.0-beta.1]

- Add `'self'` to default CSP because otherwise no joy on macOS.
  - [12268e6](https://www.github.com/tauri-apps/tauri/commit/12268e6e69dc9a7034652f50316d3545cac687c7) fix(csp): add 'self' ([#1794](https://www.github.com/tauri-apps/tauri/pull/1794)) on 2021-05-12
- Fix a typo that would result in bundle arg being ignored.
  - [71f6a5e](https://www.github.com/tauri-apps/tauri/commit/71f6a5ed442a43bf1008043c95a1a90effdd2f81) fix(cli.rs/build): fix typo getting bundle arg ([#1783](https://www.github.com/tauri-apps/tauri/pull/1783)) on 2021-05-12

## \[1.0.0-beta.0]

- Fixes a cargo `target/` cache issue.
  - [79feb6a](https://www.github.com/tauri-apps/tauri/commit/79feb6a918c2b40af771b5dccc94c8f6f4176986) fix(cli.rs): cargo build failed due to cache issue, closes [#1543](https://www.github.com/tauri-apps/tauri/pull/1543) ([#1741](https://www.github.com/tauri-apps/tauri/pull/1741)) on 2021-05-07
- Improve error logging.
  - [5cc4b11](https://www.github.com/tauri-apps/tauri/commit/5cc4b11f5d00a1e7e580e31785b31c491c06d8d7) feat(cli.rs): add context to errors ([#1674](https://www.github.com/tauri-apps/tauri/pull/1674)) on 2021-05-01
- Adds Webview2 version on `info` command.
  - [2b4e2b7](https://www.github.com/tauri-apps/tauri/commit/2b4e2b7560515b76002d0c724bcca1f470ed106f) feat(cli.rs/info): get webview2 version on windows ([#1669](https://www.github.com/tauri-apps/tauri/pull/1669)) on 2021-05-04
- Adds `--runner [PROGRAM]` argument on the `dev` and `build` command, allowing using the specified program to run and build the application (example program: `cross`).
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Adds `--target [TARGET_TRIPLE]` option to the `build` command (example: `--target arm-unknown-linux-gnueabihf`).
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Rename `--target` option on the `build` command to `--bundle`.
  - [5c1fe52](https://www.github.com/tauri-apps/tauri/commit/5c1fe52c2bd74e2a8f6c99c2870af967e6309e8d) feat(cli.rs): allow using cross instead of cargo, add target triple arg ([#1664](https://www.github.com/tauri-apps/tauri/pull/1664)) on 2021-04-30
- Automatically add Tauri dependencies to the debian package `Depends` section.
  - [72b8048](https://www.github.com/tauri-apps/tauri/commit/72b8048b5ada7a18d71b0fd8a4a0177109b43db7) feat(cli.rs): fill debian `depends` with tauri dependencies ([#1767](https://www.github.com/tauri-apps/tauri/pull/1767)) on 2021-05-10
- Properly kill `beforeDevCommand` process.
  - [ac2cbcb](https://www.github.com/tauri-apps/tauri/commit/ac2cbcb131819e01074e1ed8fb6808260c56a027) fix(cli.rs): `before dev` process kill, closes [#1626](https://www.github.com/tauri-apps/tauri/pull/1626) ([#1700](https://www.github.com/tauri-apps/tauri/pull/1700)) on 2021-05-04
- Adds support to `tauri` dependency as string and table on `Cargo.toml`.
  - [df8bdcf](https://www.github.com/tauri-apps/tauri/commit/df8bdcf0631fd4e1e7035eb20a954574da96de66) feat(cli.rs): add support to string and table dependency, closes [#1653](https://www.github.com/tauri-apps/tauri/pull/1653) ([#1654](https://www.github.com/tauri-apps/tauri/pull/1654)) on 2021-04-29
- Show `framework` and `bundler` on the `info` command by reading the `package.json` file and matching known dependencies.
  - [152c755](https://www.github.com/tauri-apps/tauri/commit/152c755c4787b323ca3469c45934cc1e4d368cfa) feat(cli.rs): `framework` and `bundler` on info cmd, closes [#1681](https://www.github.com/tauri-apps/tauri/pull/1681) ([#1682](https://www.github.com/tauri-apps/tauri/pull/1682)) on 2021-05-02

## \[1.0.0-beta-rc.4]

- Fixes the Message `command` name value on plugin invoke handler.
  - Bumped due to a bump in tauri.
  - [422dd5e](https://www.github.com/tauri-apps/tauri/commit/422dd5e2a0a03bb1556915c78f110bfab092c874) fix(core): command name on plugin invoke handler ([#1577](https://www.github.com/tauri-apps/tauri/pull/1577)) on 2021-04-21
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25
- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - Bumped due to a bump in tauri.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.3]

- Check if distDir assets are built after running `beforeDevCommand`.
  - [a670d3a](https://www.github.com/tauri-apps/tauri/commit/a670d3a457bc0c0135b879c746d26a5f121c87a7) fix(cli.rs): check if distDir exists after running `beforeDevCommand` ([#1586](https://www.github.com/tauri-apps/tauri/pull/1586)) on 2021-04-22
- Fixes `tauri info` display version for the `@tauri-apps/api` package.
  - [0012782](https://www.github.com/tauri-apps/tauri/commit/0012782e43bd4e7e49528853c226b8e0e24b8794) fix(cli.rs): `info` command `npm_package_version` parsing `beta-rc` ([#1587](https://www.github.com/tauri-apps/tauri/pull/1587)) on 2021-04-22
- Fixes crash on usage of modifier keys on Windows when running `tauri init`.
  - [d623d95](https://www.github.com/tauri-apps/tauri/commit/d623d95fcb67736bc0862866b347c7102cde66aa) fix(cli.rs): inliner dialoguer & console until they publish, fixes [#1492](https://www.github.com/tauri-apps/tauri/pull/1492) ([#1610](https://www.github.com/tauri-apps/tauri/pull/1610)) on 2021-04-25
- Enable `tauri` `updater` feature when `tauri.conf.json > tauri > updater > active` is set to `true`.
  - [9490b25](https://www.github.com/tauri-apps/tauri/commit/9490b257d2564840eb0c9167340bf444bca84699) fix(cli.rs): enable the `updater` feature on cli ([#1597](https://www.github.com/tauri-apps/tauri/pull/1597)) on 2021-04-23

## \[1.0.0-beta-rc.2]

- Add missing camelcase rename for config
  - [bdf7072](https://www.github.com/tauri-apps/tauri/commit/bdf707285e3d307ab083009c274ccb56d5053ff2) fix(cli.rs/info): add missing camelCase rename ([#1505](https://www.github.com/tauri-apps/tauri/pull/1505)) on 2021-04-14
- Fix `tauri info`
- Properly detect `yarn` and `npm` versions on windows.
- Fix a panic caused by a wrong field name in `metadata.json`
- [71666e9](https://www.github.com/tauri-apps/tauri/commit/71666e9f9cfb5499a727b3f95182e89073f67d7b) fix(cli.rs): fix panic & use `cmd` to run `yarn`&`npm` on windows ([#1511](https://www.github.com/tauri-apps/tauri/pull/1511)) on 2021-04-17
- Sync `metadata.json` via script to update version reference to cli.js, tauri (core) and tauri-build.
  - [1f64927](https://www.github.com/tauri-apps/tauri/commit/1f64927362ef20761d7cd3591281519eb292aa33) chore: sync cli.rs metadata.json file versions ([#1534](https://www.github.com/tauri-apps/tauri/pull/1534)) on 2021-04-19

## \[1.0.0-beta-rc.1]

- Missing the `files` property in the package.json which mean that the `dist` directory was not published and used.
  - Bumped due to a bump in api.
  - [b2569a7](https://www.github.com/tauri-apps/tauri/commit/b2569a729a3caa88bdba62abc31f0665e1323aaa) fix(js-api): dist ([#1498](https://www.github.com/tauri-apps/tauri/pull/1498)) on 2021-04-15

## \[1.0.0-beta-rc.0]

- You can now run `cargo tauri build -t none` to speed up the build if you don't need executables.
  - [4d507f9](https://www.github.com/tauri-apps/tauri/commit/4d507f9adfb26819f9d6406b191fdaa6188145f4) feat(cli/core): add support for building without targets ([#1203](https://www.github.com/tauri-apps/tauri/pull/1203)) on 2021-02-10
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `dev` and `build` pipeline is now written in Rust.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Run `beforeDevCommand` and `beforeBuildCommand` in a shell.
  - [32eb0d5](https://www.github.com/tauri-apps/tauri/commit/32eb0d562b135d8df19c78ff22aa53c73f459c76) feat(cli): run beforeDev and beforeBuild in a shell, closes [#1295](https://www.github.com/tauri-apps/tauri/pull/1295) ([#1399](https://www.github.com/tauri-apps/tauri/pull/1399)) on 2021-03-28
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Fixes `<a target="_blank">` polyfill.
  - [4ee044a](https://www.github.com/tauri-apps/tauri/commit/4ee044a3e662a0ac2be98f7e1286088d721c3307) fix(cli): use correct arg in `_blanks` links polyfill ([#1362](https://www.github.com/tauri-apps/tauri/pull/1362)) on 2021-03-17
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Adds `productName` and `version` configs on `tauri.conf.json > package`.
  - [5b3d9b2](https://www.github.com/tauri-apps/tauri/commit/5b3d9b2c07da766f81981ba7c4961cd354d51340) feat(config): allow setting product name and version on tauri.conf.json ([#1358](https://www.github.com/tauri-apps/tauri/pull/1358)) on 2021-03-22
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `info` command was rewritten in Rust.
  - [c3e06ee](https://www.github.com/tauri-apps/tauri/commit/c3e06ee9e88b3631da6eeb17d61ddd41cd5c6fe9) refactor(cli): rewrite info in Rust ([#1389](https://www.github.com/tauri-apps/tauri/pull/1389)) on 2021-03-25
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- The `init` command was rewritten in Rust.
  - [f72b93b](https://www.github.com/tauri-apps/tauri/commit/f72b93b676ba8c48fd9273c187de3dbbc410fa0f) refactor(cli): rewrite init command in Rust ([#1382](https://www.github.com/tauri-apps/tauri/pull/1382)) on 2021-03-24
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- All the arguments passed after `tauri dev --` are now propagated to the binary.
  - [4e9d31c](https://www.github.com/tauri-apps/tauri/commit/4e9d31c70ba13f1cabe830c6519a1b5f4789fd7b) feat(cli): propagate args passed after `dev --`, closes [#1406](https://www.github.com/tauri-apps/tauri/pull/1406) ([#1407](https://www.github.com/tauri-apps/tauri/pull/1407)) on 2021-03-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
