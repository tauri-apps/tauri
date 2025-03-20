# Changelog

## \[2.4.0]

### New Features

- [`d91bfa5cb`](https://www.github.com/tauri-apps/tauri/commit/d91bfa5cb921a078758edd45ef3eaff71358d1eb) ([#12970](https://www.github.com/tauri-apps/tauri/pull/12970) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Allow merging multiple configuration values on `tauri dev`, `tauri build`, `tauri bundle`, `tauri android dev`, `tauri android build`, `tauri ios dev` and `tauri ios build`.
- [`30f5a1553`](https://www.github.com/tauri-apps/tauri/commit/30f5a1553d3c0ce460c9006764200a9210915a44) ([#12366](https://www.github.com/tauri-apps/tauri/pull/12366) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Added `trafficLightPosition` window configuration to set the traffic light buttons position on macOS.

### Enhancements

- [`f981a5ee8`](https://www.github.com/tauri-apps/tauri/commit/f981a5ee8b292b9ea09329f60cecc7f688dda734) ([#12602](https://www.github.com/tauri-apps/tauri/pull/12602) by [@kxxt](https://www.github.com/tauri-apps/tauri/../../kxxt)) Add basic support for linux riscv64 platform.

### Bug Fixes

- [`0c4700e99`](https://www.github.com/tauri-apps/tauri/commit/0c4700e9907f242eabe579eb6149a1d75174185c) ([#12985](https://www.github.com/tauri-apps/tauri/pull/12985) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) The cli will now accept `--bundles updater` again. It's still no-op as it has been for all v2 versions. If you want to build updater artifacts, enable `createUpdaterArtifacts` in `tauri.conf.json`.
- [`b83921226`](https://www.github.com/tauri-apps/tauri/commit/b83921226cb3084992bb5357e7e39a09ea97843e) ([#12977](https://www.github.com/tauri-apps/tauri/pull/12977) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix `tauri ios` commands using the wrong working directory with `bun@>1.2`.
- [`f268b3dbd`](https://www.github.com/tauri-apps/tauri/commit/f268b3dbdf313484c85b4a1f69cd7cec63049f35) ([#12871](https://www.github.com/tauri-apps/tauri/pull/12871) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Ignore parent .gitignore files on the Tauri project path detection.

### Dependencies

- Upgraded to `tauri-cli@2.4.0`

## \[2.3.1]

### Dependencies

- Upgraded to `tauri-cli@2.3.1`

## \[2.3.0]

### Enhancements

- [`a2d36b8c3`](https://www.github.com/tauri-apps/tauri/commit/a2d36b8c34a8dcfc6736797ca5cd4665faf75e7e) ([#12181](https://www.github.com/tauri-apps/tauri/pull/12181) by [@bastiankistner](https://www.github.com/tauri-apps/tauri/../../bastiankistner)) Add an option to change the default background throttling policy (currently for WebKit only).

### Dependencies

- Upgraded to `tauri-cli@2.3.0`

## \[2.2.7]

### Bug Fixes

- [`8e9134c4a`](https://www.github.com/tauri-apps/tauri/commit/8e9134c4a2047329be0dbb868b7ae061a9d3f190) ([#12511](https://www.github.com/tauri-apps/tauri/pull/12511) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that caused `tauri dev` to fail because of an incorrect `--bins` flag.

### Dependencies

- Upgraded to `tauri-cli@2.2.7`

## \[2.2.6]

### Enhancements

- [`1a86974aa`](https://www.github.com/tauri-apps/tauri/commit/1a86974aa3d09957c6b1142a17bbfed9998798fd) ([#12406](https://www.github.com/tauri-apps/tauri/pull/12406) by [@bradleat](https://www.github.com/tauri-apps/tauri/../../bradleat)) `ios build --open` will now let xcode start the rust build process.
- [`0b79af711`](https://www.github.com/tauri-apps/tauri/commit/0b79af711430934362602fb950c3e4cb5b59cf9c) ([#12438](https://www.github.com/tauri-apps/tauri/pull/12438) by [@3lpsy](https://www.github.com/tauri-apps/tauri/../../3lpsy)) Log the command used to start the rust app in development.

### Dependencies

- Upgraded to `tauri-cli@2.2.6`

## \[2.2.5]

### Dependencies

- Upgraded to `tauri-cli@2.2.5`

## \[2.2.4]

### Bug Fixes

- [`cad550445`](https://www.github.com/tauri-apps/tauri/commit/cad5504455ffa53e297cebff473c113b1afa5d29) ([#12354](https://www.github.com/tauri-apps/tauri/pull/12354) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed and issue that caused `tauri add` to try to install incorrect npm packages.

### Dependencies

- Upgraded to `tauri-cli@2.2.4`

## \[2.2.3]

### Enhancements

- [`a0f2c84d5`](https://www.github.com/tauri-apps/tauri/commit/a0f2c84d51f5086c5055867d6f61ea90c463a26c) ([#12204](https://www.github.com/tauri-apps/tauri/pull/12204) by [@pjf-dev](https://www.github.com/tauri-apps/tauri/../../pjf-dev)) Enhance `tauri icon` command by including 64x64 png size in default icon sizes.

### Bug Fixes

- [`98f62e65a`](https://www.github.com/tauri-apps/tauri/commit/98f62e65a27a375272c6b4d9f34c23e142b9d3a6) ([#12246](https://www.github.com/tauri-apps/tauri/pull/12246) by [@marcomq](https://www.github.com/tauri-apps/tauri/../../marcomq)) Properly add NPM packages for community plugins when using the `tauri add` command.
- [`b9a99a5c6`](https://www.github.com/tauri-apps/tauri/commit/b9a99a5c69d8a2a1a3ff30e500b46872258dca15) ([#12297](https://www.github.com/tauri-apps/tauri/pull/12297) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that caused the built-in dev server to constantly refresh on Linux. This only affected users who do not have `devUrl` point to a URL.
- [`ef21ed9ac`](https://www.github.com/tauri-apps/tauri/commit/ef21ed9ac1c045c38b0c04e3d71a441694abc257) ([#12290](https://www.github.com/tauri-apps/tauri/pull/12290) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix iOS build failing when the development team contains spaces.

### Dependencies

- Upgraded to `tauri-cli@2.2.3`

## \[2.2.2]

### Bug Fixes

- [`26fc9558f`](https://www.github.com/tauri-apps/tauri/commit/26fc9558fe7b2fe649f61926da88f36110dd5707) ([#12178](https://www.github.com/tauri-apps/tauri/pull/12178) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that caused the `tauri dev` file watcher to exit after detecting file changes.

### Dependencies

- Upgraded to `tauri-cli@2.2.2`

## \[2.2.1]

### Bug Fixes

- [`881729448`](https://www.github.com/tauri-apps/tauri/commit/881729448c9abd0d0c7941a8a31c94119ce827af) ([#12164](https://www.github.com/tauri-apps/tauri/pull/12164) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that caused `tauri dev` to crash before showing the app on Linux.

### Dependencies

- Upgraded to `tauri-cli@2.2.1`

## \[2.2.0]

### New Features

- [`cccb308c7`](https://www.github.com/tauri-apps/tauri/commit/cccb308c7b559b0838138d6cea280665f060c925) ([#11562](https://www.github.com/tauri-apps/tauri/pull/11562) by [@jLynx](https://www.github.com/tauri-apps/tauri/../../jLynx)) Generate signature for `.deb` packages when `createUpdaterArtifacts` option is enabled.
- [`74212d40d`](https://www.github.com/tauri-apps/tauri/commit/74212d40d80dba4501b3d4ae30104fa3d447bdf9) ([#11653](https://www.github.com/tauri-apps/tauri/pull/11653) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Include Linux destkop environment and session type in `tauri info` command.

### Enhancements

- [`93a3a043d`](https://www.github.com/tauri-apps/tauri/commit/93a3a043d39cc96515d51d98beeb14261d3a246b) ([#11727](https://www.github.com/tauri-apps/tauri/pull/11727) by [@Kiyozz](https://www.github.com/tauri-apps/tauri/../../Kiyozz)) Add support for `Portuguese` language for NSIS windows installer.

### Bug Fixes

- [`c8700656b`](https://www.github.com/tauri-apps/tauri/commit/c8700656be3001a0cc6e087f23aebd482430a85b) ([#11985](https://www.github.com/tauri-apps/tauri/pull/11985) by [@ShaunSHamilton](https://www.github.com/tauri-apps/tauri/../../ShaunSHamilton)) Fix `tauri remove` from removing object type (`{}`) permissions.
- [`0ae06c5ca`](https://www.github.com/tauri-apps/tauri/commit/0ae06c5ca89cecd24154affdc69668f5e1e67d85) ([#11914](https://www.github.com/tauri-apps/tauri/pull/11914) by [@wtto00](https://www.github.com/tauri-apps/tauri/../../wtto00)) Fix the exclude path in file `Cargo.toml` of plugin template generated by cli. Path changed in [#9346](https://github.com/tauri-apps/tauri/pull/9346)

### Dependencies

- Upgraded to `tauri-cli@2.2.0`

## \[2.1.0]

### New Features

- [`6bf917941`](https://www.github.com/tauri-apps/tauri/commit/6bf917941ff0fcc49e86b3ba427340b75f3ce49c) ([#11322](https://www.github.com/tauri-apps/tauri/pull/11322) by [@ShaunSHamilton](https://www.github.com/tauri-apps/tauri/../../ShaunSHamilton)) Add `tauri remove` to remove plugins from projects.
- [`058c0db72`](https://www.github.com/tauri-apps/tauri/commit/058c0db72f43fbe1574d0db654560e693755cd7e) ([#11584](https://www.github.com/tauri-apps/tauri/pull/11584) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `bundle > linux > rpm > compression` config option to control RPM bundle compression type and level.

### Enhancements

- [`1f311832a`](https://www.github.com/tauri-apps/tauri/commit/1f311832ab5b2d62a533dfcf9b1d78bddf249ae8) ([#11405](https://www.github.com/tauri-apps/tauri/pull/11405) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add more context for errors when decoding secret and public keys for signing updater artifacts.
- [`e0d1307d3`](https://www.github.com/tauri-apps/tauri/commit/e0d1307d3f78987d0059921a5ab01ea4b26e0ef1) ([#11414](https://www.github.com/tauri-apps/tauri/pull/11414) by [@Czxck001](https://www.github.com/tauri-apps/tauri/../../Czxck001)) Migrate the `$schema` Tauri configuration to the v2 format.
- [`c43d5df15`](https://www.github.com/tauri-apps/tauri/commit/c43d5df15828ecffa606482ea2b60350c488c981) ([#11512](https://www.github.com/tauri-apps/tauri/pull/11512) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Associate a newly created capability file with the `main` window on the `tauri add` and `tauri permission add` commands.

### Bug Fixes

- [`7af01ff2c`](https://www.github.com/tauri-apps/tauri/commit/7af01ff2ce623d727cd13a4c8a549c1c80031882) ([#11523](https://www.github.com/tauri-apps/tauri/pull/11523) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `tauri migrate` failing to install NPM depenencies when running from Deno.
- [`100a4455a`](https://www.github.com/tauri-apps/tauri/commit/100a4455aa48df508510bbc08273215bdf70c012) ([#11529](https://www.github.com/tauri-apps/tauri/pull/11529) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix detecting yarn berry (v2 and higher) in various tauri cli commands.
- [`60e86d5f6`](https://www.github.com/tauri-apps/tauri/commit/60e86d5f6e0f0c769d34ef368cd8801a918d796d) ([#11624](https://www.github.com/tauri-apps/tauri/pull/11624) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Use the public network IP address on `android dev` by default on Windows.

### Dependencies

- Upgraded to `tauri-cli@2.1.0`

## \[2.0.4]

### Enhancements

- [`e4c9268b1`](https://www.github.com/tauri-apps/tauri/commit/e4c9268b19c614dc9ebb0895448fd16de7efee80) ([#11258](https://www.github.com/tauri-apps/tauri/pull/11258) by [@regexident](https://www.github.com/tauri-apps/tauri/../../regexident)) Support custom project directory structure where the Tauri app folder is not a subfolder of the frontend project.
  The frontend and Tauri app project paths can be set with the `TAURI_FRONTEND_PATH` and the `TAURI_APP_PATH` environment variables respectively.

### Dependencies

- Upgraded to `tauri-cli@2.0.4`

## \[2.0.3]

### New Features

- [`eda5713ea`](https://www.github.com/tauri-apps/tauri/commit/eda5713eab78d28182071ea25ceca5f1994f37ea) ([#11242](https://www.github.com/tauri-apps/tauri/pull/11242) by [@alex-sandri](https://www.github.com/tauri-apps/tauri/../../alex-sandri)) Add `Italian` to supported NSIS installer languages
- [`b3563e3d6`](https://www.github.com/tauri-apps/tauri/commit/b3563e3d6ae8dc90ee68f25f575cd5538ab1915b) ([#11304](https://www.github.com/tauri-apps/tauri/pull/11304) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add Deno support in tauri-cli operations.

### Bug Fixes

- [`d609bef9f`](https://www.github.com/tauri-apps/tauri/commit/d609bef9fd7cd6eeb2bd701558100bd9cfb6e6f6) ([#11314](https://www.github.com/tauri-apps/tauri/pull/11314) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix android invalid proguard file when using an `identifier` that contains a component that is a reserved kotlin keyword, like `in`, `class`, etc
- [`069c05e44`](https://www.github.com/tauri-apps/tauri/commit/069c05e44fd6f30083fdc00dd6c0001278898592) ([#11315](https://www.github.com/tauri-apps/tauri/pull/11315) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix CLI crashing and failing to find a `.ico` file when `bundle > icon` option is using globs and doesn't have a string that ends with `.ico`.

### Dependencies

- Upgraded to `tauri-cli@2.0.3`

## \[2.0.2]

### What's Changed

- [`4475fbb50`](https://www.github.com/tauri-apps/tauri/commit/4475fbb502c5ffb3cea4de6bef1c7869be39bed6) ([#11208](https://www.github.com/tauri-apps/tauri/pull/11208) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Update cargo-mobile2 to 0.17.3, fixing lib name validation.
- [`a49a19ffa`](https://www.github.com/tauri-apps/tauri/commit/a49a19ffa304f031fb1a04d31a567cc7f42a380a) ([#11218](https://www.github.com/tauri-apps/tauri/pull/11218)) Fix bundling `appimage`, `deb` and `rpm` bundles failing to open when using `mainBinaryName` with spaces.

### Dependencies

- Upgraded to `tauri-cli@2.0.2`

## \[2.0.1]

### Dependencies

- Upgraded to `tauri-cli@2.0.1`

## \[2.0.0]

### What's Changed

- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

### Dependencies

- Upgraded to `tauri-cli@2.0.0`

## \[2.0.0-rc.18]

### Enhancements

- [`a08e6ffa6`](https://www.github.com/tauri-apps/tauri/commit/a08e6ffa6fe499553be3c4c620726d6031cd6dd3) ([#11185](https://www.github.com/tauri-apps/tauri/pull/11185) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Enhance port forwarding on `android dev` to be more resilient and tolerate delays when booting up devices.
- [`6cfe7edf6`](https://www.github.com/tauri-apps/tauri/commit/6cfe7edf63636fdf66c429efdeb7bc9a0f404e9f) ([#11186](https://www.github.com/tauri-apps/tauri/pull/11186) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Retain logger verbosity on the `android-studio-script` and `xcode-script` commands.
- [`60a5aea53`](https://www.github.com/tauri-apps/tauri/commit/60a5aea53db02ae6af325812ab97555f2c013d70) ([#11181](https://www.github.com/tauri-apps/tauri/pull/11181) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Set the `TRUNK_SERVE_ADDRESS` environment variable when running on iOS physical devices to support Trunk.

### Bug Fixes

- [`f5d61822b`](https://www.github.com/tauri-apps/tauri/commit/f5d61822bf5988827776dd58bed75c19364e86bd) ([#11184](https://www.github.com/tauri-apps/tauri/pull/11184) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix iOS application not including the provided capabilities (entitlements).

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.18`

## \[2.0.0-rc.17]

### New Features

- [`a944b9b05`](https://www.github.com/tauri-apps/tauri/commit/a944b9b05bc5ae6125ff451e86c5b207c511f3d7) ([#11118](https://www.github.com/tauri-apps/tauri/pull/11118) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `--github-workflows` flag for `tauri plugin new/init`.
- [`f57a729cd`](https://www.github.com/tauri-apps/tauri/commit/f57a729cd8f7e10d8daf0b9d5b85f9c7ad530496) ([#11039](https://www.github.com/tauri-apps/tauri/pull/11039) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `tauri inspect wix-upgrade-code` to print default Upgrade Code for your MSI installer derived from `productName`.

### Bug Fixes

- [`62b52f60a`](https://www.github.com/tauri-apps/tauri/commit/62b52f60a22ef84c4a2a2d9e662038b49f58e16c) ([#11064](https://www.github.com/tauri-apps/tauri/pull/11064) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `tauri add` failing to add NPM depenency with `npm` package manager.
- [`56e087471`](https://www.github.com/tauri-apps/tauri/commit/56e087471a347f6bee7422221a956925c60b17e3) ([#11100](https://www.github.com/tauri-apps/tauri/pull/11100) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix iOS xcode-script usage with `bun`.
- [`b88e22a5f`](https://www.github.com/tauri-apps/tauri/commit/b88e22a5fe4e2e4376d6cad64d1e74d104ca8927) ([#11063](https://www.github.com/tauri-apps/tauri/pull/11063) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) The cli now only sets the iOS deployment target environment variable when building for iOS.
- [`8d22c0c81`](https://www.github.com/tauri-apps/tauri/commit/8d22c0c814e7227d5e56ce9a08929045ccea1a1b) ([#11101](https://www.github.com/tauri-apps/tauri/pull/11101) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Only modify the iOS Xcode project "sign style" if we need to enforce manual signing.
- [`df24cb944`](https://www.github.com/tauri-apps/tauri/commit/df24cb944249ee398f6c8ba8c19757b398eec701) ([#11168](https://www.github.com/tauri-apps/tauri/pull/11168) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes Xcode pbxproj file parsing not expecting `_` in build configuration IDs.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.17`

### Breaking Changes

- [`a944b9b05`](https://www.github.com/tauri-apps/tauri/commit/a944b9b05bc5ae6125ff451e86c5b207c511f3d7) ([#11118](https://www.github.com/tauri-apps/tauri/pull/11118) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) `tauri plugin init/new` will no longer generate a `.github` directory with workflows by default, instead use the new `--github-workflows` flag.

## \[2.0.0-rc.16]

### New Features

- [`9bb8fc618`](https://www.github.com/tauri-apps/tauri/commit/9bb8fc6189a93bcb811588b36e710d0f7818a1f9) ([#11030](https://www.github.com/tauri-apps/tauri/pull/11030) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `--no-example` flag for `tauri plugin new` and `tauri plugin init` to disable creation of an example project.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.16`

## \[2.0.0-rc.15]

### Enhancements

- [`5a0e922d4`](https://www.github.com/tauri-apps/tauri/commit/5a0e922d40dc3b7d9a8e3a65ccaf76d09f026cb8) ([#11007](https://www.github.com/tauri-apps/tauri/pull/11007) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Automatically discover the `src-tauri/src/main.rs` binary when it is not explicitly defined in the Cargo manifest bin array.

### Bug Fixes

- [`94e9d476e`](https://www.github.com/tauri-apps/tauri/commit/94e9d476ef506b1b8c09f55b81620c7839f98086) ([#11011](https://www.github.com/tauri-apps/tauri/pull/11011) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `main_binary_name` in custom wix and nsis templates including `.exe`

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.15`

## \[2.0.0-rc.14]

### Enhancements

- [`6c5340f8b`](https://www.github.com/tauri-apps/tauri/commit/6c5340f8b2549dfe89f19656304e65cd670afc92) ([#11004](https://www.github.com/tauri-apps/tauri/pull/11004) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added the `log` plugin to the app template, which is required to visualize logs on Android and iOS.
- [`3ad2427dc`](https://www.github.com/tauri-apps/tauri/commit/3ad2427dc08f12c61edc726b587acce32eca1080) ([#10961](https://www.github.com/tauri-apps/tauri/pull/10961) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Only render app logs on iOS unless `-vv` is provided to the `ios dev` command.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.13`

## \[2.0.0-rc.13]

### Bug Fixes

- [`a5848af65`](https://www.github.com/tauri-apps/tauri/commit/a5848af65b10d89686314cf737b7fd9d91f99dd8) ([#10944](https://www.github.com/tauri-apps/tauri/pull/10944) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Synchronize app version (`tauri.conf.json > version` or `Cargo.toml > package > version`) with the `CFBundleVersion` and `CFBundleShortVersionString` Info.plist values.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.12`

## \[2.0.0-rc.12]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.11`

## \[2.0.0-rc.11]

### Enhancements

- [`9c9644d15`](https://www.github.com/tauri-apps/tauri/commit/9c9644d155818d9efcad65b60aa985a59e767922) ([#10845](https://www.github.com/tauri-apps/tauri/pull/10845) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Enhance iOS library validation, checking libs built with link time optimization.

### Bug Fixes

- [`b42683592`](https://www.github.com/tauri-apps/tauri/commit/b42683592d446f25c2005b59e9e3ec551175906d) ([#10847](https://www.github.com/tauri-apps/tauri/pull/10847) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes `ios build --target [aarch64-sim | x86_64]` failing to generate the app bundle for the iOS simulator.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.10`

## \[2.0.0-rc.10]

### Bug Fixes

- [`6faa03276`](https://www.github.com/tauri-apps/tauri/commit/6faa032766b23cd161503905d4c79365ff6c50d1) ([#10854](https://www.github.com/tauri-apps/tauri/pull/10854) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes iOS code signing failing on CI due to a missing development certificate.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.9`

## \[2.0.0-rc.9]

### Bug Fixes

- [`5af1f5dec`](https://www.github.com/tauri-apps/tauri/commit/5af1f5dec1bb98f335169df8c5e30c19a24cae07) ([#10851](https://www.github.com/tauri-apps/tauri/pull/10851) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes `ios build` failing to build iOS app in CI when using an API key for automatic signing.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.9`

## \[2.0.0-rc.8]

### New Features

- [`91e9e784a`](https://www.github.com/tauri-apps/tauri/commit/91e9e784aa59634e3fe6359f8b78d071d76a9e42) ([#10729](https://www.github.com/tauri-apps/tauri/pull/10729) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add plugins information in `tauri info` output
- [`09e9dc1aa`](https://www.github.com/tauri-apps/tauri/commit/09e9dc1aab1b66aa6a3a009d5873db586abe76a0) ([#10752](https://www.github.com/tauri-apps/tauri/pull/10752) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Allow Xcode to manage iOS code sign and provisioning profiles by default.
  On CI, the `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_KEY_PATH` environment variables must be provided for authentication.

### Enhancements

- [`3a4972b39`](https://www.github.com/tauri-apps/tauri/commit/3a4972b394c65c32eefebfb2181ba56b0cfc08f7) ([#10793](https://www.github.com/tauri-apps/tauri/pull/10793) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Include architecture in the `tauri info` output.
- [`fd68b7fde`](https://www.github.com/tauri-apps/tauri/commit/fd68b7fdea3890d9f0a373a252a3682bd9d04138) ([#10785](https://www.github.com/tauri-apps/tauri/pull/10785) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Remove the `.cargo/config` file creation that used to fix mobile build caches.
- [`f67a9eb6d`](https://www.github.com/tauri-apps/tauri/commit/f67a9eb6de4567c2374b8cdbabadcf0ca44d28fb) ([#10802](https://www.github.com/tauri-apps/tauri/pull/10802) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Synchronize identifier, development team and lib name with the iOS Xcode project.

### Bug Fixes

- [`83ed090bf`](https://www.github.com/tauri-apps/tauri/commit/83ed090bfa58a1784495f474d93b16a568be513f) ([#10790](https://www.github.com/tauri-apps/tauri/pull/10790) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Do not quit `ios dev` and `android dev` process when we fail to attach the logger.
- [`2d31aef75`](https://www.github.com/tauri-apps/tauri/commit/2d31aef759f496f3afe46b7697176e61a8570511) ([#10751](https://www.github.com/tauri-apps/tauri/pull/10751) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Ensure gradlew is executable and does not use CRLF so it can be used on UNIX systems.
- [`02b2f964a`](https://www.github.com/tauri-apps/tauri/commit/02b2f964a70c61ff08b5052bd9fcde472d706d9c) ([#10795](https://www.github.com/tauri-apps/tauri/pull/10795) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix the `add` command NPM version specifier for known plugins from `2.0.0-rc` (unknown version requirement) to `^2.0.0-rc`.
- [`84070bae9`](https://www.github.com/tauri-apps/tauri/commit/84070bae92d234bc3630e795cfaf79f869f3a751) ([#10792](https://www.github.com/tauri-apps/tauri/pull/10792) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix `tauri plugin ios init` not generating the iOS folder.
- [`edb2ca31f`](https://www.github.com/tauri-apps/tauri/commit/edb2ca31f70a39004b6a09ae53425f22e243318e) ([#10794](https://www.github.com/tauri-apps/tauri/pull/10794) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Migrate v1 plugins NPM packages.
- [`9718dc9e8`](https://www.github.com/tauri-apps/tauri/commit/9718dc9e8c9bc91d9a5d9e0e06a7afab62492152) ([#10791](https://www.github.com/tauri-apps/tauri/pull/10791) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Reintroduce the `targetSdk` value in the Android application template.

### What's Changed

- [`fb6bf3142`](https://www.github.com/tauri-apps/tauri/commit/fb6bf314252c88dd49af74bdbb8499df370836ae) ([#10763](https://www.github.com/tauri-apps/tauri/pull/10763) by [@rdlabo](https://www.github.com/tauri-apps/tauri/../../rdlabo)) Update plugin template Android code to match documentation on Android package ID usage.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.8`

### Breaking Changes

- [`073bb4f45`](https://www.github.com/tauri-apps/tauri/commit/073bb4f459a923541b94970dfa7e087bccaa2cfd) ([#10772](https://www.github.com/tauri-apps/tauri/pull/10772) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Removed the deprecated `webview_fixed_runtime_path` config option, use the `webview_install_mode` instead.

## \[2.0.0-rc.7]

### Enhancements

- [`da8c9a7d3`](https://www.github.com/tauri-apps/tauri/commit/da8c9a7d3069398c26826aeb082caa44b7c92809) ([#10669](https://www.github.com/tauri-apps/tauri/pull/10669) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Modify both ExportOptions.plist and project.pbxproj to reflect changes for the `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD` and `IOS_MOBILE_PROVISION` environment variables.

### Bug Fixes

- [`793ee0531`](https://www.github.com/tauri-apps/tauri/commit/793ee0531730597e6008c9c0dedabbab7a2bef53) ([#10700](https://www.github.com/tauri-apps/tauri/pull/10700) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Allow hyphens and underscores on app identifiers.
- [`da8c9a7d3`](https://www.github.com/tauri-apps/tauri/commit/da8c9a7d3069398c26826aeb082caa44b7c92809) ([#10669](https://www.github.com/tauri-apps/tauri/pull/10669) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Synchronize Xcode project changes with the ExportOptions.plist file so `ios build` calls can work with code signing changes made in Xcode.

### What's Changed

- [`f4d5241b3`](https://www.github.com/tauri-apps/tauri/commit/f4d5241b377d0f7a1b58100ee19f7843384634ac) ([#10731](https://www.github.com/tauri-apps/tauri/pull/10731) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Update documentation icon path.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.7`

### Breaking Changes

- [`da8c9a7d3`](https://www.github.com/tauri-apps/tauri/commit/da8c9a7d3069398c26826aeb082caa44b7c92809) ([#10669](https://www.github.com/tauri-apps/tauri/pull/10669) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) The `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD` and `IOS_MOBILE_PROVISION` environment variables are now read by the `ios build` command instead of `ios init`.

## \[2.0.0-rc.6]

### New Features

- [`da381e07f`](https://www.github.com/tauri-apps/tauri/commit/da381e07f3770988fe6d0859a02331b87cc6723f) ([#10696](https://www.github.com/tauri-apps/tauri/pull/10696) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Inject configured resources on mobile apps.

### Bug Fixes

- [`1a60822a4`](https://www.github.com/tauri-apps/tauri/commit/1a60822a4220b6dbb1ad7295a2e37d6c3004edad) ([#10699](https://www.github.com/tauri-apps/tauri/pull/10699) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Changed the `add` command to use a version requirement that matches the CLI's stable and prerelease numbers.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.6`

## \[2.0.0-rc.5]

### New Features

- [`8d148a9e2`](https://www.github.com/tauri-apps/tauri/commit/8d148a9e2566edebfea2d75f32df7c9396d765a4) ([#10634](https://www.github.com/tauri-apps/tauri/pull/10634) by [@anatawa12](https://www.github.com/tauri-apps/tauri/../../anatawa12)) Custom sign command with object notation for whitespaces in the command path and arguments.

### Bug Fixes

- [`8ae52a615`](https://www.github.com/tauri-apps/tauri/commit/8ae52a615a11d934930001da63ce6ac8442c7efc) ([#10676](https://www.github.com/tauri-apps/tauri/pull/10676) by [@rdlabo](https://www.github.com/tauri-apps/tauri/../../rdlabo)) Change plugin template call to `register_ios_plugin` params to snake case
- [`7796a8fc6`](https://www.github.com/tauri-apps/tauri/commit/7796a8fc649cd7397a67048c71f8d1fbf822122a) ([#10687](https://www.github.com/tauri-apps/tauri/pull/10687) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix Swift plugin compilation on older versions.
- [`9b99ebab1`](https://www.github.com/tauri-apps/tauri/commit/9b99ebab17d6a043d82a7aeecfb76c56a995c287) ([#10431](https://www.github.com/tauri-apps/tauri/pull/10431) by [@mrguiman](https://www.github.com/tauri-apps/tauri/../../mrguiman)) Do not include the target arch when building and archiving the iOS application,
  which makes Xcode project modifications more flexible.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.5`

## \[2.0.0-rc.4]

### New Features

- [`78e22bedc`](https://www.github.com/tauri-apps/tauri/commit/78e22bedcab5096f1a4e667321fc8b2817b79214) ([#10602](https://www.github.com/tauri-apps/tauri/pull/10602) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add necessary options to `AndroidManifest.xml` in android template to support AndroidTV.
- [`3bec7b159`](https://www.github.com/tauri-apps/tauri/commit/3bec7b1595e28630a22b9fb16540beafd5eb7969) ([#10544](https://www.github.com/tauri-apps/tauri/pull/10544) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) v1 migrate script now migrates Svelte and Vue.js code.

### Enhancements

- [`bba1a4419`](https://www.github.com/tauri-apps/tauri/commit/bba1a441917defcdf9e88221e9b0e1cdd744e77a) ([#10457](https://www.github.com/tauri-apps/tauri/pull/10457) by [@mmvanheusden](https://www.github.com/tauri-apps/tauri/../../mmvanheusden)) Added `--no-fmt` option to the `add` command to skip formatting the code after applying changes.
- [`71d00646a`](https://www.github.com/tauri-apps/tauri/commit/71d00646a9b7c52311ba087820e52fd19861b3d8) ([#10504](https://www.github.com/tauri-apps/tauri/pull/10504) by [@fu050409](https://www.github.com/tauri-apps/tauri/../../fu050409)) Improve the `init` command behavior by detecting the project NPM package manager.
- [`8deb1966a`](https://www.github.com/tauri-apps/tauri/commit/8deb1966ace93d1350f271d525a878ba4b0879ce) ([#10652](https://www.github.com/tauri-apps/tauri/pull/10652) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Infer macOS codesign identity from the `APPLE_CERTIFICATE` environment variable when provided, meaning the identity no longer needs to be provided when signing on CI using that option. If the imported certificate name does not match a provided signingIdentity configuration, an error is returned.
- [`f35bcda28`](https://www.github.com/tauri-apps/tauri/commit/f35bcda2895a1350df31853da76a051783b9fd3f) ([#10598](https://www.github.com/tauri-apps/tauri/pull/10598) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) `permission add` and `add` commands now check if the plugin is known and if it is either desktop or mobile only
  we add the permission to a target-specific capability.

### Bug Fixes

- [`f712f31d1`](https://www.github.com/tauri-apps/tauri/commit/f712f31d1d21e85fab99194530702c70e45c63fc) ([#10639](https://www.github.com/tauri-apps/tauri/pull/10639) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Include notarization error output in the error message if it fails.
- [`9f75d0622`](https://www.github.com/tauri-apps/tauri/commit/9f75d06228fcb7036cf7a4e215abc7bc8d1a0a56) ([#10604](https://www.github.com/tauri-apps/tauri/pull/10604) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes `android dev` port forward failing under some conditions, add better logging and error handling.
- [`2d47352a0`](https://www.github.com/tauri-apps/tauri/commit/2d47352a07a7d742e62291a5e6810aed79fc8b50) ([#10418](https://www.github.com/tauri-apps/tauri/pull/10418) by [@samkearney](https://www.github.com/tauri-apps/tauri/../../samkearney)) CLI commands will now consistently search for the `app_dir` (the directory containing `package.json`) from the current working directory of the command invocation.
- [`f4cd68f04`](https://www.github.com/tauri-apps/tauri/commit/f4cd68f040635f019ff989667289cfe9061c7dfb) ([#10600](https://www.github.com/tauri-apps/tauri/pull/10600) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes `android dev` not working when using the builtin dev server.
- [`41c7a6646`](https://www.github.com/tauri-apps/tauri/commit/41c7a6646ba9afbb2322986fb39054a43a88e604) ([#10572](https://www.github.com/tauri-apps/tauri/pull/10572) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Exit with code 1 if a panic occurs when running the CLI with `bun`.
- [`9089d9763`](https://www.github.com/tauri-apps/tauri/commit/9089d97637e49bebbe7dba8adc6351e04b53a44d) ([#10605](https://www.github.com/tauri-apps/tauri/pull/10605) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes `[android|ios] build --config <config>` failing to resolve.
- [`712f1049f`](https://www.github.com/tauri-apps/tauri/commit/712f1049fae74bfda5f360adcee7210cea92fe63) ([#10569](https://www.github.com/tauri-apps/tauri/pull/10569) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes running `ios dev` and `ios build` using `bun`.
- [`3998570fd`](https://www.github.com/tauri-apps/tauri/commit/3998570fd3d03c1bb282bd060a4aafb4ab5437f9) ([#10540](https://www.github.com/tauri-apps/tauri/pull/10540) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes v1 migration of Cargo.toml dependencies and features.
- [`3beba92b5`](https://www.github.com/tauri-apps/tauri/commit/3beba92b5bdc62ed00c9f6a9b8f8c05cfa78f8dc) ([#10542](https://www.github.com/tauri-apps/tauri/pull/10542) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes v1 frontend code migration when using plugin default imports.
- [`10fb027b7`](https://www.github.com/tauri-apps/tauri/commit/10fb027b7590cf2c020b5c220328b9051c05adca) ([#10656](https://www.github.com/tauri-apps/tauri/pull/10656) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Migrate v1 plugins to their v2 releases.
- [`10fb027b7`](https://www.github.com/tauri-apps/tauri/commit/10fb027b7590cf2c020b5c220328b9051c05adca) ([#10656](https://www.github.com/tauri-apps/tauri/pull/10656) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Prevent duplicate permissions on v1 migration.
- [`b160f9359`](https://www.github.com/tauri-apps/tauri/commit/b160f9359d6f661d280185d2a2a4bdf280b8e72c) ([#10638](https://www.github.com/tauri-apps/tauri/pull/10638) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Only validate the output iOS library on debug builds.
- [`4bfe4880f`](https://www.github.com/tauri-apps/tauri/commit/4bfe4880fbef42d1a115f840e712d4a2f59c8ab3) ([#10550](https://www.github.com/tauri-apps/tauri/pull/10550) by [@anatawa12](https://www.github.com/tauri-apps/tauri/../../anatawa12)) fails to build universal fat binary if main bin is renamed to another name in `Cargo.toml`
- [`f3837d5b9`](https://www.github.com/tauri-apps/tauri/commit/f3837d5b98f0caebc3337f9a9e8127e7b96c3fc5) ([#10539](https://www.github.com/tauri-apps/tauri/pull/10539) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Improve migration tooling by supporting TOML configs, handle nulls and properly check for updater migration.

### What's Changed

- [`794cf8234`](https://www.github.com/tauri-apps/tauri/commit/794cf8234f8b620c74cbd23cc4b81be9b2edc386) ([#10571](https://www.github.com/tauri-apps/tauri/pull/10571) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Change iOS template default export method from deprecated `development` to `debugging`.
- [`bfc49cc7a`](https://www.github.com/tauri-apps/tauri/commit/bfc49cc7a1d43e3378e93865b9b37ce4bddfa6e6) ([#10558](https://www.github.com/tauri-apps/tauri/pull/10558) by [@ahqsoftwares](https://www.github.com/tauri-apps/tauri/../../ahqsoftwares)) Remove targetSdk from gradle files

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.4`

## \[2.0.0-rc.3]

### Enhancements

- [`5f56cb0a8`](https://www.github.com/tauri-apps/tauri/commit/5f56cb0a8b9c6f695bc6439a8db997c98b3a3997) ([#10507](https://www.github.com/tauri-apps/tauri/pull/10507) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Update gradle to 8.9 and the gradle android plugin to 8.5.1 in the android templates (requires latest Android Studio). This should add support for Java 21 but Java 17 keeps being the recommended version.

### Bug Fixes

- [`f5dfc0280`](https://www.github.com/tauri-apps/tauri/commit/f5dfc02800dbd3bdee671b032454c49ac7102fb4) ([#10533](https://www.github.com/tauri-apps/tauri/pull/10533) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue causing `tauri ios init` to fail if `iOS.minimumSystemVersion` was not configured explicitly.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.3`

## \[2.0.0-rc.2]

### New Features

- [`8dc81b6cc`](https://www.github.com/tauri-apps/tauri/commit/8dc81b6cc2b8235b11f74a971d6aa3a5df5e9f68) ([#10496](https://www.github.com/tauri-apps/tauri/pull/10496) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `bundle > ios > template` configuration option for custom Xcode project YML Handlebars template using XcodeGen.
- [`02c00abc6`](https://www.github.com/tauri-apps/tauri/commit/02c00abc63cf86e9bf9179cbb143d5145a9397b6) ([#10495](https://www.github.com/tauri-apps/tauri/pull/10495) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `bundle > ios > minimumSystemVersion` configuration option.

### Enhancements

- [`8e1e15304`](https://www.github.com/tauri-apps/tauri/commit/8e1e15304e9dc98d7f875fc8dceb7d4ce19adc47) ([#10483](https://www.github.com/tauri-apps/tauri/pull/10483) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Check if the Rust library contains the symbols required at runtime for Android and iOS apps.
- [`ca6868956`](https://www.github.com/tauri-apps/tauri/commit/ca68689564cbc8dfa9a5220d3daf81a44ef81fcc) ([#10479](https://www.github.com/tauri-apps/tauri/pull/10479) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Check if identifier or lib name changed when running mobile commands.

### Bug Fixes

- [`2e8ab7bac`](https://www.github.com/tauri-apps/tauri/commit/2e8ab7bac12046d734fb07a1b4fe5e03004b305e) ([#10481](https://www.github.com/tauri-apps/tauri/pull/10481) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Migration from v1 to v2 now adds the updater plugin when it is active.

### What's Changed

- [`a3cd9779a`](https://www.github.com/tauri-apps/tauri/commit/a3cd9779a47428e306a628d658740669faf69ccd) ([#10480](https://www.github.com/tauri-apps/tauri/pull/10480) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Removed the `[android|ios] open` command. It is recommended to use `[android|ios] dev --open` or `[android|ios] build --open` instead.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.2`

## \[2.0.0-rc.1]

### Bug Fixes

- [`fb1933f17`](https://www.github.com/tauri-apps/tauri/commit/fb1933f17442674e53374578e57a8cad241ac3c6) ([#10467](https://www.github.com/tauri-apps/tauri/pull/10467) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes running `android dev --open`.
- [`206914fe8`](https://www.github.com/tauri-apps/tauri/commit/206914fe8d97eb61a2ff2a80e94e65e7a42bcea5) ([#10466](https://www.github.com/tauri-apps/tauri/pull/10466) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes running `adb reverse` in Node.js context.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.1`

## \[2.0.0-rc.0]

### New Features

- [`d5511c311`](https://www.github.com/tauri-apps/tauri/commit/d5511c3117b1a117cb0b7359c5fa09aa4795122b) ([#10395](https://www.github.com/tauri-apps/tauri/pull/10395)) Added migration from `2.0.0-beta` to `2.0.0-rc`.
- [`a5bfbaa62`](https://www.github.com/tauri-apps/tauri/commit/a5bfbaa62b8cd0aacbb33f730d4e30b43c461fe1)([#9962](https://www.github.com/tauri-apps/tauri/pull/9962)) Added `bundle > iOS > frameworks` configuration to define a list of frameworks that are linked to the Xcode project when it is generated.

### Enhancements

- [`a0841d509`](https://www.github.com/tauri-apps/tauri/commit/a0841d509abc43b62bb7c755e8727f3f461862d1) ([#10421](https://www.github.com/tauri-apps/tauri/pull/10421)) Changes the default behavior of the `dev` command to only expose to localhost (`127.0.0.1`) instead of the default system interface.

### Security fixes

- [`289ae5555`](https://www.github.com/tauri-apps/tauri/commit/289ae5555da3802741018015bfe4927729a2eb33) ([#10386](https://www.github.com/tauri-apps/tauri/pull/10386)) Re-enable TLS checks that were previously disabled to support an insecure HTTPS custom protocol on Android which is no longer used.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-rc.0`

### Breaking Changes

- [`758d28c8a`](https://www.github.com/tauri-apps/tauri/commit/758d28c8a2d5c9567158e339326b765f72da983e) ([#10390](https://www.github.com/tauri-apps/tauri/pull/10390)) Core plugin permissions are now prefixed with `core:`, the `core:default` permission set can now be used and the `core` plugin name is reserved.
  The `tauri migrate` tool will automate the migration process, which involves prefixing all `app`, `event`, `image`, `menu`, `path`, `resources`, `tray`, `webview` and `window` permissions with `core:`.
- [`7ba67b4ac`](https://www.github.com/tauri-apps/tauri/commit/7ba67b4aca8d3f3b1aa5ad08819605029d36e6b4)([#10437](https://www.github.com/tauri-apps/tauri/pull/10437)) `ios dev` and `android dev` now uses localhost for the development server unless running on an iOS device,
  which still requires connecting to the public network address. To conditionally check this on your frontend
  framework's configuration you can check for the existence of the `TAURI_DEV_HOST`
  environment variable instead of checking if the target is iOS or Android (previous recommendation).

## \[2.0.0-beta.23]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.23`

## \[2.0.0-beta.22]

### New Features

- [`7c7fa0964`](https://www.github.com/tauri-apps/tauri/commit/7c7fa0964db3403037fdb9a34de2b877ddb8df1c) ([#9963](https://www.github.com/tauri-apps/tauri/pull/9963) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `--method` argument for `ios build` to select the export options' method.
- [`7c7fa0964`](https://www.github.com/tauri-apps/tauri/commit/7c7fa0964db3403037fdb9a34de2b877ddb8df1c) ([#9963](https://www.github.com/tauri-apps/tauri/pull/9963) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Setup iOS signing by reading `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD` and `IOS_MOBILE_PROVISION` environment variables.

### Enhancements

- [`c01e87ad4`](https://www.github.com/tauri-apps/tauri/commit/c01e87ad46e2a5b3fb8d018739e724ef932008d7) ([#10198](https://www.github.com/tauri-apps/tauri/pull/10198) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Enhance `tauri migrate` to also migrate variables like `appWindow`:

  ```ts
  import { appWindow } from '@tauri-apps/api/window'
  ```

  will become:

  ```ts
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
  const appWindow = getCurrentWebviewWindow()
  ```

### Bug Fixes

- [`94136578b`](https://www.github.com/tauri-apps/tauri/commit/94136578bc89e4b973c471050ae9c2d83ffcb7c6) ([#10186](https://www.github.com/tauri-apps/tauri/pull/10186) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `migrate` command, migrating incorrect permissions for `clipboard`.
- [`c01e87ad4`](https://www.github.com/tauri-apps/tauri/commit/c01e87ad46e2a5b3fb8d018739e724ef932008d7) ([#10198](https://www.github.com/tauri-apps/tauri/pull/10198) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `tauri migrate` incorrectly migrating `@tauri-apps/api/tauri` module to just `core` and `@tauri-apps/api/window` to just `webviewWindow`.
- [`15e125996`](https://www.github.com/tauri-apps/tauri/commit/15e12599667b749c3d7cd2259e6cf7c7b5c6e2be) ([#10234](https://www.github.com/tauri-apps/tauri/pull/10234) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix cli failing to detect the correct cargo target directory when using cargo `--target-dir` flag with `tauri build` or `tauri dev`

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.22`

## \[2.0.0-beta.21]

### New Features

- [`656a64974`](https://www.github.com/tauri-apps/tauri/commit/656a64974468bc207bf39537e02ae179bdee9b83) ([#9318](https://www.github.com/tauri-apps/tauri/pull/9318)) Added a configuration option to disable hardened runtime on macOS codesign.

### Enhancements

- [`f44a2ec47`](https://www.github.com/tauri-apps/tauri/commit/f44a2ec47c13243d472fa08a9df8b20d8490d79f) ([#10030](https://www.github.com/tauri-apps/tauri/pull/10030)) Enhance the plugin template to include `permissions/default.toml` and default capabilities file for the example application.

### Bug Fixes

- [`019a74e97`](https://www.github.com/tauri-apps/tauri/commit/019a74e970958d29cf69a6f24669d603399dcbb3) ([#9931](https://www.github.com/tauri-apps/tauri/pull/9931)) Fix wrong migration of `clipboard` and `globalShortcut` modules
- [`27838365a`](https://www.github.com/tauri-apps/tauri/commit/27838365a6841b0d3fa645ba2528221d23d4aeb2) ([#10135](https://www.github.com/tauri-apps/tauri/pull/10135)) Fix parsing of cargo profile when using `--profile=<profile>` syntax.
- [`79542f4d4`](https://www.github.com/tauri-apps/tauri/commit/79542f4d4542bd97451da7605de16e8464d6a06c) ([#10039](https://www.github.com/tauri-apps/tauri/pull/10039)) Fixed an issue that prevented `tauri icon` from rendering `<text>` nodes in SVG files.
- [`40c0f44e1`](https://www.github.com/tauri-apps/tauri/commit/40c0f44e1c74c18ed0d6c645724d650637725456) ([#9971](https://www.github.com/tauri-apps/tauri/pull/9971)) Changed the deployment target of plugin iOS Xcode project to 13.0 so it works on older iOS releases.
- [`f56cdc9e3`](https://www.github.com/tauri-apps/tauri/commit/f56cdc9e391c4d55e4d7e935203d0f891864f22d) ([#10016](https://www.github.com/tauri-apps/tauri/pull/10016)) Add missing dependency `libayatana-appindicator3.so.1` for rpm package.
- [`1601da5b5`](https://www.github.com/tauri-apps/tauri/commit/1601da5b525de05cb813002d611f22ea4217a4fb) ([#10114](https://www.github.com/tauri-apps/tauri/pull/10114)) Removed alpha channel from default icons in iOS template to comply with Apple's human interface guideline
  (https://developer.apple.com/design/human-interface-guidelines/app-icons), because
  transparent icons with alpha channel are not allowed, and will be rejected
  upon upload to Apple appstore.

### What's Changed

- [`3cca5c2be`](https://www.github.com/tauri-apps/tauri/commit/3cca5c2be88bbd52139e7dda371e88510d28bc8e) ([#9924](https://www.github.com/tauri-apps/tauri/pull/9924)) Migrate to new Android buildFeatures.buildConfig format.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.21`
- [`f955f7b49`](https://www.github.com/tauri-apps/tauri/commit/f955f7b4903bcea376c0a8b430736f66c8cebf56) ([#9929](https://www.github.com/tauri-apps/tauri/pull/9929)) Switch from `dirs_next` to `dirs` as `dirs_next` is now unmaintained while `dirs` is

### Breaking Changes

- [`911242f09`](https://www.github.com/tauri-apps/tauri/commit/911242f0928e0a2add3595fa9de27850fb875fa6) ([#9883](https://www.github.com/tauri-apps/tauri/pull/9883)) Move updater target from `bundle > targets` to a separate field `bundle > createUpdaterArtifacts`

## \[2.0.0-beta.20]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.20`

## \[2.0.0-beta.19]

### New Features

- [`8a1ae2dea`](https://www.github.com/tauri-apps/tauri/commit/8a1ae2deaf3086e531ada25b1627f900e2e421fb)([#9843](https://www.github.com/tauri-apps/tauri/pull/9843)) Added an option to use a Xcode project for the iOS plugin instead of a plain SwiftPM project.
- [`9e4b2253f`](https://www.github.com/tauri-apps/tauri/commit/9e4b2253f6ddaccd0f5c88734287bd5c84d4936a)([#9734](https://www.github.com/tauri-apps/tauri/pull/9734)) Add `tauri bundle` subcommand which runs the bundle phase only, best paired with `tauri build --no-bundle`

### Enhancements

- [`8b032c3cf`](https://www.github.com/tauri-apps/tauri/commit/8b032c3cf638e64e50df9d9cf8bc789c7e285987)([#9896](https://www.github.com/tauri-apps/tauri/pull/9896)) Add a blank LaunchScreen.storyboard to the iOS project init template to pass the App Store validation.
- [`9970d88be`](https://www.github.com/tauri-apps/tauri/commit/9970d88becee1560a4b2a7ffc1fe65991a42a8c9)([#9892](https://www.github.com/tauri-apps/tauri/pull/9892)) Update to latest gradle.

### What's Changed

- [`80aa50498`](https://www.github.com/tauri-apps/tauri/commit/80aa504987dd9cfa59aa5848c4d7960e1d58d0e6)([#9870](https://www.github.com/tauri-apps/tauri/pull/9870)) Updated Android target SDK to 34.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.19`

### Breaking Changes

- [`265c23886`](https://www.github.com/tauri-apps/tauri/commit/265c23886ee5efbcc6d7188ff5c84cb32fa82aea)([#9375](https://www.github.com/tauri-apps/tauri/pull/9375)) Avoid renaming main binary to product name and perserve the name generated by cargo.
- [`1df5cdeb0`](https://www.github.com/tauri-apps/tauri/commit/1df5cdeb06f5464e0eec4055e21b7b7bc8739eed)([#9858](https://www.github.com/tauri-apps/tauri/pull/9858)) Use `tauri.conf.json > identifier` to set the `PackageName` in Android and `BundleId` in iOS.

## \[2.0.0-beta.18]

### Bug Fixes

- [`beda18bce`](https://www.github.com/tauri-apps/tauri/commit/beda18bce95fd6e10543b2d8f1eca5fb7ca0655b)([#9855](https://www.github.com/tauri-apps/tauri/pull/9855)) Fixed an issue that caused `tauri add` to fail for multiple rust-only and platform-specific plugins.
- [`4a33bc6a6`](https://www.github.com/tauri-apps/tauri/commit/4a33bc6a62d2ed9371191c8a7f78ff3f33930455)([#9553](https://www.github.com/tauri-apps/tauri/pull/9553)) Fixes `pnpm` detection when initializing and running a mobile project.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.18`

## \[2.0.0-beta.17]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.17`

## \[2.0.0-beta.16]

### Bug Fixes

- [`97ec422f2`](https://www.github.com/tauri-apps/tauri/commit/97ec422f22d069b9570931834241c7e47bc68cc3)([#9638](https://www.github.com/tauri-apps/tauri/pull/9638)) Exit `tauri icon` with non-zero code when it fails.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.16`

## \[2.0.0-beta.15]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.15`

## \[2.0.0-beta.14]

### Enhancements

- [`8a63ceb4f`](https://www.github.com/tauri-apps/tauri/commit/8a63ceb4f31c422311b0f7dff173a9c8c0e1a604)([#9473](https://www.github.com/tauri-apps/tauri/pull/9473)) Ignore `.DS_Store` by default for `tauri dev` hot reloads.

### Bug Fixes

- [`e64b8f1dc`](https://www.github.com/tauri-apps/tauri/commit/e64b8f1dcedad3222f46755bf6f30392a7ec2f90)([#9479](https://www.github.com/tauri-apps/tauri/pull/9479)) Upgrade `heck` to v0.5 to better support Chinese and Japanese product name, because Chinese do not have word separation.
- [`aaa332c6e`](https://www.github.com/tauri-apps/tauri/commit/aaa332c6e78c956debd11efda021a0406621a01d)([#9540](https://www.github.com/tauri-apps/tauri/pull/9540)) Fix `tauri migrate` trying to migrate to a non-existing plugin.
- [`e64b8f1dc`](https://www.github.com/tauri-apps/tauri/commit/e64b8f1dcedad3222f46755bf6f30392a7ec2f90)([#9479](https://www.github.com/tauri-apps/tauri/pull/9479)) Fixed an issue causing the `build.runner` and `build.features` configs to not take effect.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.14`

## \[2.0.0-beta.13]

### Bug Fixes

- [`73c1c2d33`](https://www.github.com/tauri-apps/tauri/commit/73c1c2d33872651c32c761c838714b684980c668)([#9457](https://www.github.com/tauri-apps/tauri/pull/9457)) Gracefully handle Non-UTF8 files when using `tauri migrate`
- [`9331435a5`](https://www.github.com/tauri-apps/tauri/commit/9331435a50cc3769720bd2671da8510699d28671)([#9412](https://www.github.com/tauri-apps/tauri/pull/9412)) Fix `tauri info` crashing when Node.js is not installed.

### What's Changed

- [`8f4b1050c`](https://www.github.com/tauri-apps/tauri/commit/8f4b1050c4de0e9194680408ff3a6902b67045f8)([#9459](https://www.github.com/tauri-apps/tauri/pull/9459)) Show full expected path of `frontendDist` when if can't be found.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.13`

## \[2.0.0-beta.12]

### New Features

- [`93e0e1392`](https://www.github.com/tauri-apps/tauri/commit/93e0e1392ec341fcadf696c03e78f0ca1e73c941) Support specifying a version for `tauri add` subcommand, for example: `tauri add window-state@2.0.0-beta.2`

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.12`

## \[2.0.0-beta.11]

### Enhancements

- [`ac76a22f3`](https://www.github.com/tauri-apps/tauri/commit/ac76a22f383028d9bacdedebeb41d3fca5ec9dac)([#9183](https://www.github.com/tauri-apps/tauri/pull/9183)) Allow empty responses for `devUrl`, `beforeDevCommand` and `beforeBuildCommands` questions in `tauri init`.
- [`b525ddadf`](https://www.github.com/tauri-apps/tauri/commit/b525ddadf7e7588c3e195cf0f821c9862c545d06)([#9237](https://www.github.com/tauri-apps/tauri/pull/9237)) `openssl` is no longer a required dependency on macOS.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.11`

## \[2.0.0-beta.10]

### New Features

- [`7213b9e47`](https://www.github.com/tauri-apps/tauri/commit/7213b9e47242bef814aa7257e0bf84631bf5fe7e)([#9124](https://www.github.com/tauri-apps/tauri/pull/9124)) Add default permission for a plugin to capabilities when using `tauri add <plugin>`.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.10`

## \[2.0.0-beta.9]

### Bug Fixes

- [`c3ea3a2b7`](https://www.github.com/tauri-apps/tauri/commit/c3ea3a2b7d2fe3085f05b63dd1feb962beb4b7b3)([#9126](https://www.github.com/tauri-apps/tauri/pull/9126)) Fix bundling when `plugins > updater > windows > installerArgs` are set in `tauri.conf.json`

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.9`

## \[2.0.0-beta.8]

### Enhancements

- [`3e472d0af`](https://www.github.com/tauri-apps/tauri/commit/3e472d0afcd67545dd6d9f18d304580a3b2759a8)([#9115](https://www.github.com/tauri-apps/tauri/pull/9115)) Changed the permission and capability platforms to be optional.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.8`

## \[2.0.0-beta.7]

### Enhancements

- [`c68218b36`](https://www.github.com/tauri-apps/tauri/commit/c68218b362c417b62e56c7a2b5b32c13fe035a83)([#8990](https://www.github.com/tauri-apps/tauri/pull/8990)) Add `--no-bundle` flag for `tauri build` command to skip bundling. Previously `none` was used to skip bundling, it will now be treated as invalid format and a warning will be emitted instead.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.7`

## \[2.0.0-beta.6]

### Bug Fixes

- [`f5f3ed5f`](https://www.github.com/tauri-apps/tauri/commit/f5f3ed5f6faa0b51e83244acc15e9006299a03ba)([#9009](https://www.github.com/tauri-apps/tauri/pull/9009)) Fixes Android and iOS project initialization when the Tauri CLI is on a different disk partition.
- [`d7d03c71`](https://www.github.com/tauri-apps/tauri/commit/d7d03c7197212f3a5bebe08c929417d60927eb89)([#9017](https://www.github.com/tauri-apps/tauri/pull/9017)) Fixes dev watcher on mobile dev.
- [`b658ded6`](https://www.github.com/tauri-apps/tauri/commit/b658ded614cfc169228cb22ad5bfc64478dfe161)([#9015](https://www.github.com/tauri-apps/tauri/pull/9015)) Fixes truncation of existing BuildTask.kt when running `tauri android init`.

### What's Changed

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Updates to new ACL manifest path.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.6`

## \[2.0.0-beta.5]

### New Features

- [`06d63d67`](https://www.github.com/tauri-apps/tauri/commit/06d63d67a061459dd533ddcae755922427a6dfc5)([#8827](https://www.github.com/tauri-apps/tauri/pull/8827)) Add new subcommands for managing permissions and cababilities:

  - `tauri permission new`
  - `tauri permission add`
  - `tauri permission rm`
  - `tauri permission ls`
  - `tauri capability new`

### Breaking Changes

- [`b9e6a018`](https://www.github.com/tauri-apps/tauri/commit/b9e6a01879d9233040f3d3fab11c59e70563da7e)([#8937](https://www.github.com/tauri-apps/tauri/pull/8937)) The `custom-protocol` Cargo feature is no longer required on your application and is now ignored. To check if running on production, use `#[cfg(not(dev))]` instead of `#[cfg(feature = "custom-protocol")]`.

### Enhancements

- [`9be314f0`](https://www.github.com/tauri-apps/tauri/commit/9be314f07a4ca5d14433d41919492f3e91b5536a)([#8951](https://www.github.com/tauri-apps/tauri/pull/8951)) Add plugins to `Cargo.toml` when using `tauri migrate`

### Bug Fixes

- [`cbd9755e`](https://www.github.com/tauri-apps/tauri/commit/cbd9755e0926a7e47e59deb50f4bb93d621791a5)([#8977](https://www.github.com/tauri-apps/tauri/pull/8977)) Fixes process logs not showing on `ios dev`.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.5`

## \[2.0.0-beta.4]

### Bug Fixes

- [`e538ba58`](https://www.github.com/tauri-apps/tauri/commit/e538ba586c5b8b50955586c8ef2704adb5d7cc43)([#8949](https://www.github.com/tauri-apps/tauri/pull/8949)) Fixes android and iOS process spawning not working on Node.js.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.4`

### Breaking Changes

- [`a76fb118`](https://www.github.com/tauri-apps/tauri/commit/a76fb118ce2de22e1bdb4216bf0ac01dfc3e5799)([#8950](https://www.github.com/tauri-apps/tauri/pull/8950)) Changed the capability format to allow configuring both `remote: { urls: Vec<String> }` and `local: bool (default: true)` instead of choosing one on the `context` field.

## \[2.0.0-beta.3]

### Enhancements

- [`a029b9f7`](https://www.github.com/tauri-apps/tauri/commit/a029b9f77e432533a403c292940fa3efba68692c)([#8910](https://www.github.com/tauri-apps/tauri/pull/8910)) Setting up code signing is no longer required on iOS when using the simulator.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.3`

## \[2.0.0-beta.2]

### Enhancements

- [`83a68deb`](https://www.github.com/tauri-apps/tauri/commit/83a68deb5676d39cd4728d2e140f6b46d5f787ed)([#8797](https://www.github.com/tauri-apps/tauri/pull/8797)) Update app template following capabilities configuration change.

### Bug Fixes

- [`aa06a053`](https://www.github.com/tauri-apps/tauri/commit/aa06a0534cf224038866e0ddd6910ea873b2574d)([#8810](https://www.github.com/tauri-apps/tauri/pull/8810)) Fix `tauri plugin android init` printing invalid code that has a missing closing `"`.
- [`3cee26a5`](https://www.github.com/tauri-apps/tauri/commit/3cee26a58ab44639a12c7816f4096655daa327a4)([#8865](https://www.github.com/tauri-apps/tauri/pull/8865)) On Windows, fixed `tauri info` fails to detect the build tool when the system language is CJK.
- [`052e8b43`](https://www.github.com/tauri-apps/tauri/commit/052e8b4311d9f0f963a2866163be27bfd8f70c60)([#8838](https://www.github.com/tauri-apps/tauri/pull/8838)) Downgrade minisign dependency fixing updater signing key bug and prevent it from happening in the future.
- [`fb0d9971`](https://www.github.com/tauri-apps/tauri/commit/fb0d997117367e3387896bcd0fba004579475c40)([#8783](https://www.github.com/tauri-apps/tauri/pull/8783)) Fixes a regression on the `--config` argument not accepting file paths.
- [`baca704d`](https://www.github.com/tauri-apps/tauri/commit/baca704d4b5fae239fc320d10140f35bd705bfbb)([#8768](https://www.github.com/tauri-apps/tauri/pull/8768)) Do not migrate updater configuration if the active flag is set to false.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Enhancements

- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Moved the capability JSON schema to the `src-tauri/gen` folder so it's easier to track changes on the `capabilities` folder.
- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Update app and plugin templates following generated files change from tauri-build and tauri-plugin.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.1`

## \[2.0.0-beta.0]

### New Features

- [`7fcc0bcd`](https://www.github.com/tauri-apps/tauri/commit/7fcc0bcd3482bbc8771e330942ef6cd78cc8ec35)([#8490](https://www.github.com/tauri-apps/tauri/pull/8490)) Add plugin initialization rust code when using `tauri add`
- [`1878766f`](https://www.github.com/tauri-apps/tauri/commit/1878766f7f81a03b0f0b87ec33ee113d7aa7a902)([#8667](https://www.github.com/tauri-apps/tauri/pull/8667)) Migrate the allowlist config to the new capability file format.

### Enhancements

- [`d6c7568c`](https://www.github.com/tauri-apps/tauri/commit/d6c7568c27445653edf570f3969163bc358ba2ba)([#8720](https://www.github.com/tauri-apps/tauri/pull/8720)) Add `files` option to the AppImage Configuration.
- [`b3209bb2`](https://www.github.com/tauri-apps/tauri/commit/b3209bb28bb379d5046d577c7e42319d6e76ced0)([#8688](https://www.github.com/tauri-apps/tauri/pull/8688)) Ignore global `.gitignore` when searching for tauri directory.
- [`e691208e`](https://www.github.com/tauri-apps/tauri/commit/e691208e7b39bb8e3ffc9bf66cff731a5025ef16)([#7837](https://www.github.com/tauri-apps/tauri/pull/7837)) Prevent unneeded double Cargo.toml rewrite on `dev` and `build`.
- [`f492efd7`](https://www.github.com/tauri-apps/tauri/commit/f492efd7144fdd8d25cac0c4d2389a95ab75fb02)([#8666](https://www.github.com/tauri-apps/tauri/pull/8666)) Update app and plugin template following the new access control permission model.

### Bug Fixes

- [`9cb9aa79`](https://www.github.com/tauri-apps/tauri/commit/9cb9aa7978f231f7da238b33d6ab33fdd2d2c842)([#8672](https://www.github.com/tauri-apps/tauri/pull/8672)) Allow license field in Cargo.toml to be `{ workspace = true }`

### Dependencies

- Upgraded to `tauri-cli@2.0.0-beta.0`

### Breaking Changes

- [`8de308d1`](https://www.github.com/tauri-apps/tauri/commit/8de308d1bf6a855d7a26af58bd0e744938ba47d8)([#8723](https://www.github.com/tauri-apps/tauri/pull/8723)) Restructured Tauri config per [RFC#5](https://github.com/tauri-apps/rfcs/blob/f3e82a6b0c5390401e855850d47dc7b7d9afd684/texts/0005-tauri-config-restructure.md):

  - Moved `package.productName`, `package.version` and `tauri.bundle.identifier` fields to the top-level.
  - Removed `package` object.
  - Renamed `tauri` object to `app`.
  - Moved `tauri.bundle` object to the top-level.
  - Renamed `build.distDir` field to `frontendDist`.
  - Renamed `build.devPath` field to `devUrl` and will no longer accepts paths, it will only accept URLs.
  - Moved `tauri.pattern` to `app.security.pattern`.
  - Removed `tauri.bundle.updater` object, and its fields have been moved to the updater plugin under `plugins.updater` object.
  - Moved `build.withGlobalTauri` to `app.withGlobalTauri`.
  - Moved `tauri.bundle.dmg` object to `bundle.macOS.dmg`.
  - Moved `tauri.bundle.deb` object to `bundle.linux.deb`.
  - Moved `tauri.bundle.appimage` object to `bundle.linux.appimage`.
  - Removed all license fields from each bundle configuration object and instead added `bundle.license` and `bundle.licenseFile`.
  - Renamed `AppUrl` to `FrontendDist` and refactored its variants to be more explicit.

## \[2.0.0-alpha.21]

### New Features

- [`27bad32d`](https://www.github.com/tauri-apps/tauri/commit/27bad32d4d4acca8155b20225d529d540fb9aaf4)([#7798](https://www.github.com/tauri-apps/tauri/pull/7798)) Add `files` object on the `tauri > bundle > macOS` configuration option.
- [`0ec28c39`](https://www.github.com/tauri-apps/tauri/commit/0ec28c39f478de7199a66dd75e8642e1aa1344e6)([#8529](https://www.github.com/tauri-apps/tauri/pull/8529)) Include tauri-build on the migration script.

### Enhancements

- [`091100ac`](https://www.github.com/tauri-apps/tauri/commit/091100acbb507b51de39fb1446f685926f888fd2)([#5202](https://www.github.com/tauri-apps/tauri/pull/5202)) Add RPM packaging

### Bug Fixes

- [`4f73057e`](https://www.github.com/tauri-apps/tauri/commit/4f73057e6fd4c137bc112367fb91f5fc0c8a39f6)([#8486](https://www.github.com/tauri-apps/tauri/pull/8486)) Prevent `Invalid target triple` warnings and correctly set `TAURI_ENV_` vars when target triple contains 4 components.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.21`

### Breaking Changes

- [`4f73057e`](https://www.github.com/tauri-apps/tauri/commit/4f73057e6fd4c137bc112367fb91f5fc0c8a39f6)([#8486](https://www.github.com/tauri-apps/tauri/pull/8486)) Removed `TAURI_ENV_PLATFORM_TYPE` which will not be set for CLI hook commands anymore, use `TAURI_ENV_PLATFORM` instead. Also Changed value of `TAURI_ENV_PLATFORM` and `TAURI_ENV_ARCH` values to match the target triple more accurately:

  - `darwin` and `androideabi` are no longer replaced with `macos` and `android` in `TAURI_ENV_PLATFORM`.
  - `i686` and `i586` are no longer replaced with `x86` in `TAURI_ENV_ARCH`.

## \[2.0.0-alpha.20]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.20`

## \[2.0.0-alpha.19]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.19`

## \[2.0.0-alpha.18]

### New Features

- [`50f7ccbb`](https://www.github.com/tauri-apps/tauri/commit/50f7ccbbf3467f33cc7dd1cca53125fec6eda1c6)([#6444](https://www.github.com/tauri-apps/tauri/pull/6444)) Add suport to SVG input image for the `tauri icon` command.
- [`25e5f91d`](https://www.github.com/tauri-apps/tauri/commit/25e5f91dae7fe2bbc1ba4317d5d829402bfd1d50)([#8200](https://www.github.com/tauri-apps/tauri/pull/8200)) Merge `src-tauri/Info.plist` and `src-tauri/Info.ios.plist` with the iOS project plist file.

### Enhancements

- [`01a7a983`](https://www.github.com/tauri-apps/tauri/commit/01a7a983aba2946b455a608b8a6a4b08cb25fc11)([#8128](https://www.github.com/tauri-apps/tauri/pull/8128)) Transform paths to relative to the mobile project for the IDE script runner script.

### Bug Fixes

- [`88dac86f`](https://www.github.com/tauri-apps/tauri/commit/88dac86f3b301d1919df6473a9e20f46b560f29b)([#8149](https://www.github.com/tauri-apps/tauri/pull/8149)) Ensure `tauri add` prints `rust_code` with plugin name in snake case.
- [`977d0e52`](https://www.github.com/tauri-apps/tauri/commit/977d0e52f14b1ad01c86371765ef25b36572459e)([#8202](https://www.github.com/tauri-apps/tauri/pull/8202)) Fixes `android build --open` and `ios build --open` IDE failing to read CLI options.
- [`bfbbefdb`](https://www.github.com/tauri-apps/tauri/commit/bfbbefdb9e13ed1f42f6db7fa9ceaa84db1267e9)([#8161](https://www.github.com/tauri-apps/tauri/pull/8161)) Fix invalid plugin template.
- [`92b50a3a`](https://www.github.com/tauri-apps/tauri/commit/92b50a3a398c9d55b6992a8f5c2571e4d72bdaaf)([#8209](https://www.github.com/tauri-apps/tauri/pull/8209)) Added support to Xcode's archive. This requires regenerating the Xcode project.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.18`

## \[2.0.0-alpha.17]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.17`

### Breaking Changes

- [`198abe3c`](https://www.github.com/tauri-apps/tauri/commit/198abe3c2cae06dacab860b3a93f715dcf529a95)([#8076](https://www.github.com/tauri-apps/tauri/pull/8076)) Updated the mobile plugin templates following the tauri v2.0.0-alpha.17 changes.

## \[2.0.0-alpha.16]

### New Features

- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Add `--no-dev-server-wait` option to skip waiting for the dev server to start when using `tauri dev`.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.16`

### Breaking Changes

- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Changed a number of environment variables used by tauri CLI for consistency and clarity:

  - `TAURI_PRIVATE_KEY` -> `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_KEY_PASSWORD` -> `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - `TAURI_SKIP_DEVSERVER_CHECK` -> `TAURI_CLI_NO_DEV_SERVER_WAIT`
  - `TAURI_DEV_SERVER_PORT` -> `TAURI_CLI_PORT`
  - `TAURI_PATH_DEPTH` -> `TAURI_CLI_CONFIG_DEPTH`
  - `TAURI_FIPS_COMPLIANT` -> `TAURI_BUNDLER_WIX_FIPS_COMPLIANT`
  - `TAURI_DEV_WATCHER_IGNORE_FILE` -> `TAURI_CLI_WATCHER_IGNORE_FILENAME`
  - `TAURI_TRAY` -> `TAURI_LINUX_AYATANA_APPINDICATOR`
  - `TAURI_APPLE_DEVELOPMENT_TEAM` -> `APPLE_DEVELOPMENT_TEAM`
- [`4caa1cca`](https://www.github.com/tauri-apps/tauri/commit/4caa1cca990806f2c2ef32d5dabaf56e82f349e6)([#7990](https://www.github.com/tauri-apps/tauri/pull/7990)) The `tauri plugin` subcommand is receving a couple of consitency and quality of life improvements:

  - Renamed `tauri plugin android/ios add` command to `tauri plugin android/ios init` to match the `tauri plugin init` command.
  - Removed the `-n/--name` argument from the `tauri plugin init`, `tauri plugin android/ios init`, and is now parsed from the first positional argument.
  - Added `tauri plugin new` to create a plugin in a new directory.
  - Changed `tauri plugin init` to initalize a plugin in an existing directory (defaults to current directory) instead of creating a new one.
  - Changed `tauri plugin init` to NOT generate mobile projects by default, you can opt-in to generate them using `--android` and `--ios` flags or `--mobile` flag or initalize them later using `tauri plugin android/ios init`.
- [`8b166e9b`](https://www.github.com/tauri-apps/tauri/commit/8b166e9bf82e69ddb3200a3a825614980bd8d433)([#7949](https://www.github.com/tauri-apps/tauri/pull/7949)) Removed checking for a new version of the CLI.
- [`ebcc21e4`](https://www.github.com/tauri-apps/tauri/commit/ebcc21e4b95f4e8c27639fb1bca545b432f52d5e)([#8057](https://www.github.com/tauri-apps/tauri/pull/8057)) Renamed the beforeDevCommand, beforeBuildCommand and beforeBundleCommand hooks environment variables from `TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG` to `TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG` to differentiate the prefix with other CLI environment variables.

## \[2.0.0-alpha.15]

### New Features

- [`b2f17723`](https://www.github.com/tauri-apps/tauri/commit/b2f17723a415f04c2620132a6305eb138d7cb47f)([#7971](https://www.github.com/tauri-apps/tauri/pull/7971)) Use `devicectl` to connect to iOS 17+ devices on macOS 14+.

### Bug Fixes

- [`100d9ede`](https://www.github.com/tauri-apps/tauri/commit/100d9ede35995d9db21d2087dd5606adfafb89a5)([#7802](https://www.github.com/tauri-apps/tauri/pull/7802)) Properly read platform-specific configuration files for mobile targets.
- [`228e5a4c`](https://www.github.com/tauri-apps/tauri/commit/228e5a4c76ad5f97409c912d07699b49ba4bb162)([#7902](https://www.github.com/tauri-apps/tauri/pull/7902)) Fixes `icon` command not writing files to the correct Android project folders.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.15`

## \[2.0.0-alpha.14]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.14`

### Breaking Changes

- [`d5074af5`](https://www.github.com/tauri-apps/tauri/commit/d5074af562b2b5cb6c5711442097c4058af32db6)([#7801](https://www.github.com/tauri-apps/tauri/pull/7801)) The custom protocol on Android now uses the `http` scheme instead of `https`.

## \[2.0.0-alpha.13]

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.13`

### Breaking Changes

- [`4cb51a2d`](https://www.github.com/tauri-apps/tauri/commit/4cb51a2d56cfcae0749062c79ede5236bd8c02c2)([#7779](https://www.github.com/tauri-apps/tauri/pull/7779)) The custom protocol on Windows now uses the `http` scheme instead of `https`.
- [`974e38b4`](https://www.github.com/tauri-apps/tauri/commit/974e38b4ddc198530aa977ec77d513b76013b9f3)([#7744](https://www.github.com/tauri-apps/tauri/pull/7744)) Renamed the `plugin add` command to `add`.

## \[2.0.0-alpha.12]

### Bug Fixes

- [`b75a1210`](https://www.github.com/tauri-apps/tauri/commit/b75a1210bed589187678861d7314ae6279bf7c87)([#7762](https://www.github.com/tauri-apps/tauri/pull/7762)) Fixes a regression on alpha.11 where iOS logs aren't being displayed when using `ios dev` with a real device.
- [`8faa5a4a`](https://www.github.com/tauri-apps/tauri/commit/8faa5a4a1238a44ca7b54d2084aaed553ac2a1ba)([#7765](https://www.github.com/tauri-apps/tauri/pull/7765)) Ensure asset directory exists on the iOS project.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.12`

## \[2.0.0-alpha.11]

### New Features

- [`522de0e7`](https://www.github.com/tauri-apps/tauri/commit/522de0e78891d0bdf6387a5118985fc41a11baeb)([#7447](https://www.github.com/tauri-apps/tauri/pull/7447)) Expose an environment variable `TAURI_${PLUGIN_NAME}_PLUGIN_CONFIG` for each defined plugin configuration object.
- [`c7dacca4`](https://www.github.com/tauri-apps/tauri/commit/c7dacca4661c6ddf937c1a3dd3ace896d5baf40c)([#7446](https://www.github.com/tauri-apps/tauri/pull/7446)) Expose the `TAURI_IOS_PROJECT_PATH` and `TAURI_IOS_APP_NAME` environment variables when using `ios` commands.
- [`aa94f719`](https://www.github.com/tauri-apps/tauri/commit/aa94f7197e4345a7cab1617272b10895859674f9)([#7445](https://www.github.com/tauri-apps/tauri/pull/7445)) Generate empty entitlements file for the iOS project.
- [`d010bc07`](https://www.github.com/tauri-apps/tauri/commit/d010bc07b81116bef769b64cdc19b23dff762d48)([#7554](https://www.github.com/tauri-apps/tauri/pull/7554)) Set the iOS project PRODUCT_NAME value to the string under `tauri.conf.json > package > productName` if it is set.
- [`8af24974`](https://www.github.com/tauri-apps/tauri/commit/8af2497496f11ee481472dfdc3125757346c1a3e)([#7561](https://www.github.com/tauri-apps/tauri/pull/7561)) The `migrate` command now automatically reads all JavaScript files and updates `@tauri-apps/api` import paths and install the missing plugins.

### Enhancements

- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Update migrate command to update the configuration CSP to include `ipc:` on the `connect-src` directive, needed by the new IPC using custom protocols.

### Dependencies

- Upgraded to `tauri-cli@2.0.0-alpha.11`

## \[2.0.0-alpha.10]

### New Features

- [`7e5905ae`](https://www.github.com/tauri-apps/tauri/commit/7e5905ae1d56b920de0e821be28036cbbe302518)([#7023](https://www.github.com/tauri-apps/tauri/pull/7023)) Added `tauri plugin add` command to add a plugin to the Tauri project.
- [`b0f94775`](https://www.github.com/tauri-apps/tauri/commit/b0f947752a315b7b89c5979de50157f997f1dd6e)([#7008](https://www.github.com/tauri-apps/tauri/pull/7008)) Added `migrate` command.

### Enhancements

- [`aa6c9164`](https://www.github.com/tauri-apps/tauri/commit/aa6c9164e63b5316d690f25b1c118f1b12310570)([#7007](https://www.github.com/tauri-apps/tauri/pull/7007)) Don't build library files when building desktop targets.
- [`a28fdf7e`](https://www.github.com/tauri-apps/tauri/commit/a28fdf7ec71bf6db2498569004de83318b6d25ac)([#7044](https://www.github.com/tauri-apps/tauri/pull/7044)) Skip Rust target installation if they are already installed.
- [`735db1ce`](https://www.github.com/tauri-apps/tauri/commit/735db1ce839a16ba998c9e6786c441e3bf6c90b3)([#7044](https://www.github.com/tauri-apps/tauri/pull/7044)) Add `--skip-targets-install` flag for `tauri android init` and `tauri ios init` to skip installing needed rust targets vie rustup.

### Bug Fixes

- [`1ed2600d`](https://www.github.com/tauri-apps/tauri/commit/1ed2600da67715908af857255305eaeb293d8791)([#6771](https://www.github.com/tauri-apps/tauri/pull/6771)) Set current directory to tauri directory before reading config file.
- [`4847b87b`](https://www.github.com/tauri-apps/tauri/commit/4847b87b1067dd8c6e73986059f51e6eee1f1121)([#7209](https://www.github.com/tauri-apps/tauri/pull/7209)) Fix `tauri (android|ios) (dev|build)` failing when using `npx tauri`
- [`655c714e`](https://www.github.com/tauri-apps/tauri/commit/655c714e4100f69d4265c5ea3f08f5bc11709446)([#7240](https://www.github.com/tauri-apps/tauri/pull/7240)) Fixes panic when exiting the `ios dev` command with Ctrl + C.
- [`6252380f`](https://www.github.com/tauri-apps/tauri/commit/6252380f4447c66913b0f06611e5949005b1eec2)([#7241](https://www.github.com/tauri-apps/tauri/pull/7241)) Exit `beforeDevCommand` process if the android or iOS `dev` command fails.

## \[2.0.0-alpha.9]

- [`19cd0e49`](https://www.github.com/tauri-apps/tauri/commit/19cd0e49603ad3500cd2180bfa16e1649e3a771a)([#6811](https://www.github.com/tauri-apps/tauri/pull/6811)) Add `key.properties` file to android's `.gitignore`.
- [`124d5c5a`](https://www.github.com/tauri-apps/tauri/commit/124d5c5adf67f0b68d2e41c7ddb07d9cb63f1996)([#6788](https://www.github.com/tauri-apps/tauri/pull/6788)) On mobile, fix regression introduced in `tauri-cli` version `2.0.0-alpha.3` where library not found error was thrown.
- [`31444ac1`](https://www.github.com/tauri-apps/tauri/commit/31444ac196add770f2ad18012d7c18bce7538f22)([#6725](https://www.github.com/tauri-apps/tauri/pull/6725)) Update mobile template to `wry@0.28`
- [`41f49aea`](https://www.github.com/tauri-apps/tauri/commit/41f49aeae646f2cb70b42002bb1371c79e592243)([#6659](https://www.github.com/tauri-apps/tauri/pull/6659)) Update tauri-mobile to fix running ADB scripts.
- [`6d1fa49f`](https://www.github.com/tauri-apps/tauri/commit/6d1fa49fce3a03965ce7c656390e682ce5b776e3)([#6881](https://www.github.com/tauri-apps/tauri/pull/6881)) Clear Android plugin JSON file before building Rust library to ensure removed plugins are propagated to the Android project.
- [`5a9307d1`](https://www.github.com/tauri-apps/tauri/commit/5a9307d11c1643221bc2a280feb00024f8fa6030)([#6890](https://www.github.com/tauri-apps/tauri/pull/6890)) Update android template to gradle 8.0
- [`73c803a5`](https://www.github.com/tauri-apps/tauri/commit/73c803a561181137f20366f5d52511392a619f2b)([#6837](https://www.github.com/tauri-apps/tauri/pull/6837)) Inject Tauri configuration in the Android assets.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the updater configuration to the `BundleConfig`.
- [`d48aaa15`](https://www.github.com/tauri-apps/tauri/commit/d48aaa150a1ceeb65ec0ba18f1e3795f70c838e3)([#6894](https://www.github.com/tauri-apps/tauri/pull/6894)) Add Cargo manifest files for the plugin example templates.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed the allowlist configuration.

## \[2.0.0-alpha.8]

- Do not gitignore the Android project's `buildSrc` folder by default since we removed absolute paths from it.
  - [ee2d3b97](https://www.github.com/tauri-apps/tauri/commit/ee2d3b971df6d3630b8d935394fb4febcfa3a909) fix(cli): remove buildSrc from Android project gitignored paths ([#6702](https://www.github.com/tauri-apps/tauri/pull/6702)) on 2023-04-13
- Fixes iOS build script using the wrong path for the app library file.
  - [abc5f91f](https://www.github.com/tauri-apps/tauri/commit/abc5f91fa3569efc9dfdee46d1c501eda8755944) fix(cli): iOS Xcode script using incorrect library path ([#6699](https://www.github.com/tauri-apps/tauri/pull/6699)) on 2023-04-13

## \[2.0.0-alpha.7]

- Add `--release` flag for `tauri android dev` however you will need to sign your Android app, see https://next--tauri.netlify.app/next/guides/distribution/sign-android
  - [63f088e5](https://www.github.com/tauri-apps/tauri/commit/63f088e5fc9701fd7fb329dad7ffb27a2d8fd5aa) feat(cli): add `--release` for `android dev` ([#6638](https://www.github.com/tauri-apps/tauri/pull/6638)) on 2023-04-05
- Build only specified rust targets for `tauri android build` instead of all.
  - [d03e47d1](https://www.github.com/tauri-apps/tauri/commit/d03e47d141c3917520975be9081775dbc4e9d4fd) fix: only build specified rust targets for aab/apk build ([#6625](https://www.github.com/tauri-apps/tauri/pull/6625)) on 2023-04-05
- Use local ip address for built-in dev server on mobile.
  - [7fec0f08](https://www.github.com/tauri-apps/tauri/commit/7fec0f083c932dc63ccb8716080d97e2ab985b25) fix(cli): use local ip addr for built-in server on mobile, closes [#6454](https://www.github.com/tauri-apps/tauri/pull/6454) ([#6631](https://www.github.com/tauri-apps/tauri/pull/6631)) on 2023-04-04
- Readd the Cargo.toml file to the plugin template.
  - [5288a386](https://www.github.com/tauri-apps/tauri/commit/5288a386f1bf8ac11f991350463c3f5c20983f43) fix(cli): readd Cargo.toml to the plugin template ([#6637](https://www.github.com/tauri-apps/tauri/pull/6637)) on 2023-04-04

## \[2.0.0-alpha.6]

- Use Ubuntu 20.04 to compile the CLI, increasing the minimum libc version required.
- Automatically enable the `rustls-tls` tauri feature on mobile and `native-tls` on desktop if `rustls-tls` is not enabled.
  - [cfdee00f](https://www.github.com/tauri-apps/tauri/commit/cfdee00f2b1455a9719bc44823fdaeabbe4c1cb2) refactor(core): fix tls features, use rustls on mobile ([#6591](https://www.github.com/tauri-apps/tauri/pull/6591)) on 2023-03-30

## \[2.0.0-alpha.5]

- Fixes the iOS project script to build the Rust library.
  - [6e3e4c22](https://www.github.com/tauri-apps/tauri/commit/6e3e4c22be51500bec7856d90dcb2e40ef7fe1b4) fix(cli): use correct variable on script to build Rust iOS code ([#6581](https://www.github.com/tauri-apps/tauri/pull/6581)) on 2023-03-29
- Fix `tauri android build/dev` crashing when used with standalone `pnpm` executable on Windows.
  - [39df2c98](https://www.github.com/tauri-apps/tauri/commit/39df2c982e5e2ee8617b40f829a2f2e4abfce412) fix(cli/android): fallback to `${program}.cmd` ([#6576](https://www.github.com/tauri-apps/tauri/pull/6576)) on 2023-03-29
  - [6b469c40](https://www.github.com/tauri-apps/tauri/commit/6b469c40c6244ba773d7478adbb41db6fdc72822) chore(changes): adjust change file for Android script execution fix on 2023-03-29

## \[2.0.0-alpha.4]

- Fix android project build crashing when using `pnpm` caused by extra `--`.
  - [c787f749](https://www.github.com/tauri-apps/tauri/commit/c787f749de01b79d891615aad8c37b23037fff4c) fix(cli): only add `--` to generated android template for npm ([#6508](https://www.github.com/tauri-apps/tauri/pull/6508)) on 2023-03-21
- Fixes the Android build gradle plugin implementation on Windows.
  - [00241fa9](https://www.github.com/tauri-apps/tauri/commit/00241fa92d104870068a701519340633cc35b716) fix(cli): append .cmd on the gradle plugin binary on Windows, fix [#6502](https://www.github.com/tauri-apps/tauri/pull/6502) ([#6503](https://www.github.com/tauri-apps/tauri/pull/6503)) on 2023-03-21
- Update `napi-rs` dependencies to latest to fix CLI hanging up forever.
  - [d5ac76b5](https://www.github.com/tauri-apps/tauri/commit/d5ac76b53c7f35253db84ddfbf4ecf975ff6307d) chore(deps): update napi-rs, closes [#6502](https://www.github.com/tauri-apps/tauri/pull/6502) ([#6513](https://www.github.com/tauri-apps/tauri/pull/6513)) on 2023-03-21

## \[2.0.0-alpha.3]

- Added `plugin android add` and `plugin ios add` commands to add mobile plugin functionality to existing projects.
  - [14d03d42](https://www.github.com/tauri-apps/tauri/commit/14d03d426e86d966950a790926c04560c76203b3) refactor(cli): enhance plugin commands for mobile ([#6289](https://www.github.com/tauri-apps/tauri/pull/6289)) on 2023-02-16
- Add commands to add native Android and iOS functionality to plugins.
  - [05dad087](https://www.github.com/tauri-apps/tauri/commit/05dad0876842e2a7334431247d49365cee835d3e) feat: initial work for iOS plugins ([#6205](https://www.github.com/tauri-apps/tauri/pull/6205)) on 2023-02-11
- Use temp file instead of environment variable to pass CLI IPC websocket address to the IDE.
  - [894a8d06](https://www.github.com/tauri-apps/tauri/commit/894a8d060c12a482a0fc5b3714f3848189b809de) refactor(cli): use temp file to communicate IPC websocket address ([#6219](https://www.github.com/tauri-apps/tauri/pull/6219)) on 2023-02-08
- Change the Android template to enable minification on release and pull ProGuard rules from proguard-tauri.pro.
  - [bef4ef51](https://www.github.com/tauri-apps/tauri/commit/bef4ef51bc2c633b88db121c2087a38dddb7d6bf) feat(android): enable minify on release, add proguard rules ([#6257](https://www.github.com/tauri-apps/tauri/pull/6257)) on 2023-02-13
- Print an error if the Android project was generated with an older bundle identifier or package name.
  - [79eb0542](https://www.github.com/tauri-apps/tauri/commit/79eb054292b04bb089a0e4df401a5986b33b691e) feat(cli): handle Android package identifier change ([#6314](https://www.github.com/tauri-apps/tauri/pull/6314)) on 2023-02-19
- Fixes the generated mobile build script when using an NPM runner.
  - [62f15265](https://www.github.com/tauri-apps/tauri/commit/62f152659204ce1218178596f463f0bcfbd4e6dc) fix(cli): generate build script using NPM runner if it was used ([#6233](https://www.github.com/tauri-apps/tauri/pull/6233)) on 2023-02-10
- Resolve Android package name from single word bundle identifiers.
  - [60a8b07d](https://www.github.com/tauri-apps/tauri/commit/60a8b07dc7c56c9c45331cb57d9afb410e7eadf3) fix: handle single word bundle identifier when resolving Android domain ([#6313](https://www.github.com/tauri-apps/tauri/pull/6313)) on 2023-02-19
- Update Android project template with fix to crash on orientation change.
  - [947eb391](https://www.github.com/tauri-apps/tauri/commit/947eb391ca41cebdb11abd9ffaec642baffbf44a) fix(android): crash on orientation change due to activity recreation ([#6261](https://www.github.com/tauri-apps/tauri/pull/6261)) on 2023-02-13
- Added `--ios-color` option to the `tauri icon` command.
  - [67755425](https://www.github.com/tauri-apps/tauri/commit/677554257e40e05b1af0dd61c982d6be8a8a033c) feat(cli): add `--ios-color` option to set iOS icon background color ([#6247](https://www.github.com/tauri-apps/tauri/pull/6247)) on 2023-02-12
- Fixes HMR on mobile when devPath is configured to load a filesystem path.
  - [4a82da29](https://www.github.com/tauri-apps/tauri/commit/4a82da2919e0564ec993b2005dc65b5b49407b36) fix(cli): use local ip address for reload ([#6285](https://www.github.com/tauri-apps/tauri/pull/6285)) on 2023-02-16
- Ignore the `gen` folder on the dev watcher.
  - [cab4ff95](https://www.github.com/tauri-apps/tauri/commit/cab4ff95b98aeac88401c1fed2d8b8940e4180cb) fix(cli): ignore the `gen` folder on the dev watcher ([#6232](https://www.github.com/tauri-apps/tauri/pull/6232)) on 2023-02-09
- Correctly pass arguments from `npm run` to `tauri`.
  - [1b343bd1](https://www.github.com/tauri-apps/tauri/commit/1b343bd11686f47f24a87298d8192097c66250f6) fix(cli): use `npm run tauri -- foo` for correctly passing args to tauri ([#6448](https://www.github.com/tauri-apps/tauri/pull/6448)) on 2023-03-16
- Changed the `--api` flag on `plugin init` to `--no-api`.
  - [14d03d42](https://www.github.com/tauri-apps/tauri/commit/14d03d426e86d966950a790926c04560c76203b3) refactor(cli): enhance plugin commands for mobile ([#6289](https://www.github.com/tauri-apps/tauri/pull/6289)) on 2023-02-16

## \[2.0.0-alpha.2]

- Fixes `TAURI_*` environment variables for hook scripts on mobile commands.
  - [1af9be90](https://www.github.com/tauri-apps/tauri/commit/1af9be904a309138b9f79dc741391000b1652c75) feat(cli): properly fill target for TAURI\_ env vars on mobile ([#6116](https://www.github.com/tauri-apps/tauri/pull/6116)) on 2023-01-23
- Force colored logs on mobile commands.
  - [2c4a0bbd](https://www.github.com/tauri-apps/tauri/commit/2c4a0bbd1fbe15d7500264e6490772397e1917ed) feat(cli): force colored logs on mobile commands ([#5934](https://www.github.com/tauri-apps/tauri/pull/5934)) on 2022-12-28
- Keep the process alive even when the iOS application is closed.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Show all application logs on iOS.
  - [dee9460f](https://www.github.com/tauri-apps/tauri/commit/dee9460f9c9bc92e9c638e7691e616849ac2085b) feat: keep CLI alive when iOS app exits, show logs, closes [#5855](https://www.github.com/tauri-apps/tauri/pull/5855) ([#5902](https://www.github.com/tauri-apps/tauri/pull/5902)) on 2022-12-27
- Print log output for all tags on Android development.
  - [8cc11149](https://www.github.com/tauri-apps/tauri/commit/8cc111494d74161e489152e52191e1442dd99759) fix(cli): print Android logs for all tags on 2023-01-17
- Add support to custom and kebab case library names for mobile apps.
  - [50f6dd87](https://www.github.com/tauri-apps/tauri/commit/50f6dd87b1ac2c99f8794b055f1acba4ef7d34d3) feat: improvements to support hyphens in crate name ([#5989](https://www.github.com/tauri-apps/tauri/pull/5989)) on 2023-01-06
- Fix target directory detection when compiling for Android.
  - [e873bae0](https://www.github.com/tauri-apps/tauri/commit/e873bae09f0f27517f720a753f51c1dcb903f883) fix(cli): Cargo target dir detection on Android, closes [#5865](https://www.github.com/tauri-apps/tauri/pull/5865) ([#5932](https://www.github.com/tauri-apps/tauri/pull/5932)) on 2022-12-28

## \[2.0.0-alpha.1]

- Fixes running on device using Xcode 14.
  - [1e4a6758](https://www.github.com/tauri-apps/tauri/commit/1e4a675843c486bddc11292d09fb766e98758514) fix(cli): run on iOS device on Xcode 14 ([#5807](https://www.github.com/tauri-apps/tauri/pull/5807)) on 2022-12-12
- Improve local IP address detection with user selection.
  - [76204b89](https://www.github.com/tauri-apps/tauri/commit/76204b893846a04552f8f8b87ad2c9b55e1b417f) feat(cli): improve local IP detection ([#5817](https://www.github.com/tauri-apps/tauri/pull/5817)) on 2022-12-12

## \[2.0.0-alpha.0]

- Added `android build` command.
  - [4c9ea450](https://www.github.com/tauri-apps/tauri/commit/4c9ea450c3b47c6b8c825ba32e9837909945ccd7) feat(cli): add `android build` command ([#4999](https://www.github.com/tauri-apps/tauri/pull/4999)) on 2022-08-22
- Added `ios build` command.
  - [403859d4](https://www.github.com/tauri-apps/tauri/commit/403859d47e1a9bf978b353fa58e4b971e66337a3) feat(cli): add `ios build` command ([#5002](https://www.github.com/tauri-apps/tauri/pull/5002)) on 2022-08-22
- Added `android dev` and `ios dev` commands.
  - [6f061504](https://www.github.com/tauri-apps/tauri/commit/6f0615044d09ec58393a7ebca5e45bb175e20db3) feat(cli): add `android dev` and `ios dev` commands ([#4982](https://www.github.com/tauri-apps/tauri/pull/4982)) on 2022-08-20
- Added `android init` and `ios init` commands.
  - [d44f67f7](https://www.github.com/tauri-apps/tauri/commit/d44f67f7afd30a81d53a973ec603b2a253150bde) feat: add `android init` and `ios init` commands ([#4942](https://www.github.com/tauri-apps/tauri/pull/4942)) on 2022-08-15
- Added `android open` and `ios open` commands.
  - [a9c8e565](https://www.github.com/tauri-apps/tauri/commit/a9c8e565c6495961940877df7090f307be16b554) feat: add `android open` and `ios open` commands ([#4946](https://www.github.com/tauri-apps/tauri/pull/4946)) on 2022-08-15
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.5.11]

### Bug Fixes

- [`b15948b11`](https://www.github.com/tauri-apps/tauri/commit/b15948b11c0e362eea7ef57a4606f15f7dbd886b)([#8903](https://www.github.com/tauri-apps/tauri/pull/8903)) Fix `.taurignore` failing to ignore in some cases.

### Dependencies

- Upgraded to `tauri-cli@1.5.11`

## \[1.5.10]

### Bug Fixes

- [`b0f27814`](https://www.github.com/tauri-apps/tauri/commit/b0f27814b90ded2f1ed44b7852080eedbff0d9e4)([#8776](https://www.github.com/tauri-apps/tauri/pull/8776)) Fix `fail to rename app` when using `--profile dev`.
- [`0bff8c32`](https://www.github.com/tauri-apps/tauri/commit/0bff8c325d004fdead2023f58e0f5fd73a9c22ba)([#8697](https://www.github.com/tauri-apps/tauri/pull/8697)) Fix the built-in dev server failing to serve files when URL had queries `?` and other url components.
- [`67d7877f`](https://www.github.com/tauri-apps/tauri/commit/67d7877f27f265c133a70d48a46c83ffff31d571)([#8520](https://www.github.com/tauri-apps/tauri/pull/8520)) The cli now also watches cargo workspace members if the tauri folder is the workspace root.

### Dependencies

- Upgraded to `tauri-cli@1.5.10`

## \[1.5.9]

### Bug Fixes

- [`0a2175ea`](https://www.github.com/tauri-apps/tauri/commit/0a2175eabb736b2a4cd01ab682e08be0b5ebb2b9)([#8439](https://www.github.com/tauri-apps/tauri/pull/8439)) Expand glob patterns in workspace member paths so the CLI would watch all matching pathhs.

### Dependencies

- Upgraded to `tauri-cli@1.5.9`

## \[1.5.8]

### Dependencies

- Upgraded to `tauri-cli@1.5.8`

## \[1.5.7]

### Bug Fixes

- [`1d5aa38a`](https://www.github.com/tauri-apps/tauri/commit/1d5aa38ae418ea31f593590b6d32cf04d3bfd8c1)([#8162](https://www.github.com/tauri-apps/tauri/pull/8162)) Fixes errors on command output, occuring when the output stream contains an invalid UTF-8 character, or ends with a multi-bytes UTF-8 character.
- [`f26d9f08`](https://www.github.com/tauri-apps/tauri/commit/f26d9f0884f63f61b9f4d4fac15e6b251163793e)([#8263](https://www.github.com/tauri-apps/tauri/pull/8263)) Fixes an issue in the NSIS installer which caused the uninstallation to leave empty folders on the system if the `resources` feature was used.
- [`92bc7d0e`](https://www.github.com/tauri-apps/tauri/commit/92bc7d0e16157434330a1bcf1eefda6f0f1e5f85)([#8233](https://www.github.com/tauri-apps/tauri/pull/8233)) Fixes an issue in the NSIS installer which caused the installation to take much longer than expected when many `resources` were added to the bundle.

### Dependencies

- Upgraded to `tauri-cli@1.5.7`

## \[1.5.6]

### Bug Fixes

- [`5264e41d`](https://www.github.com/tauri-apps/tauri/commit/5264e41db3763e4c2eb0c3c21bd423fb7bece3e2)([#8082](https://www.github.com/tauri-apps/tauri/pull/8082)) Downgraded `rust-minisign` to `0.7.3` to fix signing updater bundles with empty passwords.

### Dependencies

- Upgraded to `tauri-cli@1.5.6`

## \[1.5.5]

### Enhancements

- [`9bead42d`](https://www.github.com/tauri-apps/tauri/commit/9bead42dbca0fb6dd7ea0b6bfb2f2308a5c5f992)([#8059](https://www.github.com/tauri-apps/tauri/pull/8059)) Allow rotating the updater private key.

### Bug Fixes

- [`be8e5aa3`](https://www.github.com/tauri-apps/tauri/commit/be8e5aa3071d9bc5d0bd24647e8168f312d11c8d)([#8042](https://www.github.com/tauri-apps/tauri/pull/8042)) Fixes duplicated newlines on command outputs.

### Dependencies

- Upgraded to `tauri-cli@1.5.5`

## \[1.5.4]

### Dependencies

- Upgraded to `tauri-cli@1.5.4`

## \[1.5.3]

### Dependencies

- Upgraded to `tauri-cli@1.5.3`

## \[1.5.2]

### Dependencies

- Upgraded to `tauri-cli@1.5.2`

## \[1.5.1]

### Bug Fixes

- [`d6eb46cf`](https://www.github.com/tauri-apps/tauri/commit/d6eb46cf1116d147121f6b6db9d390b5e2fb238d)([#7934](https://www.github.com/tauri-apps/tauri/pull/7934)) On macOS, fix the `apple-id` option name when using `notarytools submit`.

### Dependencies

- Upgraded to `tauri-cli@1.5.1`

## \[1.5.0]

### New Features

- [`e1526626`](https://www.github.com/tauri-apps/tauri/commit/e152662687ece7a62d383923a50751cc0dd34331)([#7723](https://www.github.com/tauri-apps/tauri/pull/7723)) Support Bun package manager in CLI

### Enhancements

- [`13279917`](https://www.github.com/tauri-apps/tauri/commit/13279917d4cae071d0ce3a686184d48af079f58a)([#7713](https://www.github.com/tauri-apps/tauri/pull/7713)) Add version of Rust Tauri CLI installed with Cargo to `tauri info` command.

### Bug Fixes

- [`dad4f54e`](https://www.github.com/tauri-apps/tauri/commit/dad4f54eec9773d2ea6233a7d9fd218741173823)([#7277](https://www.github.com/tauri-apps/tauri/pull/7277)) Removed the automatic version check of the CLI that ran after `tauri` commands which caused various issues.

### Dependencies

- Upgraded to `tauri-cli@1.5.0`

## \[1.4.0]

### New Features

- [`0ddbb3a1`](https://www.github.com/tauri-apps/tauri/commit/0ddbb3a1dc1961ba5c6c1a60081513c1380c8af1)([#7015](https://www.github.com/tauri-apps/tauri/pull/7015)) Provide prebuilt CLIs for Windows ARM64 targets.
- [`35cd751a`](https://www.github.com/tauri-apps/tauri/commit/35cd751adc6fef1f792696fa0cfb471b0bf99374)([#5176](https://www.github.com/tauri-apps/tauri/pull/5176)) Added the `desktop_template` option on `tauri.conf.json > tauri > bundle > deb`.
- [`6c5ade08`](https://www.github.com/tauri-apps/tauri/commit/6c5ade08d97844bb685789d30e589400bbe3e04c)([#4537](https://www.github.com/tauri-apps/tauri/pull/4537)) Added `tauri completions` to generate shell completions scripts.
- [`e092f799`](https://www.github.com/tauri-apps/tauri/commit/e092f799469ff32c7d1595d0f07d06fd2dab5c29)([#6887](https://www.github.com/tauri-apps/tauri/pull/6887)) Add `nsis > template` option to specify custom NSIS installer template.

### Enhancements

- [`d75c1b82`](https://www.github.com/tauri-apps/tauri/commit/d75c1b829bd96d9e3a672bcc79120597d5ada4a0)([#7181](https://www.github.com/tauri-apps/tauri/pull/7181)) Print a useful error when `updater` bundle target is specified without an updater-enabled target.
- [`52474e47`](https://www.github.com/tauri-apps/tauri/commit/52474e479d695865299d8c8d868fb98b99731020)([#7141](https://www.github.com/tauri-apps/tauri/pull/7141)) Enhance injection of Cargo features.
- [`2659ca1a`](https://www.github.com/tauri-apps/tauri/commit/2659ca1ab4799a5bda65c229c149e98bd01eb1ee)([#6900](https://www.github.com/tauri-apps/tauri/pull/6900)) Add `rustls` as default Cargo feature.

### Bug Fixes

- [`3cb7a3e6`](https://www.github.com/tauri-apps/tauri/commit/3cb7a3e642bb10ee90dc1d24daa48b8c8c15c9ce)([#6997](https://www.github.com/tauri-apps/tauri/pull/6997)) Fix built-in devserver adding hot-reload code to non-html files.
- [`fb7ef8da`](https://www.github.com/tauri-apps/tauri/commit/fb7ef8dacd9ade96976c84d22507782cdaf38acf)([#6667](https://www.github.com/tauri-apps/tauri/pull/6667)) Fix nodejs binary regex when `0` is in the version name, for example `node-20`
- [`1253bbf7`](https://www.github.com/tauri-apps/tauri/commit/1253bbf7ae11a87887e0b3bd98cc26dbb98c8130)([#7013](https://www.github.com/tauri-apps/tauri/pull/7013)) Fixes Cargo.toml feature rewriting.

## \[1.3.1]

- Correctly escape XML for resource files in WiX bundler.
  - Bumped due to a bump in tauri-bundler.
    - Bumped due to a bump in cli.rs.
  - [6a6b1388](https://www.github.com/tauri-apps/tauri/commit/6a6b1388ea5787aea4c0e0b0701a4772087bbc0f) fix(bundler): correctly escape resource xml, fixes [#6853](https://www.github.com/tauri-apps/tauri/pull/6853) ([#6855](https://www.github.com/tauri-apps/tauri/pull/6855)) on 2023-05-04

- Added the following languages to the NSIS bundler:

- `Spanish`

- `SpanishInternational`

- Bumped due to a bump in tauri-bundler.
  - Bumped due to a bump in cli.rs.

- [422b4817](https://www.github.com/tauri-apps/tauri/commit/422b48179856504e980a156500afa8e22c44d357) Add Spanish and SpanishInternational languages ([#6871](https://www.github.com/tauri-apps/tauri/pull/6871)) on 2023-05-06

- Correctly escape arguments in NSIS script to fix bundling apps that use non-default WebView2 install modes.
  - Bumped due to a bump in tauri-bundler.
    - Bumped due to a bump in cli.rs.
  - [2915bd06](https://www.github.com/tauri-apps/tauri/commit/2915bd068ed40dc01a363b69212c6b6f2d3ec01e) fix(bundler): Fix webview install modes in NSIS bundler ([#6854](https://www.github.com/tauri-apps/tauri/pull/6854)) on 2023-05-04

## \[1.3.0]

- Add `--ci` flag and respect the `CI` environment variable on the `signer generate` command. In this case the default password will be an empty string and the CLI will not prompt for a value.
  - [8fb1df8a](https://www.github.com/tauri-apps/tauri/commit/8fb1df8aa65a52cdb4a7e1bb9dda9b912a7a2895) feat(cli): add `--ci` flag to `signer generate`, closes [#6089](https://www.github.com/tauri-apps/tauri/pull/6089) ([#6097](https://www.github.com/tauri-apps/tauri/pull/6097)) on 2023-01-19
- Fix Outdated Github Actions in the Plugin Templates `with-api` and `backend`
  - [a926b49a](https://www.github.com/tauri-apps/tauri/commit/a926b49a01925ca757d391994bfac3beea29599b) Fix Github Actions of Tauri Plugin with-api template ([#6603](https://www.github.com/tauri-apps/tauri/pull/6603)) on 2023-04-03
- Do not crash on Cargo.toml watcher.
  - [e8014a7f](https://www.github.com/tauri-apps/tauri/commit/e8014a7f612a1094461ddad63aacc498a2682ff5) fix(cli): do not crash on watcher ([#6303](https://www.github.com/tauri-apps/tauri/pull/6303)) on 2023-02-17
- Fix crash when nodejs binary has the version in its name, for example `node-18`
  - [1c8229fb](https://www.github.com/tauri-apps/tauri/commit/1c8229fbe273554c0c97cccee45d5967f5df1b9f) fix(cli.js): detect `node-<version>` binary, closes [#6427](https://www.github.com/tauri-apps/tauri/pull/6427) ([#6432](https://www.github.com/tauri-apps/tauri/pull/6432)) on 2023-03-16
- Add `--png` option for the `icon` command to generate custom icon sizes.
  - [9d214412](https://www.github.com/tauri-apps/tauri/commit/9d2144128fc5fad67d8404bce95f82297ebb0e4a) feat(cli): add option to make custom icon sizes, closes [#5121](https://www.github.com/tauri-apps/tauri/pull/5121) ([#5246](https://www.github.com/tauri-apps/tauri/pull/5246)) on 2022-12-27
- Skip the password prompt on the build command when `TAURI_KEY_PASSWORD` environment variable is empty and the `--ci` argument is provided or the `CI` environment variable is set.
  - [d4f89af1](https://www.github.com/tauri-apps/tauri/commit/d4f89af18d69fd95a4d8a1ede8442547c6a6d0ee) feat: skip password prompt on the build command if CI is set fixes [#6089](https://www.github.com/tauri-apps/tauri/pull/6089) on 2023-01-18
- Fix `default-run` not deserialized.
  - [57c6bf07](https://www.github.com/tauri-apps/tauri/commit/57c6bf07bb380847abdf27c3fff9891d99c1c98c) fix(cli): fix default-run not deserialized ([#6584](https://www.github.com/tauri-apps/tauri/pull/6584)) on 2023-03-30
- Fixes HTML serialization removing template tags on the dev server.
  - [314f0e21](https://www.github.com/tauri-apps/tauri/commit/314f0e212fd2b9e452bfe3424cdce2b0bf37b5d7) fix(cli): web_dev_server html template serialization (fix [#6165](https://www.github.com/tauri-apps/tauri/pull/6165)) ([#6166](https://www.github.com/tauri-apps/tauri/pull/6166)) on 2023-01-29
- Use escaping on Handlebars templates.
  - [6d6b6e65](https://www.github.com/tauri-apps/tauri/commit/6d6b6e653ea70fc02794f723092cdc860995c259) feat: configure escaping on handlebars templates ([#6678](https://www.github.com/tauri-apps/tauri/pull/6678)) on 2023-05-02
- Add initial support for building `nsis` bundles on non-Windows platforms.
  - [60e6f6c3](https://www.github.com/tauri-apps/tauri/commit/60e6f6c3f1605f3064b5bb177992530ff788ccf0) feat(bundler): Add support for creating NSIS bundles on unix hosts ([#5788](https://www.github.com/tauri-apps/tauri/pull/5788)) on 2023-01-19
- Add `nsis` bundle target
  - [c94e1326](https://www.github.com/tauri-apps/tauri/commit/c94e1326a7c0767a13128a8b1d327a00156ece12) feat(bundler): add `nsis`, closes [#4450](https://www.github.com/tauri-apps/tauri/pull/4450), closes [#2319](https://www.github.com/tauri-apps/tauri/pull/2319) ([#4674](https://www.github.com/tauri-apps/tauri/pull/4674)) on 2023-01-03
- Remove default features from Cargo.toml template.
  - [b08ae637](https://www.github.com/tauri-apps/tauri/commit/b08ae637a0f58b38cbce9b8a1fa0b6c5dc0cfd05) fix(cli): remove default features from template ([#6074](https://www.github.com/tauri-apps/tauri/pull/6074)) on 2023-01-17
- Use Ubuntu 20.04 to compile the CLI, increasing the minimum libc version required.

## \[1.2.3]

- Pin `ignore` to `=0.4.18`.
  - [adcb082b](https://www.github.com/tauri-apps/tauri/commit/adcb082b1651ecb2a6208b093e12f4185aa3fc98) chore(deps): pin `ignore` to =0.4.18 on 2023-01-17

## \[1.2.2]

- Detect SvelteKit and Vite for the init and info commands.
  - [9d872ab8](https://www.github.com/tauri-apps/tauri/commit/9d872ab8728b1b121909af434adcd5936e5afb7d) feat(cli): detect SvelteKit and Vite ([#5742](https://www.github.com/tauri-apps/tauri/pull/5742)) on 2022-12-02
- Detect SolidJS and SolidStart for the init and info commands.
  - [9e7ce0a8](https://www.github.com/tauri-apps/tauri/commit/9e7ce0a8eef4bf3536645976e3e09162fbf772ab) feat(cli): detect SolidJS and SolidStart ([#5758](https://www.github.com/tauri-apps/tauri/pull/5758)) on 2022-12-08
- Use older icon types to work around a macOS bug resulting in corrupted 16x16px and 32x32px icons in bundled apps.
  - [2d545eff](https://www.github.com/tauri-apps/tauri/commit/2d545eff58734ec70f23f11a429d35435cdf090e) fix(cli): corrupted icons in bundled macOS icons ([#5698](https://www.github.com/tauri-apps/tauri/pull/5698)) on 2022-11-28

## \[1.2.1]

- Fixes injection of Cargo features defined in the configuration file.
  - [1ecaeb29](https://www.github.com/tauri-apps/tauri/commit/1ecaeb29aa798f591f6488dc6c3a7a8d22f6073e) fix(cli): inject config feature flags when features arg is not provided on 2022-11-18

## \[1.2.0]

- Detect JSON5 and TOML configuration files in the dev watcher.
  - [e7ccbd85](https://www.github.com/tauri-apps/tauri/commit/e7ccbd8573f6b9124e80c0b67fa2365729c3c196) feat(cli): detect JSON5 and TOML configuration files in the dev watcher ([#5439](https://www.github.com/tauri-apps/tauri/pull/5439)) on 2022-10-19
- Log dev watcher file change detection.
  - [9076d5d2](https://www.github.com/tauri-apps/tauri/commit/9076d5d2e76d432aef475ba403e9ab5bd3b9d2b0) feat(cli): add prompt information when file changing detected, closes [#5417](https://www.github.com/tauri-apps/tauri/pull/5417) ([#5428](https://www.github.com/tauri-apps/tauri/pull/5428)) on 2022-10-19
- Fix crash when nodejs binary has the version in its name, for example `node18` or when running through deno.
  - [7a231cd1](https://www.github.com/tauri-apps/tauri/commit/7a231cd1c99101f63354b13bb36223568d2f0a11) fix(cli): detect deno ([#5475](https://www.github.com/tauri-apps/tauri/pull/5475)) on 2022-10-28
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
- Use `cargo metadata` to detect the workspace root and target directory.
  - [fea70eff](https://www.github.com/tauri-apps/tauri/commit/fea70effad219c0794d919f8834fa1a1ffd204c7) refactor(cli): Use `cargo metadata` to detect the workspace root and target directory, closes [#4632](https://www.github.com/tauri-apps/tauri/pull/4632), [#4928](https://www.github.com/tauri-apps/tauri/pull/4928). ([#4932](https://www.github.com/tauri-apps/tauri/pull/4932)) on 2022-08-21
- Prompt for `beforeDevCommand` and `beforeBuildCommand` in `tauri init`.
  - [6d4945c9](https://www.github.com/tauri-apps/tauri/commit/6d4945c9f06cd1f7018e1c48686ba682aae817df) feat(cli): prompt for before\*Command, closes [#4691](https://www.github.com/tauri-apps/tauri/pull/4691) ([#4721](https://www.github.com/tauri-apps/tauri/pull/4721)) on 2022-07-25
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Automatically use any `.taurignore` file as ignore rules for dev watcher and app path finder.
  - [596fa08d](https://www.github.com/tauri-apps/tauri/commit/596fa08d48e371c7bd29e1ef799119ac8fca0d0b) feat(cli): automatically use `.taurignore`, ref [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4623](https://www.github.com/tauri-apps/tauri/pull/4623)) on 2022-07-28
- Enable WiX FIPS compliance when the `TAURI_FIPS_COMPLIANT` environment variable is set to `true`.
  - [d88b9de7](https://www.github.com/tauri-apps/tauri/commit/d88b9de7aaeaaa2e42e4795dbc2b8642b5ae7a50) feat(core): add `fips_compliant` wix config option, closes [#4541](https://www.github.com/tauri-apps/tauri/pull/4541) ([#4843](https://www.github.com/tauri-apps/tauri/pull/4843)) on 2022-08-04
- Fixes dev watcher incorrectly exiting the CLI when sequential file updates are detected.
  - [47fab680](https://www.github.com/tauri-apps/tauri/commit/47fab6809a1e23b3b9a84695e2d91ff0826ba79a) fix(cli): dev watcher incorrectly killing process on multiple file write ([#4684](https://www.github.com/tauri-apps/tauri/pull/4684)) on 2022-07-25
- Add `libc` field to Node packages.
  - [f7d2dfc7](https://www.github.com/tauri-apps/tauri/commit/f7d2dfc7a6d086dd1a218d6c1492b3fef8a64f03) chore: add libc field to node packages ([#4856](https://www.github.com/tauri-apps/tauri/pull/4856)) on 2022-08-04
- Set the `MACOSX_DEPLOYMENT_TARGET` environment variable with the configuration `minimum_system_version` value.
  - [fa23310f](https://www.github.com/tauri-apps/tauri/commit/fa23310f23cb9e6a02ec2524f1ef394a5b42990e) fix(cli): set MACOSX_DEPLOYMENT_TARGET env var, closes [#4704](https://www.github.com/tauri-apps/tauri/pull/4704) ([#4842](https://www.github.com/tauri-apps/tauri/pull/4842)) on 2022-08-02
- Added `--no-watch` argument to the `dev` command to disable the file watcher.
  - [0983d7ce](https://www.github.com/tauri-apps/tauri/commit/0983d7ce7f24ab43f9ae7b5e1177ff244d8885a8) feat(cli): add `--no-watch` argument to the dev command, closes [#4617](https://www.github.com/tauri-apps/tauri/pull/4617) ([#4793](https://www.github.com/tauri-apps/tauri/pull/4793)) on 2022-07-29
- Validate updater signature matches configured public key.
  - [b2a8930b](https://www.github.com/tauri-apps/tauri/commit/b2a8930b3c4b72c50ce72e73575f42c9cbe91bad) feat(cli): validate updater private key when signing ([#4754](https://www.github.com/tauri-apps/tauri/pull/4754)) on 2022-07-25

## \[1.0.5]

- Correctly fill the architecture when building Debian packages targeting ARM64 (aarch64).
  - Bumped due to a bump in cli.rs.
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
- Revert the `run` command to run in a separate thread.
  - [f65eb4f8](https://www.github.com/tauri-apps/tauri/commit/f65eb4f84d8e511cd30d01d20a8223a297f7e584) fix(cli.js): revert `run` command to be nonblocking on 2022-07-04
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
