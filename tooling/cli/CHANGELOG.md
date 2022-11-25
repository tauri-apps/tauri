# Changelog

## \[1.2.1]

- Fixes injection of Cargo features defined in the configuration file.
  - [1ecaeb29](https://www.github.com/tauri-apps/tauri/commit/1ecaeb29aa798f591f6488dc6c3a7a8d22f6073e) fix(cli): inject config feature flags when features arg is not provided on 2022-11-18

## \[1.2.0]

- Keep `tauri dev` watcher alive when the configuration is invalid.
  - [cc186c7a](https://www.github.com/tauri-apps/tauri/commit/cc186c7a0eab1c364f8b58101f86979ae4ed3d03) fix(cli): keep dev watcher alive if config is incorrect, closes [#5173](https://www.github.com/tauri-apps/tauri/pull/5173) ([#5495](https://www.github.com/tauri-apps/tauri/pull/5495)) on 2022-10-28
- Ignore workspace members in dev watcher if they are ignored by `.taurignore`
  - [9417ce40](https://www.github.com/tauri-apps/tauri/commit/9417ce401c4985e97245ce02d3b7cc31fb4bf59e) fix(cli): apply `.taurignore` rules to workspace members, closes [#5355](https://www.github.com/tauri-apps/tauri/pull/5355) ([#5460](https://www.github.com/tauri-apps/tauri/pull/5460)) on 2022-10-28
- Detect JSON5 and TOML configuration files in the dev watcher.
  - [e7ccbd85](https://www.github.com/tauri-apps/tauri/commit/e7ccbd8573f6b9124e80c0b67fa2365729c3c196) feat(cli): detect JSON5 and TOML configuration files in the dev watcher ([#5439](https://www.github.com/tauri-apps/tauri/pull/5439)) on 2022-10-19
- Fix cli passing `--no-default-features` to the app instead of the runner (Cargo).
  - [a3a70218](https://www.github.com/tauri-apps/tauri/commit/a3a70218f3cc438b4875a046a182ca44dab357ae) fix(cli): pass `--no-default-features` to runner instead of app, closes [#5415](https://www.github.com/tauri-apps/tauri/pull/5415) ([#5474](https://www.github.com/tauri-apps/tauri/pull/5474)) on 2022-10-25
- Validate `package > productName` in the tauri config and produce errors if it contains one of the following characters `/\:*?\"<>|`
  - [b9316a64](https://www.github.com/tauri-apps/tauri/commit/b9316a64eaa9348c79efafb8b94960d9b4d5b27a) fix(cli): validate `productName` in config, closes [#5233](https://www.github.com/tauri-apps/tauri/pull/5233) ([#5262](https://www.github.com/tauri-apps/tauri/pull/5262)) on 2022-09-28
- Hot-reload the frontend when `tauri.conf.json > build > devPath` points to a directory.
  - [54c337e0](https://www.github.com/tauri-apps/tauri/commit/54c337e06f3bc624c4780cf002bc54790f446c90) feat(cli): hotreload support for frontend static files, closes [#2173](https://www.github.com/tauri-apps/tauri/pull/2173) ([#5256](https://www.github.com/tauri-apps/tauri/pull/5256)) on 2022-09-28
- Expose `TAURI_TARGET_TRIPLE` to `beforeDevCommand`, `beforeBuildCommand` and `beforeBundleCommand`
  - [a4aec9f0](https://www.github.com/tauri-apps/tauri/commit/a4aec9f0a864ec6d7712db8bd50989d9c2e2fd2e) feat(cli): expose `TAURI_TARGET_TRIPLE` to before\*Commands, closes [#5091](https://www.github.com/tauri-apps/tauri/pull/5091) ([#5101](https://www.github.com/tauri-apps/tauri/pull/5101)) on 2022-10-03
- Log dev watcher file change detection.
  - [9076d5d2](https://www.github.com/tauri-apps/tauri/commit/9076d5d2e76d432aef475ba403e9ab5bd3b9d2b0) feat(cli): add prompt information when file changing detected, closes [#5417](https://www.github.com/tauri-apps/tauri/pull/5417) ([#5428](https://www.github.com/tauri-apps/tauri/pull/5428)) on 2022-10-19
- Set `TAURI_PLATFORM_TYPE`, `TAURI_FAMILY`, `TAURI_ARCH` and `TAURI_PLATFORM` env vars for hook commands to based on the app not the cli.
  - [a4aec9f0](https://www.github.com/tauri-apps/tauri/commit/a4aec9f0a864ec6d7712db8bd50989d9c2e2fd2e) feat(cli): expose `TAURI_TARGET_TRIPLE` to before\*Commands, closes [#5091](https://www.github.com/tauri-apps/tauri/pull/5091) ([#5101](https://www.github.com/tauri-apps/tauri/pull/5101)) on 2022-10-03
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Add `tauri.conf.json > bundle > publisher` field to specify the app publisher.
  - [628285c1](https://www.github.com/tauri-apps/tauri/commit/628285c1cf43f03ed62378f3b6cc0c991317526f) feat(bundler): add `publisher` field, closes [#5273](https://www.github.com/tauri-apps/tauri/pull/5273) ([#5283](https://www.github.com/tauri-apps/tauri/pull/5283)) on 2022-09-28
- Changed the project template to not enable all APIs by default.
  - [582c25a0](https://www.github.com/tauri-apps/tauri/commit/582c25a0f0fa2725d786ec4edd0defe7811ad6e8) refactor(cli): disable api-all on templates ([#5538](https://www.github.com/tauri-apps/tauri/pull/5538)) on 2022-11-03

## \[1.1.1]

- Fix wrong cli metadata that caused new projects (created through `tauri init`) fail to build
  - [db26aaf2](https://www.github.com/tauri-apps/tauri/commit/db26aaf2b44ce5335c9223c571ef2b2175e0cd6d) fix: fix wrong cli metadata ([#5214](https://www.github.com/tauri-apps/tauri/pull/5214)) on 2022-09-16

## \[1.1.0]

- Allow adding `build > beforeBundleCommand` in tauri.conf.json to run a shell command before the bundling phase.
  - [57ab9847](https://www.github.com/tauri-apps/tauri/commit/57ab9847eb2d8c9a5da584b873b7c072e9ee26bf) feat(cli): add `beforeBundleCommand`, closes [#4879](https://www.github.com/tauri-apps/tauri/pull/4879) ([#4893](https://www.github.com/tauri-apps/tauri/pull/4893)) on 2022-08-09
- Change `before_dev_command` and `before_build_command` config value to allow configuring the current working directory.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Allow configuring the `before_dev_command` to force the CLI to wait for the command to finish before proceeding.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Check if the default build target is set in the Cargo configuration.
  - [436f3d8d](https://www.github.com/tauri-apps/tauri/commit/436f3d8d66727f5b64165522f0b55f4ab54bd1ae) feat(cli): load Cargo configuration to check default build target ([#4990](https://www.github.com/tauri-apps/tauri/pull/4990)) on 2022-08-21
- Add support to cargo-binstall.
  - [90d5929f](https://www.github.com/tauri-apps/tauri/commit/90d5929fea6df575a2aa3c0a749374358f1ddb9b) feat(cli.rs): add support to cargo-binstall, closes [#4651](https://www.github.com/tauri-apps/tauri/pull/4651) ([#4817](https://www.github.com/tauri-apps/tauri/pull/4817)) on 2022-08-02
- Use `cargo metadata` to detect the workspace root and target directory.
  - [fea70eff](https://www.github.com/tauri-apps/tauri/commit/fea70effad219c0794d919f8834fa1a1ffd204c7) refactor(cli): Use `cargo metadata` to detect the workspace root and target directory, closes [#4632](https://www.github.com/tauri-apps/tauri/pull/4632), [#4928](https://www.github.com/tauri-apps/tauri/pull/4928). ([#4932](https://www.github.com/tauri-apps/tauri/pull/4932)) on 2022-08-21
- Prompt for `beforeDevCommand` and `beforeBuildCommand` in `tauri init`.
  - [6d4945c9](https://www.github.com/tauri-apps/tauri/commit/6d4945c9f06cd1f7018e1c48686ba682aae817df) feat(cli): prompt for before\*Command, closes [#4691](https://www.github.com/tauri-apps/tauri/pull/4691) ([#4721](https://www.github.com/tauri-apps/tauri/pull/4721)) on 2022-07-25
- Add `icon` command to generate icons.
  - [12e9d811](https://www.github.com/tauri-apps/tauri/commit/12e9d811e69813e7e9db5344b422101ddc09589f) feat(cli): Add `icon` command (tauricon) ([#4992](https://www.github.com/tauri-apps/tauri/pull/4992)) on 2022-09-03
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Automatically use any `.taurignore` file as ignore rules for dev watcher and app path finder.
  - [596fa08d](https://www.github.com/tauri-apps/tauri/commit/596fa08d48e371c7bd29e1ef799119ac8fca0d0b) feat(cli): automatically use `.taurignore`, ref [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4623](https://www.github.com/tauri-apps/tauri/pull/4623)) on 2022-07-28
- Enable WiX FIPS compliance when the `TAURI_FIPS_COMPLIANT` environment variable is set to `true`.
  - [d88b9de7](https://www.github.com/tauri-apps/tauri/commit/d88b9de7aaeaaa2e42e4795dbc2b8642b5ae7a50) feat(core): add `fips_compliant` wix config option, closes [#4541](https://www.github.com/tauri-apps/tauri/pull/4541) ([#4843](https://www.github.com/tauri-apps/tauri/pull/4843)) on 2022-08-04
- Fixes dev watcher incorrectly exiting the CLI when sequential file updates are detected.
  - [47fab680](https://www.github.com/tauri-apps/tauri/commit/47fab6809a1e23b3b9a84695e2d91ff0826ba79a) fix(cli): dev watcher incorrectly killing process on multiple file write ([#4684](https://www.github.com/tauri-apps/tauri/pull/4684)) on 2022-07-25
- Set the `MACOSX_DEPLOYMENT_TARGET` environment variable with the configuration `minimum_system_version` value.
  - [fa23310f](https://www.github.com/tauri-apps/tauri/commit/fa23310f23cb9e6a02ec2524f1ef394a5b42990e) fix(cli): set MACOSX_DEPLOYMENT_TARGET env var, closes [#4704](https://www.github.com/tauri-apps/tauri/pull/4704) ([#4842](https://www.github.com/tauri-apps/tauri/pull/4842)) on 2022-08-02
- Added `--no-watch` argument to the `dev` command to disable the file watcher.
  - [0983d7ce](https://www.github.com/tauri-apps/tauri/commit/0983d7ce7f24ab43f9ae7b5e1177ff244d8885a8) feat(cli): add `--no-watch` argument to the dev command, closes [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4793](https://www.github.com/tauri-apps/tauri/pull/4793)) on 2022-07-29
- Validate updater signature matches configured public key.
  - [b2a8930b](https://www.github.com/tauri-apps/tauri/commit/b2a8930b3c4b72c50ce72e73575f42c9cbe91bad) feat(cli): validate updater private key when signing ([#4754](https://www.github.com/tauri-apps/tauri/pull/4754)) on 2022-07-25

## \[1.0.5]

- Correctly fill the architecture when building Debian packages targeting ARM64 (aarch64).
  - Bumped due to a bump in tauri-bundler.
  - [635f23b8](https://www.github.com/tauri-apps/tauri/commit/635f23b88adbb8726d628f67840709cd870836dc) fix(bundler): correctly set debian architecture for aarch64 ([#4700](https://www.github.com/tauri-apps/tauri/pull/4700)) on 2022-07-17

## \[1.0.4]

- Do not capture and force colors of `cargo build` output.
  - [c635a0da](https://www.github.com/tauri-apps/tauri/commit/c635a0dad437860d54109adffaf245b7c21bc684) refactor(cli): do not capture and force colors of cargo build output ([#4627](https://www.github.com/tauri-apps/tauri/pull/4627)) on 2022-07-12
- Reduce the amount of allocations when converting cases.
  - [bc370e32](https://www.github.com/tauri-apps/tauri/commit/bc370e326810446e15b1f50fb962b980114ba16b) feat: reduce the amount of `heck`-related allocations ([#4634](https://www.github.com/tauri-apps/tauri/pull/4634)) on 2022-07-11

## \[1.0.3]

- Changed the app template to not set the default app menu as it is now set automatically on macOS which is the platform that needs a menu to function properly.
  - [91055883](https://www.github.com/tauri-apps/tauri/commit/9105588373cc8401bd9ad79bdef26f509b2d76b7) feat: add implicit default menu for macOS only, closes [#4551](https://www.github.com/tauri-apps/tauri/pull/4551) ([#4570](https://www.github.com/tauri-apps/tauri/pull/4570)) on 2022-07-04
- Improved bundle identifier validation showing the exact source of the configuration value.
  - [8e3e7fc6](https://www.github.com/tauri-apps/tauri/commit/8e3e7fc64641afc7a6833bc93205e6f525562545) feat(cli): improve bundle identifier validation, closes [#4589](https://www.github.com/tauri-apps/tauri/pull/4589) ([#4596](https://www.github.com/tauri-apps/tauri/pull/4596)) on 2022-07-05
- Improve configuration deserialization error messages.
  - [9170c920](https://www.github.com/tauri-apps/tauri/commit/9170c9207044fa561535f624916dfdbaa41ff79d) feat(core): improve config deserialization error messages ([#4607](https://www.github.com/tauri-apps/tauri/pull/4607)) on 2022-07-06
- Skip the static link of the `vcruntime140.dll` if the `STATIC_VCRUNTIME` environment variable is set to `false`.
  - [2e61abaa](https://www.github.com/tauri-apps/tauri/commit/2e61abaa9ae5d7a41ca1fa6505b5d6c368625ce5) feat(cli): allow dynamic link vcruntime, closes [#4565](https://www.github.com/tauri-apps/tauri/pull/4565) ([#4601](https://www.github.com/tauri-apps/tauri/pull/4601)) on 2022-07-06
- The `TAURI_CONFIG` environment variable now represents the configuration to be merged instead of the entire JSON.
  - [fa028ebf](https://www.github.com/tauri-apps/tauri/commit/fa028ebf3c8ca7b43a70d283a01dbea86217594f) refactor: do not pass entire config from CLI to core, send patch instead ([#4598](https://www.github.com/tauri-apps/tauri/pull/4598)) on 2022-07-06
- Watch for Cargo workspace members in the `dev` file watcher.
  - [dbb8c87b](https://www.github.com/tauri-apps/tauri/commit/dbb8c87b96dec9942b1bf877b29bafb8246514d4) feat(cli): watch Cargo workspaces in the dev command, closes [#4222](https://www.github.com/tauri-apps/tauri/pull/4222) ([#4572](https://www.github.com/tauri-apps/tauri/pull/4572)) on 2022-07-03

## \[1.0.2]

- Fixes a crash on the `signer sign` command.
  - [8e808fec](https://www.github.com/tauri-apps/tauri/commit/8e808fece95f2e506acf2c446d37b9913fd67d50) fix(cli.rs): conflicts_with arg doesn't exist closes  ([#4538](https://www.github.com/tauri-apps/tauri/pull/4538)) on 2022-06-30

## \[1.0.1]

- No longer adds the `pkg-config` dependency to `.deb` packages when the `systemTray` is used.
  This only works with recent versions of `libappindicator-sys` (including https://github.com/tauri-apps/libappindicator-rs/pull/38),
  so a `cargo update` may be necessary if you create `.deb` bundles and use the tray feature.
  - [0e6edeb1](https://www.github.com/tauri-apps/tauri/commit/0e6edeb14f379af1e02a7cebb4e3a5c9e87ebf7f) fix(cli): Don't add `pkg-config` to `deb` ([#4508](https://www.github.com/tauri-apps/tauri/pull/4508)) on 2022-06-29
- AppImage bundling will now prefer bundling correctly named appindicator library (including `.1` version suffix). With a symlink for compatibility with the old naming.
  - [bf45ca1d](https://www.github.com/tauri-apps/tauri/commit/bf45ca1df6691c05bdf72c5716cc01e89a7791d4) fix(cli,bundler): prefer AppImage libraries with ABI version ([#4505](https://www.github.com/tauri-apps/tauri/pull/4505)) on 2022-06-29
- Improve error message when `cargo` is not installed.
  - [e0e5f772](https://www.github.com/tauri-apps/tauri/commit/e0e5f772430f6349ec99ba891e601331e376e3c7) feat(cli): improve `cargo not found` error message, closes [#4428](https://www.github.com/tauri-apps/tauri/pull/4428) ([#4430](https://www.github.com/tauri-apps/tauri/pull/4430)) on 2022-06-21
- The app template now only sets the default menu on macOS.
  - [5105b428](https://www.github.com/tauri-apps/tauri/commit/5105b428c4726b2179cd4b3244350d1a1ee73734) feat(cli): change app template to only set default menu on macOS ([#4518](https://www.github.com/tauri-apps/tauri/pull/4518)) on 2022-06-29
- Warn if updater is enabled but not in the bundle target list.
  - [31c15cd2](https://www.github.com/tauri-apps/tauri/commit/31c15cd2bd94dbe39fb94982a15cbe02ac5d8925) docs(config): enhance documentation for bundle targets, closes [#3251](https://www.github.com/tauri-apps/tauri/pull/3251) ([#4418](https://www.github.com/tauri-apps/tauri/pull/4418)) on 2022-06-21
- Check if target exists and is installed on dev and build commands.
  - [13b8a240](https://www.github.com/tauri-apps/tauri/commit/13b8a2403d1353a8c3a643fbc6b6e862af68376e) feat(cli): validate target argument ([#4458](https://www.github.com/tauri-apps/tauri/pull/4458)) on 2022-06-24
- Fixes the covector configuration on the plugin templates.
  - [b8a64d01](https://www.github.com/tauri-apps/tauri/commit/b8a64d01bab11f955b7bbdf323d0afa1a3db4b64) fix(cli): add prepublish scripts to the plugin templates on 2022-06-19
- Set the binary name to the product name in development.
  - [b025b9f5](https://www.github.com/tauri-apps/tauri/commit/b025b9f581ac1a6ae0a26789c2be1e9928fb0282) refactor(cli): set binary name on dev ([#4447](https://www.github.com/tauri-apps/tauri/pull/4447)) on 2022-06-23
- Allow registering a `.gitignore` file to skip watching some project files and directories via the `TAURI_DEV_WATCHER_IGNORE_FILE` environment variable.
  - [83186dd8](https://www.github.com/tauri-apps/tauri/commit/83186dd89768407984db35fb67c3cc51f50ea8f5) Read extra ignore file for dev watcher, closes [#4406](https://www.github.com/tauri-apps/tauri/pull/4406) ([#4409](https://www.github.com/tauri-apps/tauri/pull/4409)) on 2022-06-20
- Fix shebang for `kill-children.sh`.
  - [35dd51db](https://www.github.com/tauri-apps/tauri/commit/35dd51db6826ec1eed7b90082b9eb6b2a699b627) fix(cli): add shebang for kill-children.sh, closes [#4262](https://www.github.com/tauri-apps/tauri/pull/4262) ([#4416](https://www.github.com/tauri-apps/tauri/pull/4416)) on 2022-06-22
- Update plugin templates to use newer `tauri-apps/create-pull-request` GitHub action.
  - [07f90795](https://www.github.com/tauri-apps/tauri/commit/07f9079532a42f3517d96faeaf46cad6176b31ac) chore(cli): update plugin template tauri-apps/create-pull-request on 2022-06-19
- Use UNIX path separator on the init `$schema` field.
  - [01053045](https://www.github.com/tauri-apps/tauri/commit/010530459ef62c48eed68ca965f2688accabcf69) chore(cli): use unix path separator on $schema ([#4384](https://www.github.com/tauri-apps/tauri/pull/4384)) on 2022-06-19
- The `info` command now can check the Cargo lockfile on workspaces.
  - [12f65219](https://www.github.com/tauri-apps/tauri/commit/12f65219ea75a51ebd38659ddce1563e015a036c) fix(cli): read lockfile from workspace on the info command, closes [#4232](https://www.github.com/tauri-apps/tauri/pull/4232) ([#4423](https://www.github.com/tauri-apps/tauri/pull/4423)) on 2022-06-21
- Preserve the `Cargo.toml` formatting when the features array is not changed.
  - [6650e5d6](https://www.github.com/tauri-apps/tauri/commit/6650e5d6720c215530ca1fdccd19bd2948dd6ca3) fix(cli): preserve Cargo manifest formatting when possible ([#4431](https://www.github.com/tauri-apps/tauri/pull/4431)) on 2022-06-21
- Change the updater signature metadata to include the file name instead of its full path.
  - [094b3eb3](https://www.github.com/tauri-apps/tauri/commit/094b3eb352bcf5de28414015e7c44290d619ea8c) fix(cli): file name instead of path on updater sig comment, closes [#4467](https://www.github.com/tauri-apps/tauri/pull/4467) ([#4484](https://www.github.com/tauri-apps/tauri/pull/4484)) on 2022-06-27
- Validate bundle identifier as it must only contain alphanumeric characters, hyphens and periods.
  - [0674a801](https://www.github.com/tauri-apps/tauri/commit/0674a80129d7c31bc93257849afc0a5069129fed) fix: assert config.bundle.identifier to be only alphanumeric, hyphens or dots. closes [#4359](https://www.github.com/tauri-apps/tauri/pull/4359) ([#4363](https://www.github.com/tauri-apps/tauri/pull/4363)) on 2022-06-17

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.16]

- Use the default window menu in the app template.
  - [4c4acc30](https://www.github.com/tauri-apps/tauri/commit/4c4acc3094218dd9cee0f1ad61810c979e0b41fa) feat: implement `Default` for `Menu`, closes [#2398](https://www.github.com/tauri-apps/tauri/pull/2398) ([#4291](https://www.github.com/tauri-apps/tauri/pull/4291)) on 2022-06-15

## \[1.0.0-rc.15]

- Removed the tray icon from the Debian and AppImage bundles since they are embedded in the binary now.
  - [4ce8e228](https://www.github.com/tauri-apps/tauri/commit/4ce8e228134cd3f22973b74ef26ca0d165fbbbd9) refactor(core): use `Icon` for tray icons ([#4342](https://www.github.com/tauri-apps/tauri/pull/4342)) on 2022-06-14

## \[1.0.0-rc.14]

- Set the `TRAY_LIBRARY_PATH` environment variable to make the bundle copy the appindicator library to the AppImage.
  - [34552444](https://www.github.com/tauri-apps/tauri/commit/3455244436578003a5fbb447b039e5c8971152ec) feat(cli): bundle appindicator library in the AppImage, closes [#3859](https://www.github.com/tauri-apps/tauri/pull/3859) ([#4267](https://www.github.com/tauri-apps/tauri/pull/4267)) on 2022-06-07
- Set the `APPIMAGE_BUNDLE_GSTREAMER` environment variable to make the bundler copy additional gstreamer files to the AppImage.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10
- Configure the AppImage bundler to copy the `/usr/bin/xdg-open` binary if it exists and the shell `open` API is enabled.
  - [2322ac11](https://www.github.com/tauri-apps/tauri/commit/2322ac11cf6290c6bf65413048a049c8072f863b) fix(bundler): bundle `/usr/bin/xdg-open` in appimage if open API enabled ([#4265](https://www.github.com/tauri-apps/tauri/pull/4265)) on 2022-06-04
- Fixes multiple occurrences handling of the `bundles` and `features` arguments.
  - [f685df39](https://www.github.com/tauri-apps/tauri/commit/f685df399a5a05480b6e4f5d92da71f3b87895ef) fix(cli): parsing of arguments with multiple values, closes [#4231](https://www.github.com/tauri-apps/tauri/pull/4231) ([#4233](https://www.github.com/tauri-apps/tauri/pull/4233)) on 2022-05-29
- Log command output in real time instead of waiting for it to finish.
  - [76d1eaae](https://www.github.com/tauri-apps/tauri/commit/76d1eaaebda5c8f6b0d41bf6587945e98cd441f3) feat(cli): debug command output in real time ([#4318](https://www.github.com/tauri-apps/tauri/pull/4318)) on 2022-06-12
- Configure the `STATIC_VCRUNTIME` environment variable so `tauri-build` statically links it on the build command.
  - [d703d27a](https://www.github.com/tauri-apps/tauri/commit/d703d27a707edc028f13b35603205da1133fcc2b) fix(build): statically link VC runtime only on `tauri build` ([#4292](https://www.github.com/tauri-apps/tauri/pull/4292)) on 2022-06-07
- Use the `TAURI_TRAY` environment variable to determine which package should be added to the Debian `depends` section. Possible values are `ayatana` and `gtk`.
  - [6216eb49](https://www.github.com/tauri-apps/tauri/commit/6216eb49e72863bfb6d4c9edb8827b21406ac393) refactor(core): drop `ayatana-tray` and `gtk-tray` Cargo features ([#4247](https://www.github.com/tauri-apps/tauri/pull/4247)) on 2022-06-02

## \[1.0.0-rc.13]

- Check if `$CWD/src-tauri/tauri.conf.json` exists before walking through the file tree to find the tauri dir in case the whole project is gitignored.
  - [bd8f3e29](https://www.github.com/tauri-apps/tauri/commit/bd8f3e298a0cb71809f2e93cc3ebc8e6e5b6a626) fix(cli): manual config lookup to handle gitignored folders, fixes [#3527](https://www.github.com/tauri-apps/tauri/pull/3527) ([#4224](https://www.github.com/tauri-apps/tauri/pull/4224)) on 2022-05-26
- Statically link the Visual C++ runtime instead of using a merge module on the installer.
  - [bb061509](https://www.github.com/tauri-apps/tauri/commit/bb061509fb674bef86ecbc1de3aa8f3e367a9907) refactor(core): statically link vcruntime, closes [#4122](https://www.github.com/tauri-apps/tauri/pull/4122) ([#4227](https://www.github.com/tauri-apps/tauri/pull/4227)) on 2022-05-27

## \[1.0.0-rc.12]

- Properly fetch the NPM dependency information when using Yarn 2+.
  - [cdfa6255](https://www.github.com/tauri-apps/tauri/commit/cdfa62551115586725bd3e4c04f12c5256f20790) fix(cli): properly read info when using yarn 2+, closes [#4106](https://www.github.com/tauri-apps/tauri/pull/4106) ([#4193](https://www.github.com/tauri-apps/tauri/pull/4193)) on 2022-05-23

## \[1.0.0-rc.11]

- Allow configuring the display options for the MSI execution allowing quieter updates.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.10]

- Resolve binary file extension from target triple instead of compile-time checks to allow cross compilation.
  - [4562e671](https://www.github.com/tauri-apps/tauri/commit/4562e671e4795e9386429348bf738f7078706945) fix(build): append .exe binary based on target triple instead of running OS, closes [#3870](https://www.github.com/tauri-apps/tauri/pull/3870) ([#4032](https://www.github.com/tauri-apps/tauri/pull/4032)) on 2022-05-03
- Fixes text overflow on `tauri dev` on Windows.
  - [094534d1](https://www.github.com/tauri-apps/tauri/commit/094534d138a9286e4746b61adff2da616e3b6a61) fix(cli): dev command stderr text overflow on Windows, closes [#3995](https://www.github.com/tauri-apps/tauri/pull/3995) ([#4000](https://www.github.com/tauri-apps/tauri/pull/4000)) on 2022-04-29
- Improve CLI's logging output, making use of the standard rust `log` system.
  - [35f21471](https://www.github.com/tauri-apps/tauri/commit/35f2147161e6697cbd2824681eeaf870b5a991c2) feat(cli): Improve CLI logging ([#4060](https://www.github.com/tauri-apps/tauri/pull/4060)) on 2022-05-07
- Don't override the default keychain on macOS while code signing.
  - [a4fcaf1d](https://www.github.com/tauri-apps/tauri/commit/a4fcaf1d04aafc3b4d42186f0fb386797d959a9d) fix: don't override default keychain, closes [#4008](https://www.github.com/tauri-apps/tauri/pull/4008) ([#4053](https://www.github.com/tauri-apps/tauri/pull/4053)) on 2022-05-05
- - Remove startup delay in `tauri dev` caused by checking for a newer cli version. The check is now done upon process exit.
- Add `TAURI_SKIP_UPDATE_CHECK` env variable to skip checking for a newer CLI version.
- [bbabc8cd](https://www.github.com/tauri-apps/tauri/commit/bbabc8cd1ea2c1f6806610fd2d533c99305d320c) fix(cli.rs): remove startup delay in `tauri dev` ([#3999](https://www.github.com/tauri-apps/tauri/pull/3999)) on 2022-04-29
- Fix `tauri info` panic when a package isn't installed.
  - [4f0f3187](https://www.github.com/tauri-apps/tauri/commit/4f0f3187c9e69262ef28350331b368c831ab930a) fix(cli.rs): fix `tauri info` panic when a package isn't installed, closes [#3985](https://www.github.com/tauri-apps/tauri/pull/3985) ([#3996](https://www.github.com/tauri-apps/tauri/pull/3996)) on 2022-04-29
- Added `$schema` support to `tauri.conf.json`.
  - [715cbde3](https://www.github.com/tauri-apps/tauri/commit/715cbde3842a916c4ebeab2cab348e1774b5c192) feat(config): add `$schema` to `tauri.conf.json`, closes [#3464](https://www.github.com/tauri-apps/tauri/pull/3464) ([#4031](https://www.github.com/tauri-apps/tauri/pull/4031)) on 2022-05-03
- **Breaking change:** The `dev` command now reads the custom config file from CWD instead of the Tauri folder.
  - [a1929c6d](https://www.github.com/tauri-apps/tauri/commit/a1929c6dacccd00af4cdbcc4d29cfb98d8428f55) fix(cli): always read custom config file from CWD, closes [#4067](https://www.github.com/tauri-apps/tauri/pull/4067) ([#4074](https://www.github.com/tauri-apps/tauri/pull/4074)) on 2022-05-07
- Fixes a Powershell crash when sending SIGINT to the dev command.
  - [32048486](https://www.github.com/tauri-apps/tauri/commit/320484866b83ecabb01eb58d158e0fedd9dd08be) fix(cli): powershell crashing on SIGINT, closes [#3997](https://www.github.com/tauri-apps/tauri/pull/3997) ([#4007](https://www.github.com/tauri-apps/tauri/pull/4007)) on 2022-04-29
- Prevent building when the bundle identifier is the default `com.tauri.dev`.
  - [95726ebb](https://www.github.com/tauri-apps/tauri/commit/95726ebb6180d371be44bff9f16ca1eee049006a) feat(cli): prevent default bundle identifier from building, closes [#4041](https://www.github.com/tauri-apps/tauri/pull/4041) ([#4042](https://www.github.com/tauri-apps/tauri/pull/4042)) on 2022-05-04

## \[1.0.0-rc.9]

- Exit CLI when Cargo returns a non-compilation error in `tauri dev`.
  - [b5622882](https://www.github.com/tauri-apps/tauri/commit/b5622882cf3748e1e5a90915f415c0cd922aaaf8) fix(cli): exit on non-compilation Cargo errors, closes [#3930](https://www.github.com/tauri-apps/tauri/pull/3930) ([#3942](https://www.github.com/tauri-apps/tauri/pull/3942)) on 2022-04-22
- Notify CLI update when running `tauri dev`.
  - [a649aad7](https://www.github.com/tauri-apps/tauri/commit/a649aad7ad26d4578699370d6e63d80edeca1f97) feat(cli): check and notify about updates on `tauri dev`, closes [#3789](https://www.github.com/tauri-apps/tauri/pull/3789) ([#3960](https://www.github.com/tauri-apps/tauri/pull/3960)) on 2022-04-25
- The CLI will not automatically run `strip` on release binaries anymore. Use the \[`strip`]\[strip] profile setting stabilized with Cargo 1.59.

[`strip`]: https://doc.rust-lang.org/cargo/reference/profiles.html#strip

- [62106224](https://www.github.com/tauri-apps/tauri/commit/621062246d065f21800d340126cf58315177f97e) refactor: drop strip from build command. closes [#3559](https://www.github.com/tauri-apps/tauri/pull/3559) ([#3863](https://www.github.com/tauri-apps/tauri/pull/3863)) on 2022-04-06
- Kill the `beforeDevCommand` and app processes if the dev command returns an error.
  - [485c9743](https://www.github.com/tauri-apps/tauri/commit/485c97438ac956d86bcf3794ceaa626bef968a4e) fix(cli): kill beforeDevCommand if dev code returns an error ([#3907](https://www.github.com/tauri-apps/tauri/pull/3907)) on 2022-04-19
- Fix `info` command showing outdated text for latest versions.
  - [73a4b74a](https://www.github.com/tauri-apps/tauri/commit/73a4b74aea8544e6fda51c1f6697630b0768072c) fix(cli.rs/info):  don't show outdated text for latest versions ([#3829](https://www.github.com/tauri-apps/tauri/pull/3829)) on 2022-04-02
- **Breaking change:** Enable default Cargo features except `tauri/custom-protocol` on the dev command.
  - [f2a30d8b](https://www.github.com/tauri-apps/tauri/commit/f2a30d8bc54fc3ba49e16f69a413eca5f61a9b1f) refactor(core): use ayatana appindicator by default, keep option to use gtk ([#3916](https://www.github.com/tauri-apps/tauri/pull/3916)) on 2022-04-19
- Kill the `beforeDevCommand` process recursively on Unix.
  - [e251e1b0](https://www.github.com/tauri-apps/tauri/commit/e251e1b0991d26ab10aea33cfb228f3e7f0f85b5) fix(cli): kill before dev command recursively on Unix, closes [#2794](https://www.github.com/tauri-apps/tauri/pull/2794) ([#3848](https://www.github.com/tauri-apps/tauri/pull/3848)) on 2022-04-03

## \[1.0.0-rc.8]

- Allows the `tauri.conf.json` file to be git ignored on the path lookup function.
  - [cc7c2d77](https://www.github.com/tauri-apps/tauri/commit/cc7c2d77da2e4a39ec2a97b080d41a719e6d0161) feat(cli): allow conf path to be gitignored, closes [#3636](https://www.github.com/tauri-apps/tauri/pull/3636) ([#3683](https://www.github.com/tauri-apps/tauri/pull/3683)) on 2022-03-13
- Remove `minimumSystemVersion: null` from the application template configuration.
  - [c81534eb](https://www.github.com/tauri-apps/tauri/commit/c81534ebd873c358e0346c7949aeb171803149a5) feat(cli): use default macOS minimum system version when it is empty ([#3658](https://www.github.com/tauri-apps/tauri/pull/3658)) on 2022-03-13
- Improve readability of the `info` subcommand output.
  - [49d2f13f](https://www.github.com/tauri-apps/tauri/commit/49d2f13fc07d763d5de9bf4b19d00c901776c11d) feat(cli): colorful cli ([#3635](https://www.github.com/tauri-apps/tauri/pull/3635)) on 2022-03-08
- Properly terminate the `beforeDevCommand` process.
  - [94d78efb](https://www.github.com/tauri-apps/tauri/commit/94d78efbe542e7be3c00e3b2355c22803816715f) fix(cli.rs): terminate the beforeDevCommand, closes [#2794](https://www.github.com/tauri-apps/tauri/pull/2794) ([#2883](https://www.github.com/tauri-apps/tauri/pull/2883)) on 2022-03-27
- Fixes DMG bundling on macOS 12.3.
  - [348a1ab5](https://www.github.com/tauri-apps/tauri/commit/348a1ab59d2697478a594016016f1fccbf1ac054) fix(bundler): DMG bundling on macOS 12.3 cannot use bless, closes [#3719](https://www.github.com/tauri-apps/tauri/pull/3719) ([#3721](https://www.github.com/tauri-apps/tauri/pull/3721)) on 2022-03-18
- Fixes resources bundling on Windows when the path is on the root of the Tauri folder.
  - [4c84559e](https://www.github.com/tauri-apps/tauri/commit/4c84559e1f3019e7aa2666b10a1a0bd97bb09d24) fix(cli): root resource bundling on Windows, closes [#3539](https://www.github.com/tauri-apps/tauri/pull/3539) ([#3685](https://www.github.com/tauri-apps/tauri/pull/3685)) on 2022-03-13

## \[1.0.0-rc.7]

- Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
  server under code signing on Windows if set to `true`.
  - [bdd5f7c2](https://www.github.com/tauri-apps/tauri/commit/bdd5f7c2f03af4af8b60a9527e55bb18525d989b) fix: add support for Time-Stamping Protocol for Windows codesigning (fix [#3563](https://www.github.com/tauri-apps/tauri/pull/3563)) ([#3570](https://www.github.com/tauri-apps/tauri/pull/3570)) on 2022-03-07
- Change the `plugin init` templates to use the new `tauri::plugin::Builder` syntax.
  - [f7acb061](https://www.github.com/tauri-apps/tauri/commit/f7acb061e4d1ecdbfe182793587632d7ba6d8eff) feat(cli): use plugin::Builder syntax on the plugin template ([#3606](https://www.github.com/tauri-apps/tauri/pull/3606)) on 2022-03-03

## \[1.0.0-rc.6]

- Improve "waiting for your dev server to start" message.
  - [5999379f](https://www.github.com/tauri-apps/tauri/commit/5999379fb06052a115f04f99274ab46d1eefd659) chore(cli): improve "waiting for dev server" message, closes [#3491](https://www.github.com/tauri-apps/tauri/pull/3491) ([#3504](https://www.github.com/tauri-apps/tauri/pull/3504)) on 2022-02-18
- Do not panic if the updater private key password is wrong.
  - [17f17a80](https://www.github.com/tauri-apps/tauri/commit/17f17a80f818bcc20c387583a6ff00a8e07ec533) fix(cli): do not panic if private key password is wrong, closes [#3449](https://www.github.com/tauri-apps/tauri/pull/3449) ([#3495](https://www.github.com/tauri-apps/tauri/pull/3495)) on 2022-02-17
- Check the current folder before checking the directories on the app and tauri dir path lookup function.
  - [a06de376](https://www.github.com/tauri-apps/tauri/commit/a06de3760184caa71acfe7a2fe2189a033b565f5) fix(cli): path lookup should not check subfolder before the current one ([#3465](https://www.github.com/tauri-apps/tauri/pull/3465)) on 2022-02-15
- Fixes the signature of the `signer sign` command to not have duplicated short flags.
  - [a9755514](https://www.github.com/tauri-apps/tauri/commit/a975551461f3698db3f3b6afa5101189aaeeada9) fix(cli): duplicated short flag for `signer sign`, closes [#3483](https://www.github.com/tauri-apps/tauri/pull/3483) ([#3492](https://www.github.com/tauri-apps/tauri/pull/3492)) on 2022-02-17

## \[1.0.0-rc.5]

- Allow passing arguments to the `build` runner (`tauri build -- <ARGS>...`).
  - [679fe1fe](https://www.github.com/tauri-apps/tauri/commit/679fe1fedd6ed016ab1140c8087c2d1404504bfb) feat(cli.rs): allow passing arguments to the build runner, closes [#3398](https://www.github.com/tauri-apps/tauri/pull/3398) ([#3431](https://www.github.com/tauri-apps/tauri/pull/3431)) on 2022-02-13
- Improve error message when the dev runner command fails.
  - [759d1afb](https://www.github.com/tauri-apps/tauri/commit/759d1afb86f3657f6071a2ae39c9be21e20ed22c) feat(cli): improve error message when dev runner command fails ([#3447](https://www.github.com/tauri-apps/tauri/pull/3447)) on 2022-02-13
- Increase `tauri.conf.json` directory lookup depth to `3` and allow changing it with the `TAURI_PATH_DEPTH` environment variable.
  - [c6031c70](https://www.github.com/tauri-apps/tauri/commit/c6031c7070c6bb7539bbfdfe42cb73012829c910) feat(cli): increase lookup depth, add env var option ([#3451](https://www.github.com/tauri-apps/tauri/pull/3451)) on 2022-02-13
- Added `tauri-build`, `tao` and `wry` version to the `info` command output.
  - [16f1173f](https://www.github.com/tauri-apps/tauri/commit/16f1173f456b1db543d0160df2c9828708bfc68a) feat(cli): add tao and wry version to the `info` output ([#3443](https://www.github.com/tauri-apps/tauri/pull/3443)) on 2022-02-13
- **Breaking change:** The extra arguments passed to `tauri dev` using `-- <ARGS>...` are now propagated to the runner (defaults to cargo). To pass arguments to your binary using Cargo, you now need to run `tauri dev -- -- <ARGS-TO-YOUR-BINARY>...` (notice the double `--`).
  - [679fe1fe](https://www.github.com/tauri-apps/tauri/commit/679fe1fedd6ed016ab1140c8087c2d1404504bfb) feat(cli.rs): allow passing arguments to the build runner, closes [#3398](https://www.github.com/tauri-apps/tauri/pull/3398) ([#3431](https://www.github.com/tauri-apps/tauri/pull/3431)) on 2022-02-13
- Change the `init` template configuration to disable CSP for better usability for new users.
  - [102a5e9b](https://www.github.com/tauri-apps/tauri/commit/102a5e9bb83c5d8388dc9aedc7f03cc57bdae8cb) refactor(cli.rs): change template config CSP to null, closes [#3427](https://www.github.com/tauri-apps/tauri/pull/3427) ([#3429](https://www.github.com/tauri-apps/tauri/pull/3429)) on 2022-02-13

## \[1.0.0-rc.4]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.3]

- Fixes Tauri path resolution on projects without Git or a `.gitignore` file.
  - [d8acbe11](https://www.github.com/tauri-apps/tauri/commit/d8acbe11492bd990e6983c7e63e0f1a8f1ea5c7c) fix(cli.rs): app path resolution on projects without git, closes [#3409](https://www.github.com/tauri-apps/tauri/pull/3409) ([#3410](https://www.github.com/tauri-apps/tauri/pull/3410)) on 2022-02-11

## \[1.0.0-rc.2]

- Fix `init` command prompting for values even if the argument has been provided on the command line.
  - [def76840](https://www.github.com/tauri-apps/tauri/commit/def76840257a1447723ecda13c807cf0c881f083) fix(cli.rs): do not prompt for `init` values if arg set ([#3400](https://www.github.com/tauri-apps/tauri/pull/3400)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11
- Fixes CLI freezing when running `light.exe` on Windows without the `--verbose` flag.
  - [8beab636](https://www.github.com/tauri-apps/tauri/commit/8beab6363491e2a8757cc9fc0fa1eccc98ece916) fix(cli): build freezing on Windows, closes [#3399](https://www.github.com/tauri-apps/tauri/pull/3399) ([#3402](https://www.github.com/tauri-apps/tauri/pull/3402)) on 2022-02-11
- Respect `.gitignore` configuration when looking for the folder with the `tauri.conf.json` file.
  - [9c6c5a8c](https://www.github.com/tauri-apps/tauri/commit/9c6c5a8c52c6460d0b0a1a55300e1828262994ba) perf(cli.rs): improve performance of tauri dir lookup reading .gitignore ([#3405](https://www.github.com/tauri-apps/tauri/pull/3405)) on 2022-02-11
  - [41052dee](https://www.github.com/tauri-apps/tauri/commit/41052deeda2a00ee2b8ec2041c9c87c11de82ab2) fix(covector): add cli.js to change files on 2022-02-11

## \[1.0.0-rc.1]

- Include `vswhere.exe` on the published package.
  - [3227502e](https://www.github.com/tauri-apps/tauri/commit/3227502e8c9f137e5783cba2e0c692473cc5456d) fix(cli.rs): package `vswhere.exe` on 2022-02-10

## \[1.0.0-rc.0]

- Do not force Tauri application code on `src-tauri` folder and use a glob pattern to look for a subfolder with a `tauri.conf.json` file.
  - [a8cff6b3](https://www.github.com/tauri-apps/tauri/commit/a8cff6b3bc3288a53d7cdc5b3cb95d371309d2d6) feat(cli): do not enforce `src-tauri` folder structure, closes [#2643](https://www.github.com/tauri-apps/tauri/pull/2643) ([#2654](https://www.github.com/tauri-apps/tauri/pull/2654)) on 2021-09-27
- Define `TAURI_PLATFORM`, `TAURI_ARCH`, `TAURI_FAMILY`, `TAURI_PLATFORM_TYPE`, `TAURI_PLATFORM_VERSION` and `TAURI_DEBUG` environment variables for the `beforeDevCommand` and `beforeBuildCommand` scripts.
  - [8599313a](https://www.github.com/tauri-apps/tauri/commit/8599313a0f56d9777d335426467e79ba687be1d4) feat(cli.rs): env vars for beforeDev/beforeBuild commands, closes [#2610](https://www.github.com/tauri-apps/tauri/pull/2610) ([#2655](https://www.github.com/tauri-apps/tauri/pull/2655)) on 2021-09-26
  - [b5ee03a1](https://www.github.com/tauri-apps/tauri/commit/b5ee03a13a1c0d0ff677cf9c8d7ef28516fffa5b) feat(cli.rs): expose debug flag to beforeDev/beforeBuild commands ([#2727](https://www.github.com/tauri-apps/tauri/pull/2727)) on 2021-10-08
  - [9bb68973](https://www.github.com/tauri-apps/tauri/commit/9bb68973dd10f3cb98d2a95e5432bfc765d77064) fix(cli.rs): prefix the "before script" env vars with `TAURI_` ([#3274](https://www.github.com/tauri-apps/tauri/pull/3274)) on 2022-01-24
- Allow `config` arg to be a path to a JSON file on the `dev` and `build` commands.
  - [7b81e5b8](https://www.github.com/tauri-apps/tauri/commit/7b81e5b82e665fe0562b91ac33b63a871af9e111) feat(cli.rs): allow config argument to be a path to a JSON file ([#2938](https://www.github.com/tauri-apps/tauri/pull/2938)) on 2021-11-22
- Add `rustup` version and active rust toolchain to the `info` command output.
  - [28aaec87](https://www.github.com/tauri-apps/tauri/commit/28aaec87e2f6445859e9dbaaf2231d02d1e1d4b5) feat(cli.rs): add active toolchain and rustup to `tauri info`, closes [#2730](https://www.github.com/tauri-apps/tauri/pull/2730) ([#2986](https://www.github.com/tauri-apps/tauri/pull/2986)) on 2021-12-09
- Add `Visual Studio Build Tools` installed versions to the `info` command output.
  - [d5f07d14](https://www.github.com/tauri-apps/tauri/commit/d5f07d14f31954fe89ff5045fd4ef72cfc2b9ac1) feat(cli.rs): build tools info ([#2618](https://www.github.com/tauri-apps/tauri/pull/2618)) on 2021-09-21
- The inferred development server port for Svelte is now `8080` (assumes latest Svelte with `sirv-cli >= 2.0.0`).
  - [de0543f3](https://www.github.com/tauri-apps/tauri/commit/de0543f3e052d2981a95ce3baa8470592740a1a2) feat(cli.rs): change inferred dev server port to 8080 for Svelte apps on 2022-02-05
- Detect if tauri is used from git in the `info` command.
  - [65ad5b5e](https://www.github.com/tauri-apps/tauri/commit/65ad5b5ef923bf1e6b6f078d794d071a04fcdf57) feat(cli.rs/info): detect if tauri is used from git ([#3309](https://www.github.com/tauri-apps/tauri/pull/3309)) on 2022-02-05
- Drop the `dialoguer` soft fork and use the published version instead.
  - [b1f5c6d7](https://www.github.com/tauri-apps/tauri/commit/b1f5c6d7ac48c7407f28402afef0d3e521314127) refactor(cli.rs): drop `dialoguer` and `console` soft fork ([#2790](https://www.github.com/tauri-apps/tauri/pull/2790)) on 2021-10-22
- Fix `build` command when executed on a 32-bit Windows machine when pulling from the `binary-releases` repo.
  - [35588b2e](https://www.github.com/tauri-apps/tauri/commit/35588b2e04d5be8e5708583bdc52a012341bc75e) fix(cli.rs): check default arch at runtime, closes [#3067](https://www.github.com/tauri-apps/tauri/pull/3067) ([#3078](https://www.github.com/tauri-apps/tauri/pull/3078)) on 2021-12-27
- The `generate` and `sign` commands are now available under a `signer` subcommand.
  - [1458ab3c](https://www.github.com/tauri-apps/tauri/commit/1458ab3c535637ada996ab0ff3494cd75fe40bf7) refactor(cli.rs): `signer` and `plugin` subcommands, use new clap derive syntax ([#2928](https://www.github.com/tauri-apps/tauri/pull/2928)) on 2021-12-09
- Use `tauri-utils` to get the `Config` types.
  - [4de285c3](https://www.github.com/tauri-apps/tauri/commit/4de285c3967d32250d73acdd5d171a6fd332d2b3) feat(core): validate Cargo features matching allowlist \[TRI-023] on 2022-01-09
- Print warning and exit if `distDir` contains `node_modules`, `src-tauri` or `target` folders.
  - [7ed3f3b7](https://www.github.com/tauri-apps/tauri/commit/7ed3f3b7e4268708bbe8f83c45653e5d6704824b) feat(cli.rs): validate `distDir`, closes [#2554](https://www.github.com/tauri-apps/tauri/pull/2554) ([#2701](https://www.github.com/tauri-apps/tauri/pull/2701)) on 2021-10-04
- Fix `tauri build` failing on Windows if `tauri.conf.json > tauri > bundle > Windows > wix > license` is used.
  - [17a1ad68](https://www.github.com/tauri-apps/tauri/commit/17a1ad682363e51365b57899c8d7557b1b65201c) fix(cli.rs): ensure `target/release/wix` exists, closes [#2927](https://www.github.com/tauri-apps/tauri/pull/2927) ([#2987](https://www.github.com/tauri-apps/tauri/pull/2987)) on 2021-12-07
- Added `dev_csp` to the `security` configuration object.
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- Kill process if `beforeDevCommand` exits with a non-zero status code.
  - [a2d5929a](https://www.github.com/tauri-apps/tauri/commit/a2d5929a8f1f6310d186199cc54246fcb0a01b46) feat(cli.rs): wait for dev URL to be reachable, exit if command fails ([#3358](https://www.github.com/tauri-apps/tauri/pull/3358)) on 2022-02-08
- Fixes output directory detection when using Cargo workspaces.
  - [8d630bc8](https://www.github.com/tauri-apps/tauri/commit/8d630bc8c494cba6ac1604b7777b89b763044471) fix(cli.rs): fix workspace detection, fixes [#2614](https://www.github.com/tauri-apps/tauri/pull/2614), closes [#2515](https://www.github.com/tauri-apps/tauri/pull/2515) ([#2644](https://www.github.com/tauri-apps/tauri/pull/2644)) on 2021-09-23
- Allow using a fixed version for the Webview2 runtime via the `tauri > bundle > windows > webviewFixedRuntimePath` config option.
  - [85df94f2](https://www.github.com/tauri-apps/tauri/commit/85df94f2b0d40255812b42c5e32a70c4b45392df) feat(core): config for fixed webview2 runtime version path ([#27](https://www.github.com/tauri-apps/tauri/pull/27)) on 2021-11-02
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
- Added `$ tauri plugin init` command, which initializes a Tauri plugin.
  - [ac8e69a9](https://www.github.com/tauri-apps/tauri/commit/ac8e69a98ca1d6f646344ffdef1876cc9274323a) feat(cli.rs): add `init plugin` command, bootstraps a plugin project ([#2669](https://www.github.com/tauri-apps/tauri/pull/2669)) on 2021-09-27
  - [db275f0b](https://www.github.com/tauri-apps/tauri/commit/db275f0b633f44fb2f85755d32929dfb7893b1e0) refactor(cli.rs): rename `init plugin` subcommand to `plugin init` ([#2885](https://www.github.com/tauri-apps/tauri/pull/2885)) on 2021-11-13
- **Breaking change:** Add `macos-private-api` feature flag, enabled via `tauri.conf.json > tauri > macOSPrivateApi`.
  - [6ac21b3c](https://www.github.com/tauri-apps/tauri/commit/6ac21b3cef7f14358df38cc69ea3d277011accaf) feat: add private api feature flag ([#7](https://www.github.com/tauri-apps/tauri/pull/7)) on 2022-01-09
- Move the copying of resources and sidecars from `cli.rs` to `tauri-build` so using the Cargo CLI directly processes the files for the application execution in development.
  - [5eb72c24](https://www.github.com/tauri-apps/tauri/commit/5eb72c24deddf5a01093bea96b90c0d8806afc3f) refactor: copy resources and sidecars on the Cargo build script ([#3357](https://www.github.com/tauri-apps/tauri/pull/3357)) on 2022-02-08
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Automatically `strip` the built binary on Linux and macOS if `--debug` is not specified.
  - [2f3a582c](https://www.github.com/tauri-apps/tauri/commit/2f3a582c69994d66f2035bbe62825eafc869d90f) feat(cli.rs): strip release binaries \[TRI-031] ([#22](https://www.github.com/tauri-apps/tauri/pull/22)) on 2022-01-09
- Fixes pnpm error when running `pnpm tauri info`.
  - [2026134f](https://www.github.com/tauri-apps/tauri/commit/2026134f471a212aea9b227a873ecc937dda1acc) fix(cli.rs): pnpm tauri info exits with error (fix [#2509](https://www.github.com/tauri-apps/tauri/pull/2509)) ([#2510](https://www.github.com/tauri-apps/tauri/pull/2510)) on 2021-08-24
- Add support to building Universal macOS Binaries through the virtual target `universal-apple-darwin` (run `tauri build --target universal-apple-darwin`).
  - [83f52fdb](https://www.github.com/tauri-apps/tauri/commit/83f52fdbe3a9ffd98dffc752e5f9e14322b56e60) feat: Add `universal-darwin-macos` build target, closes [#3317](https://www.github.com/tauri-apps/tauri/pull/3317) ([#3318](https://www.github.com/tauri-apps/tauri/pull/3318)) on 2022-02-04
- Wait for `devPath` URL to be reachable before starting the application. Skipped if the `TAURI_SKIP_DEVSERVER_CHECK` environment variable is set to `true`.
  - [a2d5929a](https://www.github.com/tauri-apps/tauri/commit/a2d5929a8f1f6310d186199cc54246fcb0a01b46) feat(cli.rs): wait for dev URL to be reachable, exit if command fails ([#3358](https://www.github.com/tauri-apps/tauri/pull/3358)) on 2022-02-08
- On Windows, Fix `beforeDevCommand` and `beforeBuildCommand` not executing the expected command if it contains quotes. This is done by executing them with `CMD /S /C {command}` instead of `CMD /C {command}` on Windows.
  - [52e9a6d8](https://www.github.com/tauri-apps/tauri/commit/52e9a6d81a701a66a8cf6a67c2b37d135583543f) fix: Make CMD handle quotes `"` properly. ([#3334](https://www.github.com/tauri-apps/tauri/pull/3334)) on 2022-02-06
- Allow setting the localization file for WiX.
  - [af329f27](https://www.github.com/tauri-apps/tauri/commit/af329f2722d6194c6d70e976fc970dc2c9e4de2b) feat(bundler): wix localization, closes [#3174](https://www.github.com/tauri-apps/tauri/pull/3174) ([#3179](https://www.github.com/tauri-apps/tauri/pull/3179)) on 2022-02-05

## \[1.0.0-beta.7]

- Update cli.yml to pass clap ArgSettings::MultipleValues assertion.
  - [0391ac3d](https://www.github.com/tauri-apps/tauri/commit/0391ac3dc96d9c74c34a957e4cb70da88a0a85b7) fix: Update cli.yml to pass clap ArgSettings::MultipleValues assertion. ([#2506](https://www.github.com/tauri-apps/tauri/pull/2506)) ([#2507](https://www.github.com/tauri-apps/tauri/pull/2507)) on 2021-08-22

## \[1.0.0-beta.6]

- Added `APPLE_SIGNING_IDENTITY` as supported environment variable for the bundler.
  - [44f6ee4c](https://www.github.com/tauri-apps/tauri/commit/44f6ee4cfdfad5fb21d96e69f8776c0e68685682) chore(ci): add step to detect code signing ([#2245](https://www.github.com/tauri-apps/tauri/pull/2245)) on 2021-08-08
- Added configuration for the WiX banner icon under `tauri.conf.json > tauri > bundle > windows > wix > bannerPath`.
  - [13003ec7](https://www.github.com/tauri-apps/tauri/commit/13003ec761b1530705d6129519dc4e226eb992c7) feat(bundler): add config for WiX banner path, closes [#2175](https://www.github.com/tauri-apps/tauri/pull/2175) ([#2448](https://www.github.com/tauri-apps/tauri/pull/2448)) on 2021-08-16
- Added configuration for the WiX dialog background bitmap under `tauri.conf.json > tauri > bundle > windows > wix > dialogImagePath`.
  - [9bfdeb42](https://www.github.com/tauri-apps/tauri/commit/9bfdeb42effeeec27aa15bbc5b05040eadfda5ba) feat(bundler): add config for WiX dialog image path ([#2449](https://www.github.com/tauri-apps/tauri/pull/2449)) on 2021-08-16
- Only convert package name and binary name to kebab-case, keeping the `.desktop` `Name` field with the original configured value.
  - [3f039cb8](https://www.github.com/tauri-apps/tauri/commit/3f039cb8a308b0f18deaa37d7cfb1cc50d308d0e) fix: keep original `productName` for .desktop `Name` field, closes [#2295](https://www.github.com/tauri-apps/tauri/pull/2295) ([#2384](https://www.github.com/tauri-apps/tauri/pull/2384)) on 2021-08-10
- Merge platform-specific `tauri.linux.conf.json`, `tauri.windows.conf.json` and `tauri.macos.conf.json` into the config JSON from `tauri.conf.json`.
  - [71d687b7](https://www.github.com/tauri-apps/tauri/commit/71d687b787cd722c60879adda543897826bf42c9) feat(cli.rs): platform-specific conf.json ([#2309](https://www.github.com/tauri-apps/tauri/pull/2309)) on 2021-07-28
- Update minimum Rust version to 1.54.0.
  - [a5394716](https://www.github.com/tauri-apps/tauri/commit/a53947160985a4f5b0ad1fbb4aa6865d6f852c66) chore: update rust to 1.54.0 ([#2434](https://www.github.com/tauri-apps/tauri/pull/2434)) on 2021-08-15

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
