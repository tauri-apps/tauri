# Changelog

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
- Fixes DMG bundling on macOS 12.3.
  - [348a1ab5](https://www.github.com/tauri-apps/tauri/commit/348a1ab59d2697478a594016016f1fccbf1ac054) fix(bundler): DMG bundling on macOS 12.3 cannot use bless, closes [#3719](https://www.github.com/tauri-apps/tauri/pull/3719) ([#3721](https://www.github.com/tauri-apps/tauri/pull/3721)) on 2022-03-18
- Fixes resources bundling on Windows when the path is on the root of the Tauri folder.
  - [4c84559e](https://www.github.com/tauri-apps/tauri/commit/4c84559e1f3019e7aa2666b10a1a0bd97bb09d24) fix(cli): root resource bundling on Windows, closes [#3539](https://www.github.com/tauri-apps/tauri/pull/3539) ([#3685](https://www.github.com/tauri-apps/tauri/pull/3685)) on 2022-03-13

## \[1.0.0-rc.6]

- Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
  server under code signing on Windows if set to `true`.
  - [bdd5f7c2](https://www.github.com/tauri-apps/tauri/commit/bdd5f7c2f03af4af8b60a9527e55bb18525d989b) fix: add support for Time-Stamping Protocol for Windows codesigning (fix [#3563](https://www.github.com/tauri-apps/tauri/pull/3563)) ([#3570](https://www.github.com/tauri-apps/tauri/pull/3570)) on 2022-03-07
- Added `i686-pc-windows-msvc` to the prebuilt targets.
  - [fb6744da](https://www.github.com/tauri-apps/tauri/commit/fb6744daa45165c7e00e5c01f7df0880d69ca509) feat(cli.js): add 32bit cli for windows ([#3540](https://www.github.com/tauri-apps/tauri/pull/3540)) on 2022-02-24
- Change the `plugin init` templates to use the new `tauri::plugin::Builder` syntax.
  - [f7acb061](https://www.github.com/tauri-apps/tauri/commit/f7acb061e4d1ecdbfe182793587632d7ba6d8eff) feat(cli): use plugin::Builder syntax on the plugin template ([#3606](https://www.github.com/tauri-apps/tauri/pull/3606)) on 2022-03-03

## \[1.0.0-rc.5]

- Improve "waiting for your dev server to start" message.
  - [5999379f](https://www.github.com/tauri-apps/tauri/commit/5999379fb06052a115f04f99274ab46d1eefd659) chore(cli): improve "waiting for dev server" message, closes [#3491](https://www.github.com/tauri-apps/tauri/pull/3491) ([#3504](https://www.github.com/tauri-apps/tauri/pull/3504)) on 2022-02-18
- Do not panic if the updater private key password is wrong.
  - [17f17a80](https://www.github.com/tauri-apps/tauri/commit/17f17a80f818bcc20c387583a6ff00a8e07ec533) fix(cli): do not panic if private key password is wrong, closes [#3449](https://www.github.com/tauri-apps/tauri/pull/3449) ([#3495](https://www.github.com/tauri-apps/tauri/pull/3495)) on 2022-02-17
- Check the current folder before checking the directories on the app and tauri dir path lookup function.
  - [a06de376](https://www.github.com/tauri-apps/tauri/commit/a06de3760184caa71acfe7a2fe2189a033b565f5) fix(cli): path lookup should not check subfolder before the current one ([#3465](https://www.github.com/tauri-apps/tauri/pull/3465)) on 2022-02-15
- Fixes the signature of the `signer sign` command to not have duplicated short flags.
  - [a9755514](https://www.github.com/tauri-apps/tauri/commit/a975551461f3698db3f3b6afa5101189aaeeada9) fix(cli): duplicated short flag for `signer sign`, closes [#3483](https://www.github.com/tauri-apps/tauri/pull/3483) ([#3492](https://www.github.com/tauri-apps/tauri/pull/3492)) on 2022-02-17

## \[1.0.0-rc.4]

- Change the `run` function to take a callback and run asynchronously instead of blocking the event loop.
  - [cd9a20b9](https://www.github.com/tauri-apps/tauri/commit/cd9a20b9ab013759b4bdb742f358988022450795) refactor(cli.js): run on separate thread ([#3436](https://www.github.com/tauri-apps/tauri/pull/3436)) on 2022-02-13
- Improve error message when the dev runner command fails.
  - [759d1afb](https://www.github.com/tauri-apps/tauri/commit/759d1afb86f3657f6071a2ae39c9be21e20ed22c) feat(cli): improve error message when dev runner command fails ([#3447](https://www.github.com/tauri-apps/tauri/pull/3447)) on 2022-02-13
- Show full error message from `cli.rs` instead of just the outermost underlying error message.
  - [63826010](https://www.github.com/tauri-apps/tauri/commit/63826010d1f38544f36afd3aac67c45d4608d15b) feat(cli.js): show full error message ([#3442](https://www.github.com/tauri-apps/tauri/pull/3442)) on 2022-02-13
- Increase `tauri.conf.json` directory lookup depth to `3` and allow changing it with the `TAURI_PATH_DEPTH` environment variable.
  - [c6031c70](https://www.github.com/tauri-apps/tauri/commit/c6031c7070c6bb7539bbfdfe42cb73012829c910) feat(cli): increase lookup depth, add env var option ([#3451](https://www.github.com/tauri-apps/tauri/pull/3451)) on 2022-02-13
- Added `tauri-build`, `tao` and `wry` version to the `info` command output.
  - [16f1173f](https://www.github.com/tauri-apps/tauri/commit/16f1173f456b1db543d0160df2c9828708bfc68a) feat(cli): add tao and wry version to the `info` output ([#3443](https://www.github.com/tauri-apps/tauri/pull/3443)) on 2022-02-13

## \[1.0.0-rc.3]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in cli.rs.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

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
