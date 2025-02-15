# Changelog

## \[2.1.1]

### Bug Fixes

- [`46935212b`](https://www.github.com/tauri-apps/tauri/commit/46935212b61da44dc82dfeb803fceebf5659f7b7) ([#11658](https://www.github.com/tauri-apps/tauri/pull/11658) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Fix `.json5` capability files not recognized even with `config-json5` feature enabled

## \[2.1.0]

### New Features

- [`fabc2f283`](https://www.github.com/tauri-apps/tauri/commit/fabc2f283e38b62c721326e44645d47138418cbc) ([#11485](https://www.github.com/tauri-apps/tauri/pull/11485) by [@39zde](https://www.github.com/tauri-apps/tauri/../../39zde)) Adds a new configuration option `app > security > headers` to define headers that will be added to every http response from tauri to the web view. This doesn't include IPC messages and error responses.
- [`4d545ab3c`](https://www.github.com/tauri-apps/tauri/commit/4d545ab3ca228c8a21b966b709f84a0da2864479) ([#11486](https://www.github.com/tauri-apps/tauri/pull/11486) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Added `Window::set_background_color` and `WindowBuilder::background_color`.
- [`cbc095ec5`](https://www.github.com/tauri-apps/tauri/commit/cbc095ec5fe7de29b5c9265576d4e071ec159c1c) ([#11451](https://www.github.com/tauri-apps/tauri/pull/11451) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `app > windows > devtools` config option and when creating the webview from JS, to enable or disable devtools for a specific webview.
- [`058c0db72`](https://www.github.com/tauri-apps/tauri/commit/058c0db72f43fbe1574d0db654560e693755cd7e) ([#11584](https://www.github.com/tauri-apps/tauri/pull/11584) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `bundle > linux > rpm > compression` config option to control RPM bundle compression type and level.
- [`f37e97d41`](https://www.github.com/tauri-apps/tauri/commit/f37e97d410c4a219e99f97692da05ca9d8e0ba3a) ([#11477](https://www.github.com/tauri-apps/tauri/pull/11477) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `app > windows > useHttpsScheme` config option to choose whether the custom protocols should use `https://<scheme>.localhost` instead of the default `http://<scheme>.localhost` on Windows and Android
- [`2a75c64b5`](https://www.github.com/tauri-apps/tauri/commit/2a75c64b5431284e7340e8743d4ea56a62c75466) ([#11469](https://www.github.com/tauri-apps/tauri/pull/11469) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Added `app > windows > windowClassname` config option to specify the name of the window class on Windows.

### Enhancements

- [`c33bbf457`](https://www.github.com/tauri-apps/tauri/commit/c33bbf45740274b6918ea6c647f366fb6008e459) ([#11575](https://www.github.com/tauri-apps/tauri/pull/11575) by [@kornelski](https://www.github.com/tauri-apps/tauri/../../kornelski)) Include the path in ACL I/O errors.

### Bug Fixes

- [`378142914`](https://www.github.com/tauri-apps/tauri/commit/37814291475814b4a24cc77b6fa457ec9ba7a779) ([#11429](https://www.github.com/tauri-apps/tauri/pull/11429) by [@griffi-gh](https://www.github.com/tauri-apps/tauri/../../griffi-gh)) Enhance resource directory resolution to support running on distros like NixOS.

## \[2.0.2]

### New Features

- Add `bundler > windows > wix > version` to manually specify a wix-compatible version.

## \[2.0.1]

### What's Changed

- [`0ab2b3306`](https://www.github.com/tauri-apps/tauri/commit/0ab2b330644b6419f6cee1d5377bfb5cdda2ccf9) ([#11205](https://www.github.com/tauri-apps/tauri/pull/11205) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

### What's Changed

- [`382ed482b`](https://www.github.com/tauri-apps/tauri/commit/382ed482bd08157c39e62f9a0aaad8802f1092cb) Bump MSRV to 1.78.
- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

## \[2.0.0-rc.13]

### New Features

- [`a247170e1`](https://www.github.com/tauri-apps/tauri/commit/a247170e1f620a9b012274b11cfe51e90327d6e9) ([#11056](https://www.github.com/tauri-apps/tauri/pull/11056) by [@SpikeHD](https://www.github.com/tauri-apps/tauri/../../SpikeHD)) Expose the ability to enabled browser extensions in WebView2 on Windows.
- [`f57a729cd`](https://www.github.com/tauri-apps/tauri/commit/f57a729cd8f7e10d8daf0b9d5b85f9c7ad530496) ([#11039](https://www.github.com/tauri-apps/tauri/pull/11039) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `upgradeCode` in `wix` configuration to set an upgrade code for your MSI installer. This is recommended to be set if you plan to change your `productName`.

### Bug Fixes

- [`1efa5e718`](https://www.github.com/tauri-apps/tauri/commit/1efa5e7184009537b688a395596c696173ae0231) ([#11099](https://www.github.com/tauri-apps/tauri/pull/11099) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Rerun build script if the platform-specific configuration file changes.

## \[2.0.0-rc.12]

### New Features

- [`ad294d274`](https://www.github.com/tauri-apps/tauri/commit/ad294d274dd81d2ef91ed73af9163b6e9b8eb964) ([#11032](https://www.github.com/tauri-apps/tauri/pull/11032) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `app > windows > create` option to choose whether to create this window at app startup or not.

## \[2.0.0-rc.11]

### New Features

- [`35bd9dd3d`](https://www.github.com/tauri-apps/tauri/commit/35bd9dd3dc3d8972bbc4aa5f4a6c6fa14354e9bf) ([#10977](https://www.github.com/tauri-apps/tauri/pull/10977) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `mainBinaryName` config option to set the file name for the main binary.

## \[2.0.0-rc.10]

### Bug Fixes

- [`0a47bf043`](https://www.github.com/tauri-apps/tauri/commit/0a47bf04302ca8502d3da21b3bc27818720fe34a) ([#10946](https://www.github.com/tauri-apps/tauri/pull/10946) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue that made the `identifier` in `tauri.conf.json` optional while it was actually required.

## \[2.0.0-rc.9]

### Dependencies

- [`d9c8d3cc8`](https://www.github.com/tauri-apps/tauri/commit/d9c8d3cc8d5ca67cd767ffc7a521f801b41ce201) ([#10902](https://www.github.com/tauri-apps/tauri/pull/10902) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Update infer to 0.16, tray icon to 0.17, urlpattern to 0.3, image to 0.25

### Breaking Changes

- [`faa259bac`](https://www.github.com/tauri-apps/tauri/commit/faa259bacf1ace670af763982c6903190faf163a) ([#10907](https://www.github.com/tauri-apps/tauri/pull/10907) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) The `Assets::iter` function now must return a iterator with `Item = (Cow<'_, str>, Cow<'_, [u8]>)` to be more flexible on contexts where the assets are not `'static`.

## \[2.0.0-rc.8]

### Enhancements

- [`f0acf504a`](https://www.github.com/tauri-apps/tauri/commit/f0acf504a2a972c063188a00143d8bf2b887691d) ([#10858](https://www.github.com/tauri-apps/tauri/pull/10858) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Validate duplicate capability identifier.

## \[2.0.0-rc.7]

### New Features

- [`58dda44a5`](https://www.github.com/tauri-apps/tauri/commit/58dda44a59b915f091602cdfc53385a148469793) ([#10339](https://www.github.com/tauri-apps/tauri/pull/10339) by [@Legend-Master](https://www.github.com/tauri-apps/tauri/../../Legend-Master)) Add a new option `minimumWebview2Version` for Windows NSIS installer to trigger a webview2 update if the user's webview2 is older than this version

### Bug Fixes

- [`03f2a5098`](https://www.github.com/tauri-apps/tauri/commit/03f2a50981b8c01b1c196811fce9d93f1bf0820d) ([#10718](https://www.github.com/tauri-apps/tauri/pull/10718) by [@rdlabo](https://www.github.com/tauri-apps/tauri/../../rdlabo)) Update swift-rs fixing a plugin build when native dependencies are used.

### Breaking Changes

- [`073bb4f45`](https://www.github.com/tauri-apps/tauri/commit/073bb4f459a923541b94970dfa7e087bccaa2cfd) ([#10772](https://www.github.com/tauri-apps/tauri/pull/10772) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Removed the deprecated `webview_fixed_runtime_path` config option, use the `webview_install_mode` instead.

## \[2.0.0-rc.6]

### Bug Fixes

- [`9bcff3cd7`](https://www.github.com/tauri-apps/tauri/commit/9bcff3cd7997fe13425a21b577f93317831f77fa) ([#10703](https://www.github.com/tauri-apps/tauri/pull/10703) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Properly remove isolation script on Android.

### What's Changed

- [`f4d5241b3`](https://www.github.com/tauri-apps/tauri/commit/f4d5241b377d0f7a1b58100ee19f7843384634ac) ([#10731](https://www.github.com/tauri-apps/tauri/pull/10731) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Update documentation icon path.

## \[2.0.0-rc.5]

### Bug Fixes

- [`da381e07f`](https://www.github.com/tauri-apps/tauri/commit/da381e07f3770988fe6d0859a02331b87cc6723f) ([#10696](https://www.github.com/tauri-apps/tauri/pull/10696) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Implemented `resource_dir` on Android, which returns a URI that needs to be resolved using [AssetManager::open](https://developer.android.com/reference/android/content/res/AssetManager#open\(java.lang.String,%20int\)). This will be handled by the file system plugin.
- [`da381e07f`](https://www.github.com/tauri-apps/tauri/commit/da381e07f3770988fe6d0859a02331b87cc6723f) ([#10696](https://www.github.com/tauri-apps/tauri/pull/10696) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix `resource_dir` on iOS.

## \[2.0.0-rc.4]

### New Features

- [`8d148a9e2`](https://www.github.com/tauri-apps/tauri/commit/8d148a9e2566edebfea2d75f32df7c9396d765a4) ([#10634](https://www.github.com/tauri-apps/tauri/pull/10634) by [@anatawa12](https://www.github.com/tauri-apps/tauri/../../anatawa12)) Custom sign command with object notation for whitespaces in the command path and arguments.

### Bug Fixes

- [`5c335ae9a`](https://www.github.com/tauri-apps/tauri/commit/5c335ae9ad88e46c2135a557390f6e808c9a6088) ([#10648](https://www.github.com/tauri-apps/tauri/pull/10648) by [@Flakebi](https://www.github.com/tauri-apps/tauri/../../Flakebi)) Prevent build script from rerunning unnecessarily by only writing files when the content changes.

## \[2.0.0-rc.3]

### Enhancements

- [`0bb7b0f35`](https://www.github.com/tauri-apps/tauri/commit/0bb7b0f352960fb5111a40157c0953d19e76f1fd) ([#10559](https://www.github.com/tauri-apps/tauri/pull/10559) by [@Norbiros](https://www.github.com/tauri-apps/tauri/../../Norbiros)) Return autogenerated permissions from `autogenerate_command_permissions`.

### Bug Fixes

- [`9e891933d`](https://www.github.com/tauri-apps/tauri/commit/9e891933d8ac7a67e37770a149d0a5dd385ee625) ([#10293](https://www.github.com/tauri-apps/tauri/pull/10293) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Fix `ResourcePaths` iterator returning an unexpected result for mapped resources, for example `"../resources/user.json": "resources/user.json"` generates this resource `resources/user.json/user.json` where it should generate just `resources/user.json`.
- [`9fe846615`](https://www.github.com/tauri-apps/tauri/commit/9fe846615b7c4f310f07897ded881c239e3df30a) ([#10547](https://www.github.com/tauri-apps/tauri/pull/10547) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fix plugin permissions documentation heading for permissions table.

### Dependencies

- [`0afee5ed8`](https://www.github.com/tauri-apps/tauri/commit/0afee5ed80265c95c4581e173c4886677cfed593) ([#10436](https://www.github.com/tauri-apps/tauri/pull/10436) by [@nyurik](https://www.github.com/tauri-apps/tauri/../../nyurik)) Updated brotli to v6.

## \[2.0.0-rc.2]

### Bug Fixes

- [`f5dfc0280`](https://www.github.com/tauri-apps/tauri/commit/f5dfc02800dbd3bdee671b032454c49ac7102fb4) ([#10533](https://www.github.com/tauri-apps/tauri/pull/10533) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Fixed an issue causing `tauri ios init` to fail if `iOS.minimumSystemVersion` was not configured explicitly.

## \[2.0.0-rc.1]

### New Features

- [`8dc81b6cc`](https://www.github.com/tauri-apps/tauri/commit/8dc81b6cc2b8235b11f74a971d6aa3a5df5e9f68) ([#10496](https://www.github.com/tauri-apps/tauri/pull/10496) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `bundle > ios > template` configuration option for custom Xcode project YML Handlebars template using XcodeGen.
- [`02c00abc6`](https://www.github.com/tauri-apps/tauri/commit/02c00abc63cf86e9bf9179cbb143d5145a9397b6) ([#10495](https://www.github.com/tauri-apps/tauri/pull/10495) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `bundle > ios > minimumSystemVersion` configuration option.

### Bug Fixes

- [`7e810cb2a`](https://www.github.com/tauri-apps/tauri/commit/7e810cb2a3fd934017ae973e737864dfa4bdf64e) ([#10485](https://www.github.com/tauri-apps/tauri/pull/10485) by [@anatawa12](https://www.github.com/tauri-apps/tauri/../../anatawa12)) Fixed an issue where permission files will be generated with ':' in the file path.

## \[2.0.0-rc.0]

### New Features

- [`a5bfbaa62`](https://www.github.com/tauri-apps/tauri/commit/a5bfbaa62b8cd0aacbb33f730d4e30b43c461fe1)([#9962](https://www.github.com/tauri-apps/tauri/pull/9962)) Added `bundle > iOS > frameworks` configuration to define a list of frameworks that are linked to the Xcode project when it is generated.

### Enhancements

- [`7aeac39e7`](https://www.github.com/tauri-apps/tauri/commit/7aeac39e7fb97dc57ca278f1c097058275c20aa2) ([#10397](https://www.github.com/tauri-apps/tauri/pull/10397)) Make the set of gtk application id optional, to allow more then one instance of the app running at the same time.

### Bug Fixes

- [`498f405ca`](https://www.github.com/tauri-apps/tauri/commit/498f405ca80440447823dd3c9cd53c0f79d655b5) ([#10404](https://www.github.com/tauri-apps/tauri/pull/10404)) Fixed an issue where configuration parsing errors always displayed 'tauri.conf.json' as the file path, even when using 'Tauri.toml' or 'tauri.conf.json5'.

  The error messages now correctly shows the actual config file being used.

### Security fixes

- [`426d14bb4`](https://www.github.com/tauri-apps/tauri/commit/426d14bb4164290d93b5a0f61e925cb2dfc4aafa) ([#10423](https://www.github.com/tauri-apps/tauri/pull/10423)) Explicitly check that the main frame's origin is the sender of Isolation Payloads

## \[2.0.0-beta.19]

### New Features

- [`4c239729c`](https://www.github.com/tauri-apps/tauri/commit/4c239729c3e1b899ecbc6793c3682848e8de1729) ([#10167](https://www.github.com/tauri-apps/tauri/pull/10167) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Add `RawIsolationPayload::content_type` method.

## \[2.0.0-beta.18]

### New Features

- [`fafc238f7`](https://www.github.com/tauri-apps/tauri/commit/fafc238f7288548975ca7d3e5207b925c0295c91) ([#9977](https://www.github.com/tauri-apps/tauri/pull/9977)) Add `bundle > homepage` option, if unset, it will fallback to `homepage` defined in `Cargo.toml`.
- [`656a64974`](https://www.github.com/tauri-apps/tauri/commit/656a64974468bc207bf39537e02ae179bdee9b83) ([#9318](https://www.github.com/tauri-apps/tauri/pull/9318)) Added a configuration option to disable hardened runtime on macOS codesign.
- [`5b769948a`](https://www.github.com/tauri-apps/tauri/commit/5b769948a81cac333f64c870a470ba6525bd5cd3) ([#9959](https://www.github.com/tauri-apps/tauri/pull/9959)) Add `include_image` macro to help embedding instances of `Image` struct at compile-time in rust to be used with window, menu or tray icons.
- [`3ab170917`](https://www.github.com/tauri-apps/tauri/commit/3ab170917ed535fc9013f0a9255631fb34493e18) ([#9932](https://www.github.com/tauri-apps/tauri/pull/9932)) Add an option to disable NSIS compression `bundle > nsis > compression: "none"`
- [`f21029b1b`](https://www.github.com/tauri-apps/tauri/commit/f21029b1bc25f5cb987e1a25de94c2d364e3e462) ([#9994](https://www.github.com/tauri-apps/tauri/pull/9994)) Add `bundle > nsis > startMenuFolder` option to customize start menu folder for NSIS installer

### Enhancements

- [`878198777`](https://www.github.com/tauri-apps/tauri/commit/878198777ef693efdbd394cb4be4b234e8a7ed3d) ([#9999](https://www.github.com/tauri-apps/tauri/pull/9999)) Mark ACL `permissions` array with unique items

### Breaking Changes

- [`3ab170917`](https://www.github.com/tauri-apps/tauri/commit/3ab170917ed535fc9013f0a9255631fb34493e18) ([#9932](https://www.github.com/tauri-apps/tauri/pull/9932)) Changed `NsisSettings::compression` field from `Option<NsisCompression>` to just `NsisCompression`
- [`911242f09`](https://www.github.com/tauri-apps/tauri/commit/911242f0928e0a2add3595fa9de27850fb875fa6) ([#9883](https://www.github.com/tauri-apps/tauri/pull/9883)) Move updater target from `bundle > targets` to a separate field `bundle > createUpdaterArtifacts`
- [`3ab170917`](https://www.github.com/tauri-apps/tauri/commit/3ab170917ed535fc9013f0a9255631fb34493e18) ([#9932](https://www.github.com/tauri-apps/tauri/pull/9932)) Changed `NsisConfig::compression` field from `Option<NsisCompression>` to just `NsisCompression`

## \[2.0.0-beta.17]

### New Features

- [`8a1ae2dea`](https://www.github.com/tauri-apps/tauri/commit/8a1ae2deaf3086e531ada25b1627f900e2e421fb)([#9843](https://www.github.com/tauri-apps/tauri/pull/9843)) Added an option to use a Xcode project for the iOS plugin instead of a plain SwiftPM project.
- [`5462e5cad`](https://www.github.com/tauri-apps/tauri/commit/5462e5cadc73c1b9083d852061d7c7f982cfbe53)([#9731](https://www.github.com/tauri-apps/tauri/pull/9731)) Add `installer_hooks` NSIS configuration field
- [`d6d3efbd1`](https://www.github.com/tauri-apps/tauri/commit/d6d3efbd125489cb46642b6d013cdc1eb7fc1a66)([#9865](https://www.github.com/tauri-apps/tauri/pull/9865)) Add `sign_command` in `WindowsConfig`

### Breaking Changes

- [`fc1543c65`](https://www.github.com/tauri-apps/tauri/commit/fc1543c65e736622bed93543dcc6504c43e200bb)([#9864](https://www.github.com/tauri-apps/tauri/pull/9864)) Removed `skip_webview_install` (`skipWebviewInstall`) option from config, which has been deprecated for a while now and planned to be removed in v2. Use `webview_install_mode` (`webviewInstallMode`) instead.
- [`265c23886`](https://www.github.com/tauri-apps/tauri/commit/265c23886ee5efbcc6d7188ff5c84cb32fa82aea)([#9375](https://www.github.com/tauri-apps/tauri/pull/9375)) Removed `Config::binary_name` and `PackageInfo::package_name`

## \[2.0.0-beta.16]

### Bug Fixes

- [`be95d8d37`](https://www.github.com/tauri-apps/tauri/commit/be95d8d37c4b1a420c0d28a83b7efa40ab0b0ab5)([#9782](https://www.github.com/tauri-apps/tauri/pull/9782)) Fixes the `ToTokens` implementation for `Capability`.

## \[2.0.0-beta.15]

### Bug Fixes

- [`a5205f179`](https://www.github.com/tauri-apps/tauri/commit/a5205f179e577cce5c05b710b597da8f4e1d0780)([#9691](https://www.github.com/tauri-apps/tauri/pull/9691)) Fixes the ToTokens implementation of the window configuration `proxy_url` field.

## \[2.0.0-beta.14]

### Bug Fixes

- [`3fbc1703f`](https://www.github.com/tauri-apps/tauri/commit/3fbc1703f107dfb2c5a75e848805dcf60d449eb1)([#9676](https://www.github.com/tauri-apps/tauri/pull/9676)) Fixes `schemars` compilation issue.

## \[2.0.0-beta.13]

### Bug Fixes

- [`a1e0e268f`](https://www.github.com/tauri-apps/tauri/commit/a1e0e268f02f7e2934bec48de4cac0dc00529a2b)([#9477](https://www.github.com/tauri-apps/tauri/pull/9477)) Replace `tauri:` prefix with `tauri-` for temporary permission file names

## \[2.0.0-beta.12]

### New Features

- [`36b4c1249`](https://www.github.com/tauri-apps/tauri/commit/36b4c12497fbe636066f4848c6877b3ab6cc892e)([#9331](https://www.github.com/tauri-apps/tauri/pull/9331)) Added support for `provides`, `conflicts` and `replaces` (`obsoletes` for RPM) options for `bundler > deb` and `bundler > rpm` configs.

## \[2.0.0-beta.11]

### New Features

- [`259d84529`](https://www.github.com/tauri-apps/tauri/commit/259d845290dde40639537258b2810567910f47f3)([#9209](https://www.github.com/tauri-apps/tauri/pull/9209)) Added `preInstallScript`, `postInstallScript`, `preRemoveScript` and `postRemoveScript` options for `bundler > deb` and `bundler > rpm` configs.

### Enhancements

- [`7c334cb18`](https://www.github.com/tauri-apps/tauri/commit/7c334cb1851ab034a3cfb472dd99dfc61ad3ca7f)([#9327](https://www.github.com/tauri-apps/tauri/pull/9327)) Make the isolation pattern encrypt key unextractable.
- [`a804a70a7`](https://www.github.com/tauri-apps/tauri/commit/a804a70a7aa1dc40fa9043206ad2265c6a5a437b)([#9328](https://www.github.com/tauri-apps/tauri/pull/9328)) The isolation iframe script now removes itself after execution.

### Breaking Changes

- [`06833f4fa`](https://www.github.com/tauri-apps/tauri/commit/06833f4fa8e63ecc55fe3fc874a9e397e77a5709)([#9100](https://www.github.com/tauri-apps/tauri/pull/9100)) Rename `FileDrop` to `DragDrop` on structs, enums and enum variants. Also renamed `file_drop` to `drag_drop` on fields and function names.

## \[2.0.0-beta.10]

### New Features

- [`e227fe02f`](https://www.github.com/tauri-apps/tauri/commit/e227fe02f986e145c0731a64693e1c830a9eb5b0)([#9156](https://www.github.com/tauri-apps/tauri/pull/9156)) Added the `plugin` module.

### Enhancements

- [`7213b9e47`](https://www.github.com/tauri-apps/tauri/commit/7213b9e47242bef814aa7257e0bf84631bf5fe7e)([#9124](https://www.github.com/tauri-apps/tauri/pull/9124)) Fallback to an empty permission set if the plugin did not define its `default` permissions.

## \[2.0.0-beta.9]

### Breaking Changes

- [`490a6b424`](https://www.github.com/tauri-apps/tauri/commit/490a6b424e81714524150aef96fbf6cf7004b940)([#9147](https://www.github.com/tauri-apps/tauri/pull/9147)) Removed the `assets::Assets` trait which is now part of the `tauri` crate.

## \[2.0.0-beta.8]

### Enhancements

- [`3e472d0af`](https://www.github.com/tauri-apps/tauri/commit/3e472d0afcd67545dd6d9f18d304580a3b2759a8)([#9115](https://www.github.com/tauri-apps/tauri/pull/9115)) Changed the permission and capability platforms to be optional.

### Breaking Changes

- [`4ef17d083`](https://www.github.com/tauri-apps/tauri/commit/4ef17d08336a2e0df4a7ef9adea746d7419710b6)([#9116](https://www.github.com/tauri-apps/tauri/pull/9116)) The ACL configuration for remote URLs now uses the URLPattern standard instead of glob patterns.

## \[2.0.0-beta.7]

### Bug Fixes

- [`86fa339de`](https://www.github.com/tauri-apps/tauri/commit/86fa339de7b176efafa9b3e89f94dcad5fcd03da)([#9071](https://www.github.com/tauri-apps/tauri/pull/9071)) Fix compile time error in context generation when using `app.windows.windowEffects.color`
- [`6c0683224`](https://www.github.com/tauri-apps/tauri/commit/6c068322460300e9d56a4fac5b018d4c437daa9e)([#9068](https://www.github.com/tauri-apps/tauri/pull/9068)) Fixes scope resolution grouping scopes for all windows.
- [`c68218b36`](https://www.github.com/tauri-apps/tauri/commit/c68218b362c417b62e56c7a2b5b32c13fe035a83)([#8990](https://www.github.com/tauri-apps/tauri/pull/8990)) Fix `BundleTarget::to_vec` returning an empty vec for `BundleTarget::All` variant.
- [`c68218b36`](https://www.github.com/tauri-apps/tauri/commit/c68218b362c417b62e56c7a2b5b32c13fe035a83)([#8990](https://www.github.com/tauri-apps/tauri/pull/8990)) Add `BundleType::all` method to return all possible `BundleType` variants.

### Breaking Changes

- [`9aa0d6e95`](https://www.github.com/tauri-apps/tauri/commit/9aa0d6e959269a9d99ff474e7f12bd397ea75fcd)([#9069](https://www.github.com/tauri-apps/tauri/pull/9069)) Removed `debug_eprintln!` and `consume_unused_variable` macros.
- [`bb23511ea`](https://www.github.com/tauri-apps/tauri/commit/bb23511ea80bcaffbdebf057301e463fff268c90)([#9079](https://www.github.com/tauri-apps/tauri/pull/9079)) Changed `CapabiltyFile::List` enum variant to be a tuple-struct and added `CapabiltyFile::NamedList`. This allows more flexibility when parsing capabilties from JSON files.

## \[2.0.0-beta.6]

### New Features

- [`d7f56fef`](https://www.github.com/tauri-apps/tauri/commit/d7f56fef85cac3af4e2dbac1eac40e5567b1f160)([#9014](https://www.github.com/tauri-apps/tauri/pull/9014)) Allow defining a permission that only applies to a set of target platforms via the `platforms` configuration option.

### Enhancements

- [`04440edc`](https://www.github.com/tauri-apps/tauri/commit/04440edce870f9d06055616034941d79443d5a87)([#9019](https://www.github.com/tauri-apps/tauri/pull/9019)) Changed plugin markdown docs generation to table format.

### Breaking Changes

- [`3657ad82`](https://www.github.com/tauri-apps/tauri/commit/3657ad82f88ce528551d032d521c52eed3f396b4)([#9008](https://www.github.com/tauri-apps/tauri/pull/9008)) Allow defining permissions for the application commands via `tauri_build::Attributes::app_manifest`.

## \[2.0.0-beta.5]

### Enhancements

- [`bc5b5e67`](https://www.github.com/tauri-apps/tauri/commit/bc5b5e671a546512f823f1c157421b4c3311dfc0)([#8984](https://www.github.com/tauri-apps/tauri/pull/8984)) Do not include a CSP tag in the application HTML and rely on the custom protocol response header instead.

## \[2.0.0-beta.4]

### Breaking Changes

- [`a76fb118`](https://www.github.com/tauri-apps/tauri/commit/a76fb118ce2de22e1bdb4216bf0ac01dfc3e5799)([#8950](https://www.github.com/tauri-apps/tauri/pull/8950)) Changed the capability format to allow configuring both `remote: { urls: Vec<String> }` and `local: bool (default: true)` instead of choosing one on the `context` field.

## \[2.0.0-beta.3]

### Breaking Changes

- [`361ec37f`](https://www.github.com/tauri-apps/tauri/commit/361ec37fd4a5caa5b6630b9563ef079f53c6c336)([#8932](https://www.github.com/tauri-apps/tauri/pull/8932)) Moved `ProgressBarState` from `tauri-utils` to the `tauri::window` module and removed the `unity_uri` field.

## \[2.0.0-beta.2]

### Enhancements

- [`0cb0a15c`](https://www.github.com/tauri-apps/tauri/commit/0cb0a15ce22af3d649cf219ac04188c14c5f4905)([#8789](https://www.github.com/tauri-apps/tauri/pull/8789)) Add `webviews` array on the capability for usage on multiwebview contexts.
- [`83a68deb`](https://www.github.com/tauri-apps/tauri/commit/83a68deb5676d39cd4728d2e140f6b46d5f787ed)([#8797](https://www.github.com/tauri-apps/tauri/pull/8797)) Added a new configuration option `tauri.conf.json > app > security > capabilities` to reference existing capabilities and inline new ones. If it is empty, all capabilities are still included preserving the current behavior.
- [`8d16a80d`](https://www.github.com/tauri-apps/tauri/commit/8d16a80d2fb2468667e7987d0cc99dbc7e3b9d0a)([#8802](https://www.github.com/tauri-apps/tauri/pull/8802)) The `Context` struct now includes the runtime authority instead of the resolved ACL. This does not impact most applications.
- [`28fb036c`](https://www.github.com/tauri-apps/tauri/commit/28fb036ce476c6f22815c35385f923135212c6f3)([#8852](https://www.github.com/tauri-apps/tauri/pull/8852)) Enhance resource directory resolution on development.
- [`dd7571a7`](https://www.github.com/tauri-apps/tauri/commit/dd7571a7808676c8063a4983b9c6687dfaf03a09)([#8815](https://www.github.com/tauri-apps/tauri/pull/8815)) Do not generate JSON schema and markdown reference file if the plugin does not define any permissions and delete those files if they exist.
- [`5618f6d2`](https://www.github.com/tauri-apps/tauri/commit/5618f6d2ffc9ebf40710145538b06bebfa55f878)([#8856](https://www.github.com/tauri-apps/tauri/pull/8856)) Relax requirements on plugin's identifiers to be alphanumeric and `-` instead of only lower alpha and `-`.
- [`8d16a80d`](https://www.github.com/tauri-apps/tauri/commit/8d16a80d2fb2468667e7987d0cc99dbc7e3b9d0a)([#8802](https://www.github.com/tauri-apps/tauri/pull/8802)) Refactored the capability types and resolution algorithm.

### Bug Fixes

- [`ae0fe47c`](https://www.github.com/tauri-apps/tauri/commit/ae0fe47c4c35fa87c77acf42af32ef3f0615cb08)([#8774](https://www.github.com/tauri-apps/tauri/pull/8774)) Fix compile error when `tauri.conf.json` had `bundle > license` set.

### Breaking Changes

- [`f284f9c5`](https://www.github.com/tauri-apps/tauri/commit/f284f9c545deeb77d15b6e8b1d0d05f49c40634c)([#8898](https://www.github.com/tauri-apps/tauri/pull/8898)) Changed the capability `remote` configuration to take a list of `urls` instead of `domains` for more flexibility.

## \[2.0.0-beta.1]

### Enhancements

- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Moved the capability JSON schema to the `src-tauri/gen` folder so it's easier to track changes on the `capabilities` folder.

### Bug Fixes

- [`4e101f80`](https://www.github.com/tauri-apps/tauri/commit/4e101f801657e7d01ce8c22f9c6468067d0caab2)([#8756](https://www.github.com/tauri-apps/tauri/pull/8756)) Rerun build script when a new permission is added.

## \[2.0.0-beta.0]

### New Features

- [`74a2a603`](https://www.github.com/tauri-apps/tauri/commit/74a2a6036a5e57462f161d728cbd8a6f121028ca)([#8661](https://www.github.com/tauri-apps/tauri/pull/8661)) Implement access control list for IPC usage.
- [`9eaeb5a8`](https://www.github.com/tauri-apps/tauri/commit/9eaeb5a8cd95ae24b5e66205bdc2763cb7f965ce)([#8622](https://www.github.com/tauri-apps/tauri/pull/8622)) Add `parent` option for window config.
- [`58fe2e81`](https://www.github.com/tauri-apps/tauri/commit/58fe2e812a85b9f4eba105286a63f271ea637836)([#8670](https://www.github.com/tauri-apps/tauri/pull/8670)) Add `WebviewUrl::CustomProtocol` enum variant.

### What's Changed

- [`6639a579`](https://www.github.com/tauri-apps/tauri/commit/6639a579c76d45210f33a72d37e21d4c5a9d334b)([#8441](https://www.github.com/tauri-apps/tauri/pull/8441)) Added the `WindowConfig::proxy_url` `WebviewBuilder::proxy_url() / WebviewWindowBuilder::proxy_url()` options when creating a webview.

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
- [`c77b4032`](https://www.github.com/tauri-apps/tauri/commit/c77b40324ea9bf580871fc11aed69ba0c9b6b8cf)([#8280](https://www.github.com/tauri-apps/tauri/pull/8280)) Renamed `config::WindowUrl` to `config::WebviewUrl`.
- [`a093682d`](https://www.github.com/tauri-apps/tauri/commit/a093682d2df7169b024bb4f736c7f1fd2ea8b327)([#8621](https://www.github.com/tauri-apps/tauri/pull/8621)) Changed `error` field in `ConfigError::FormatToml` to be boxed `Box<toml::de::Error>` to reduce the enum `ConfigError` size in memory.
- [`58fe2e81`](https://www.github.com/tauri-apps/tauri/commit/58fe2e812a85b9f4eba105286a63f271ea637836)([#8670](https://www.github.com/tauri-apps/tauri/pull/8670)) Changed `dist_dir` and `dev_path` config options to be optional.

## \[2.0.0-alpha.13]

### Bug Fixes

- [`9b230de7`](https://www.github.com/tauri-apps/tauri/commit/9b230de7bc6690c2733f5324d50b999af1f7a6ef)([#8407](https://www.github.com/tauri-apps/tauri/pull/8407)) Fix compile error when parsing config that includes float values.

## \[2.0.0-alpha.12]

### New Features

- [\`\`](https://www.github.com/tauri-apps/tauri/commit/undefined) Add bundle DMG configuration options.

## \[2.0.0-alpha.11]

### Breaking Changes

- [`5e84e92e`](https://www.github.com/tauri-apps/tauri/commit/5e84e92e99376f24b730f8eba002239379b593e1)([#8243](https://www.github.com/tauri-apps/tauri/pull/8243)) Changed `platform::windows_version` to return a `(u32, u32, u32)` instead of `Option<(u32, u32, u32)>`

## \[2.0.0-alpha.10]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.

### Dependencies

- [`c7c2507d`](https://www.github.com/tauri-apps/tauri/commit/c7c2507da16a9beb71bf06745fe7ac1325ab7c2a)([#8035](https://www.github.com/tauri-apps/tauri/pull/8035)) Update `windows` to version `0.51` and `webview2-com` to version `0.27`

## \[2.0.0-alpha.9]

### New Features

- [`c085adda`](https://www.github.com/tauri-apps/tauri/commit/c085addab58ba851398373c6fd13f9cb026d71e8)([#8009](https://www.github.com/tauri-apps/tauri/pull/8009)) Added `set_progress_bar` to `Window`.
- [`c1ec0f15`](https://www.github.com/tauri-apps/tauri/commit/c1ec0f155118527361dd5645d920becbc8afd569)([#7933](https://www.github.com/tauri-apps/tauri/pull/7933)) Added the `always_on_bottom` option to the window configuration.
- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.
- [`ed32257d`](https://www.github.com/tauri-apps/tauri/commit/ed32257d044f90b5eb15053efd1667125def2d2b)([#7794](https://www.github.com/tauri-apps/tauri/pull/7794)) On Windows, add `WindowEffect::Tabbed`,`WindowEffect::TabbedDark` and `WindowEffect::TabbedLight`

### Breaking Changes

- [`ebcc21e4`](https://www.github.com/tauri-apps/tauri/commit/ebcc21e4b95f4e8c27639fb1bca545b432f52d5e)([#8057](https://www.github.com/tauri-apps/tauri/pull/8057)) Renamed the beforeDevCommand, beforeBuildCommand and beforeBundleCommand hooks environment variables from `TAURI_PLATFORM, TAURI_ARCH, TAURI_FAMILY, TAURI_PLATFORM_VERSION, TAURI_PLATFORM_TYPE and TAURI_DEBUG` to `TAURI_ENV_PLATFORM, TAURI_ENV_ARCH, TAURI_ENV_FAMILY, TAURI_ENV_PLATFORM_VERSION, TAURI_ENV_PLATFORM_TYPE and TAURI_ENV_DEBUG` to differentiate the prefix with other CLI environment variables.

## \[2.0.0-alpha.8]

### Enhancements

- Add an option to specify `id` for the tray icon in the tauri configuration file.

### Breaking Changes

- [`100d9ede`](https://www.github.com/tauri-apps/tauri/commit/100d9ede35995d9db21d2087dd5606adfafb89a5)([#7802](https://www.github.com/tauri-apps/tauri/pull/7802)) Follow file name conventions set by desktop for mobile Tauri configuration files. Added `target` argument on most `config::parse` methods.

## \[2.0.0-alpha.7]

### New Features

- [`4db363a0`](https://www.github.com/tauri-apps/tauri/commit/4db363a03c182349f8491f46ced258d84723b11f)([#6589](https://www.github.com/tauri-apps/tauri/pull/6589)) Added `visible_on_all_workspaces` configuration option to `WindowBuilder`, `Window`, and `WindowConfig`.
- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) Add option to specify a tooltip text for the tray icon in the config.
- [`74b1f4fc`](https://www.github.com/tauri-apps/tauri/commit/74b1f4fc6625d5b4f9b86f70e4eebd6551c61809)([#7384](https://www.github.com/tauri-apps/tauri/pull/7384)) Add `WindowEffect::MicaDark` and `WindowEffect::MicaLight`
- [`3b98141a`](https://www.github.com/tauri-apps/tauri/commit/3b98141aa26f74c641a4090874247b97079bd58a)([#3736](https://www.github.com/tauri-apps/tauri/pull/3736)) Add a configuration object for file associations under `BundleConfig`.

### Enhancements

- [`fbeb5b91`](https://www.github.com/tauri-apps/tauri/commit/fbeb5b9185baeda19e865228179e3e44c165f1d9)([#7170](https://www.github.com/tauri-apps/tauri/pull/7170)) Use custom protocols on the IPC implementation to enhance performance.

### Security fixes

- [`43c6285e`](https://www.github.com/tauri-apps/tauri/commit/43c6285e9006fb84066461d57fe09ea8db76d636)([#7359](https://www.github.com/tauri-apps/tauri/pull/7359)) Changed HTML implementation from unmaintained `kuchiki` to `kuchikiki`.

### Breaking Changes

- [`7fb419c3`](https://www.github.com/tauri-apps/tauri/commit/7fb419c326aaf72ecd556d8404377444ebb200e7)([#7535](https://www.github.com/tauri-apps/tauri/pull/7535)) `systemTray` config option has been renamed to `trayIcon`.

## \[2.0.0-alpha.6]

### New Features

- [`e0f0dce2`](https://www.github.com/tauri-apps/tauri/commit/e0f0dce220730e2822fc202463aedf0166145de7)([#6442](https://www.github.com/tauri-apps/tauri/pull/6442)) Added the `window_effects` option to the window configuration.

## \[2.0.0-alpha.5]

- [`9a79dc08`](https://www.github.com/tauri-apps/tauri/commit/9a79dc085870e0c1a5df13481ff271b8c6cc3b78)([#6947](https://www.github.com/tauri-apps/tauri/pull/6947)) Remove `enable_tauri_api` from the IPC scope.
- [`09376af5`](https://www.github.com/tauri-apps/tauri/commit/09376af59424cc27803fa2820d2ac0d4cdc90a6d)([#6704](https://www.github.com/tauri-apps/tauri/pull/6704)) Moved the `cli` feature to its own plugin in the plugins-workspace repository.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the `protocol` scope configuration to the `asset_protocol` field in `SecurityConfig`.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Moved the updater configuration to the `BundleConfig`.
- [`b072daa3`](https://www.github.com/tauri-apps/tauri/commit/b072daa3bd3e38b808466666619ddb885052c5b2)([#6919](https://www.github.com/tauri-apps/tauri/pull/6919)) Moved the `updater` feature to its own plugin in the plugins-workspace repository.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`e1e85dc2`](https://www.github.com/tauri-apps/tauri/commit/e1e85dc2a5f656fc37867e278cae8042037740ac)([#6925](https://www.github.com/tauri-apps/tauri/pull/6925)) Removed the allowlist configuration.
- [`2d5378bf`](https://www.github.com/tauri-apps/tauri/commit/2d5378bfc1ba817ee2f331b41738a90e5997e5e8)([#6717](https://www.github.com/tauri-apps/tauri/pull/6717)) Remove the updater's dialog option.

## \[2.0.0-alpha.4]

- Added `android` configuration object under `tauri > bundle`.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07

## \[2.0.0-alpha.3]

- Pull changes from Tauri 1.3 release.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined

## \[2.0.0-alpha.2]

- Added the `shadow` option to the window configuration and `set_shadow` option to the `window` allow list.
  - [a81750d7](https://www.github.com/tauri-apps/tauri/commit/a81750d779bc72f0fdb7de90b7fbddfd8049b328) feat(core): add shadow APIs ([#6206](https://www.github.com/tauri-apps/tauri/pull/6206)) on 2023-02-08

## \[2.0.0-alpha.1]

- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30
- Added `crate_name` field on `PackageInfo`.
  - [630a7f4b](https://www.github.com/tauri-apps/tauri/commit/630a7f4b18cef169bfd48673609306fec434e397) refactor: remove mobile log initialization, ref [#6049](https://www.github.com/tauri-apps/tauri/pull/6049) ([#6081](https://www.github.com/tauri-apps/tauri/pull/6081)) on 2023-01-17

## \[2.0.0-alpha.0]

- Parse `android` and `ios` Tauri configuration files.
  - [b3a3afc7](https://www.github.com/tauri-apps/tauri/commit/b3a3afc7de8de4021d73559288f5192732a706cf) feat(core): detect android and ios platform configuration files ([#4997](https://www.github.com/tauri-apps/tauri/pull/4997)) on 2022-08-22
- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.5.3]

### New features

- [`7aa30dec`](https://www.github.com/tauri-apps/tauri/commit/7aa30dec85a17c3d3faaf3841b93e10991b991b0)([#8620](https://www.github.com/tauri-apps/tauri/pull/8620)) Add `priority`, `section` and `changelog` options in Debian config.

## \[1.5.2]

### Bug Fixes

- [`9b230de7`](https://www.github.com/tauri-apps/tauri/commit/9b230de7bc6690c2733f5324d50b999af1f7a6ef)([#8407](https://www.github.com/tauri-apps/tauri/pull/8407)) Fix compile error when parsing config that includes float values.

## \[1.5.3]

### New Features

- [`b3e53e72`](https://www.github.com/tauri-apps/tauri/commit/b3e53e7243311a2659b7569dddc20c56ac9f9d8e)([#8288](https://www.github.com/tauri-apps/tauri/pull/8288)) Added `Assets::iter` to iterate on all embedded assets.

## \[1.5.0]

### New Features

- [`4dd4893d`](https://www.github.com/tauri-apps/tauri/commit/4dd4893d7d166ac3a3b6dc2e3bd2540326352a78)([#5950](https://www.github.com/tauri-apps/tauri/pull/5950)) Allow specifying resources as a map specifying source and target paths.

### Enhancements

- [`9aa34ada`](https://www.github.com/tauri-apps/tauri/commit/9aa34ada5769dbefa7dfe5f7a6288b3d20b294e4)([#7645](https://www.github.com/tauri-apps/tauri/pull/7645)) Add setting to switch to `http://<scheme>.localhost/` for custom protocols on Windows.

### Bug Fixes

- [`a6b52e44`](https://www.github.com/tauri-apps/tauri/commit/a6b52e44f22844009e273fb0250368d7a463f095)([#6519](https://www.github.com/tauri-apps/tauri/pull/6519)) Fix `io::read_line` not including the new line character `\n`.

### Security fixes

- [`eeff1784`](https://www.github.com/tauri-apps/tauri/commit/eeff1784e1ffa568e4ba024e17dd611f8e086784)([#7367](https://www.github.com/tauri-apps/tauri/pull/7367)) Changed HTML implementation from unmaintained `kuchiki` to `kuchikiki`.

## \[1.4.0]

### New Features

- [`acc36fe1`](https://www.github.com/tauri-apps/tauri/commit/acc36fe1176cc8aa9063bde932abeb94796c5c72)([#6158](https://www.github.com/tauri-apps/tauri/pull/6158)) Add option to configure `require_literal_leading_dot` on `fs` and `asset` protocol scopes.
- [`35cd751a`](https://www.github.com/tauri-apps/tauri/commit/35cd751adc6fef1f792696fa0cfb471b0bf99374)([#5176](https://www.github.com/tauri-apps/tauri/pull/5176)) Added the `desktop_template` option on `tauri.conf.json > tauri > bundle > deb`.
- [`c4d6fb4b`](https://www.github.com/tauri-apps/tauri/commit/c4d6fb4b1ea8acf02707a9fe5dcab47c1c5bae7b)([#2353](https://www.github.com/tauri-apps/tauri/pull/2353)) Added the `maximizable`, `minimizable` and `closable` options to the window configuration.
- [`3cb7a3e6`](https://www.github.com/tauri-apps/tauri/commit/3cb7a3e642bb10ee90dc1d24daa48b8c8c15c9ce)([#6997](https://www.github.com/tauri-apps/tauri/pull/6997)) Add `MimeType::parse_with_fallback` and `MimeType::parse_from_uri_with_fallback`
- [`29488205`](https://www.github.com/tauri-apps/tauri/commit/2948820579d20dfaa0861c2f0a58bd7737a7ffd1)([#6867](https://www.github.com/tauri-apps/tauri/pull/6867)) Allow specifying custom language files of Tauri's custom messages for the NSIS installer
- [`e092f799`](https://www.github.com/tauri-apps/tauri/commit/e092f799469ff32c7d1595d0f07d06fd2dab5c29)([#6887](https://www.github.com/tauri-apps/tauri/pull/6887)) Add `nsis > template` option to specify custom NSIS installer template.
- [`cd3846c8`](https://www.github.com/tauri-apps/tauri/commit/cd3846c8ce61ab2879b3911e831525e6242aaab2)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) Add `WindowsUpdateInstallMode::nsis_args`
- [`85e77fb7`](https://www.github.com/tauri-apps/tauri/commit/85e77fb797ec17882f55d0944578d929fc6c9c1f)([#6762](https://www.github.com/tauri-apps/tauri/pull/6762)) Correctly determine MIME type of `.txt` files.

## \[1.3.0]

- Added the `additional_browser_args` option to the window configuration.
  - [3dc38b15](https://www.github.com/tauri-apps/tauri/commit/3dc38b150ea8c59c8ba67fd586f921016928f47c) feat(core): expose additional_browser_args to window config (fix: [#5757](https://www.github.com/tauri-apps/tauri/pull/5757)) ([#5799](https://www.github.com/tauri-apps/tauri/pull/5799)) on 2022-12-14
- Added the `content_protected` option to the window configuration.
  - [4ab5545b](https://www.github.com/tauri-apps/tauri/commit/4ab5545b7a831c549f3c65e74de487ede3ab7ce5) feat: add content protection api, closes [#5132](https://www.github.com/tauri-apps/tauri/pull/5132) ([#5513](https://www.github.com/tauri-apps/tauri/pull/5513)) on 2022-12-13
- Correctly determine mime type of `.less`, `.sass` and `.styl` files.
  - [5fdf8dcb](https://www.github.com/tauri-apps/tauri/commit/5fdf8dcb8ed171d06121dceb32078a7e4f86cc64) fix(core): mime type of .less, .sass and .styl files ([#6316](https://www.github.com/tauri-apps/tauri/pull/6316)) on 2023-02-19
- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31
- Add initial support for building `nsis` bundles on non-Windows platforms.
  - [60e6f6c3](https://www.github.com/tauri-apps/tauri/commit/60e6f6c3f1605f3064b5bb177992530ff788ccf0) feat(bundler): Add support for creating NSIS bundles on unix hosts ([#5788](https://www.github.com/tauri-apps/tauri/pull/5788)) on 2023-01-19
- Add `nsis` bundle target
  - [c94e1326](https://www.github.com/tauri-apps/tauri/commit/c94e1326a7c0767a13128a8b1d327a00156ece12) feat(bundler): add `nsis`, closes [#4450](https://www.github.com/tauri-apps/tauri/pull/4450), closes [#2319](https://www.github.com/tauri-apps/tauri/pull/2319) ([#4674](https://www.github.com/tauri-apps/tauri/pull/4674)) on 2023-01-03
- Added configuration to specify remote URLs allowed to access the IPC.
  - [ee71c31f](https://www.github.com/tauri-apps/tauri/commit/ee71c31fd09cc5427da6d29d37c003a914547696) feat(core): allow configuring remote domains with IPC access, closes [#5088](https://www.github.com/tauri-apps/tauri/pull/5088) ([#5918](https://www.github.com/tauri-apps/tauri/pull/5918)) on 2023-04-11

## \[1.2.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[1.2.0]

- Validate `package > productName` in the tauri config and produce errors if it contains one of the following characters `/\:*?\"<>|`
  - [b9316a64](https://www.github.com/tauri-apps/tauri/commit/b9316a64eaa9348c79efafb8b94960d9b4d5b27a) fix(cli): validate `productName` in config, closes [#5233](https://www.github.com/tauri-apps/tauri/pull/5233) ([#5262](https://www.github.com/tauri-apps/tauri/pull/5262)) on 2022-09-28
- Properly serialize HTML template tags.
  - [aec5537d](https://www.github.com/tauri-apps/tauri/commit/aec5537de0205f62b2ae5c89da04d21930a6fc2e) fix(codegen): serialize template tags, closes [#4410](https://www.github.com/tauri-apps/tauri/pull/4410) ([#5247](https://www.github.com/tauri-apps/tauri/pull/5247)) on 2022-09-28
- `PatternKind::Isolation` is now defined even without the `isolation` feature.
  - [a178f95d](https://www.github.com/tauri-apps/tauri/commit/a178f95d68b773779b40235a3a22115a5e36aa6a) feat: config schema generator ([#5193](https://www.github.com/tauri-apps/tauri/pull/5193)) on 2022-10-28
- Added the `app` allowlist module.
  - [39bf895b](https://www.github.com/tauri-apps/tauri/commit/39bf895b73ec6b53f5758815396ba85dda6b9c67) feat(macOS): Add application `show` and `hide` methods ([#3689](https://www.github.com/tauri-apps/tauri/pull/3689)) on 2022-10-03
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Add `tauri.conf.json > bundle > publisher` field to specify the app publisher.
  - [628285c1](https://www.github.com/tauri-apps/tauri/commit/628285c1cf43f03ed62378f3b6cc0c991317526f) feat(bundler): add `publisher` field, closes [#5273](https://www.github.com/tauri-apps/tauri/pull/5273) ([#5283](https://www.github.com/tauri-apps/tauri/pull/5283)) on 2022-09-28
- Canonicalize the return value of `platform::resource_dir`.
  - [a06dc699](https://www.github.com/tauri-apps/tauri/commit/a06dc6993148f10ff7623c9dcc81f313dd960ad0) fix(core): canonicalize resource dir to fix scope check, closes [#5196](https://www.github.com/tauri-apps/tauri/pull/5196) ([#5218](https://www.github.com/tauri-apps/tauri/pull/5218)) on 2022-09-29
- Added `title` option on the system tray configuration (macOS only).
  - [8f1ace77](https://www.github.com/tauri-apps/tauri/commit/8f1ace77956ac3477826ceb059a191e55b3fff93) feat: expose `set_title` for MacOS tray ([#5182](https://www.github.com/tauri-apps/tauri/pull/5182)) on 2022-09-30
- Added the `user_agent` option to the window configuration.
  - [a6c94119](https://www.github.com/tauri-apps/tauri/commit/a6c94119d8545d509723b147c273ca5edfe3729f) feat(core): expose user_agent to window config ([#5317](https://www.github.com/tauri-apps/tauri/pull/5317)) on 2022-10-02
- Add `mime_type` module.
  - [54c337e0](https://www.github.com/tauri-apps/tauri/commit/54c337e06f3bc624c4780cf002bc54790f446c90) feat(cli): hotreload support for frontend static files, closes [#2173](https://www.github.com/tauri-apps/tauri/pull/2173) ([#5256](https://www.github.com/tauri-apps/tauri/pull/5256)) on 2022-09-28

## \[1.1.1]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.1.0]

- Allow adding `build > beforeBundleCommand` in tauri.conf.json to run a shell command before the bundling phase.
  - [57ab9847](https://www.github.com/tauri-apps/tauri/commit/57ab9847eb2d8c9a5da584b873b7c072e9ee26bf) feat(cli): add `beforeBundleCommand`, closes [#4879](https://www.github.com/tauri-apps/tauri/pull/4879) ([#4893](https://www.github.com/tauri-apps/tauri/pull/4893)) on 2022-08-09
- Change `before_dev_command` and `before_build_command` config value to allow configuring the current working directory.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Allow configuring the `before_dev_command` to force the CLI to wait for the command to finish before proceeding.
  - [d6f7d3cf](https://www.github.com/tauri-apps/tauri/commit/d6f7d3cfe8a7ec8d68c8341016c4e0a3103da587) Add cwd option to `before` commands, add wait option to dev [#4740](https://www.github.com/tauri-apps/tauri/pull/4740) [#3551](https://www.github.com/tauri-apps/tauri/pull/3551) ([#4834](https://www.github.com/tauri-apps/tauri/pull/4834)) on 2022-08-02
- Added support to configuration files in TOML format (Tauri.toml file).
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Refactored the `config::parse` module.
  - [ae83d008](https://www.github.com/tauri-apps/tauri/commit/ae83d008f9e1b89bfc8dddaca42aa5c1fbc36f6d) feat: add support to TOML config file `Tauri.toml`, closes [#4806](https://www.github.com/tauri-apps/tauri/pull/4806) ([#4813](https://www.github.com/tauri-apps/tauri/pull/4813)) on 2022-08-02
- Update windows to 0.39.0 and webview2-com to 0.19.1.
  - [e6d9b670](https://www.github.com/tauri-apps/tauri/commit/e6d9b670b0b314ed667b0e164f2c8d27048e678f) refactor: remove unneeded focus code ([#5065](https://www.github.com/tauri-apps/tauri/pull/5065)) on 2022-09-03

## \[1.0.3]

- Added `menu_on_left_click: bool` to the `SystemTrayConfig`.
  - [f8a3becb](https://www.github.com/tauri-apps/tauri/commit/f8a3becb287942db7f7b551b5db6aeb5a2e939ee) feat(core): add option to disable tray menu on left click, closes [#4584](https://www.github.com/tauri-apps/tauri/pull/4584) ([#4587](https://www.github.com/tauri-apps/tauri/pull/4587)) on 2022-07-05
- Added `config::parse::read_platform` and `config::parse::get_platform_config_filename`.
  - [8e3e7fc6](https://www.github.com/tauri-apps/tauri/commit/8e3e7fc64641afc7a6833bc93205e6f525562545) feat(cli): improve bundle identifier validation, closes [#4589](https://www.github.com/tauri-apps/tauri/pull/4589) ([#4596](https://www.github.com/tauri-apps/tauri/pull/4596)) on 2022-07-05

## \[1.0.2]

- Expose `platform::windows_version` function.
  - [bf764e83](https://www.github.com/tauri-apps/tauri/commit/bf764e83e01e7443e6cc54572001e1c98c357465) feat(utils): expose `windows_version` function ([#4534](https://www.github.com/tauri-apps/tauri/pull/4534)) on 2022-06-30

## \[1.0.1]

- Changed the `BundleConfig::targets` to a `BundleTarget` enum to enhance generated documentation.
  - [31c15cd2](https://www.github.com/tauri-apps/tauri/commit/31c15cd2bd94dbe39fb94982a15cbe02ac5d8925) docs(config): enhance documentation for bundle targets, closes [#3251](https://www.github.com/tauri-apps/tauri/pull/3251) ([#4418](https://www.github.com/tauri-apps/tauri/pull/4418)) on 2022-06-21
- Added `platform::is_windows_7`.
  - [57039fb2](https://www.github.com/tauri-apps/tauri/commit/57039fb2166571de85271b014a8711a29f06be1a) fix(core): add windows 7 notification support ([#4491](https://www.github.com/tauri-apps/tauri/pull/4491)) on 2022-06-28
- Suppress unused variable warning in release builds.
  - [45981851](https://www.github.com/tauri-apps/tauri/commit/45981851e35119266c1a079e1ff27a39f1fdfaed) chore(lint): unused variable warnings for release builds ([#4411](https://www.github.com/tauri-apps/tauri/pull/4411)) on 2022-06-22
- Added webview install mode options.
  - [2ca762d2](https://www.github.com/tauri-apps/tauri/commit/2ca762d207030a892a6d128b519e150e2d733468) feat(bundler): extend webview2 installation options, closes [#2882](https://www.github.com/tauri-apps/tauri/pull/2882) [#2452](https://www.github.com/tauri-apps/tauri/pull/2452) ([#4466](https://www.github.com/tauri-apps/tauri/pull/4466)) on 2022-06-26

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.9]

- Added a config flag to bundle the media framework used by webkit2gtk `tauri.conf.json > tauri > bundle > appimage > bundleMediaFramework`.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10

## \[1.0.0-rc.8]

- **Breaking change:** `PackageInfo::version` is now a `semver::Version` instead of a `String`.
  - [2badbd2d](https://www.github.com/tauri-apps/tauri/commit/2badbd2d7ed51bf33c1b547b4c837b600574bd4a) refactor: force semver versions, change updater `should_install` sig ([#4215](https://www.github.com/tauri-apps/tauri/pull/4215)) on 2022-05-25
  - [a7388e23](https://www.github.com/tauri-apps/tauri/commit/a7388e23c3b9019d48b078cae00a75c74d74d11b) fix(ci): adjust change file to include tauri-utils and tauri-codegen on 2022-05-27

## \[1.0.0-rc.7]

- Allow configuring the display options for the MSI execution allowing quieter updates.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.6]

- Added `$schema` support to `tauri.conf.json`.
  - [715cbde3](https://www.github.com/tauri-apps/tauri/commit/715cbde3842a916c4ebeab2cab348e1774b5c192) feat(config): add `$schema` to `tauri.conf.json`, closes [#3464](https://www.github.com/tauri-apps/tauri/pull/3464) ([#4031](https://www.github.com/tauri-apps/tauri/pull/4031)) on 2022-05-03
- The `dangerous_allow_asset_csp_modification` configuration value has been changed to allow a list of CSP directives to disable.
  - [164078c0](https://www.github.com/tauri-apps/tauri/commit/164078c0b719ccbc12e956fecf8a7d4a3c5044e1) feat: allow limiting dangerousDisableAssetCspModification, closes [#3831](https://www.github.com/tauri-apps/tauri/pull/3831) ([#4021](https://www.github.com/tauri-apps/tauri/pull/4021)) on 2022-05-02

## \[1.0.0-rc.5]

- Added the `io` module with the `read_line` method.
  - [b5622882](https://www.github.com/tauri-apps/tauri/commit/b5622882cf3748e1e5a90915f415c0cd922aaaf8) fix(cli): exit on non-compilation Cargo errors, closes [#3930](https://www.github.com/tauri-apps/tauri/pull/3930) ([#3942](https://www.github.com/tauri-apps/tauri/pull/3942)) on 2022-04-22
- **Breaking change:** Removed the `useBootstrapper` option. Use https://github.com/tauri-apps/fix-path-env-rs instead.
  - [6a5ff08c](https://www.github.com/tauri-apps/tauri/commit/6a5ff08ce9052b656aa40accedfd4315825164a3) refactor: remove bootstrapper, closes [#3786](https://www.github.com/tauri-apps/tauri/pull/3786) ([#3832](https://www.github.com/tauri-apps/tauri/pull/3832)) on 2022-03-31

## \[1.0.0-rc.4]

- Added an option to disable the CSP injection of distributable assets nonces and hashes.
  - [f6e32ee1](https://www.github.com/tauri-apps/tauri/commit/f6e32ee1880eb364ed76beb937c9d12e14d54910) feat(core): add dangerous option to disable compile time CSP injection ([#3775](https://www.github.com/tauri-apps/tauri/pull/3775)) on 2022-03-28

- Use the default value for `MacConfig.minimumSystemVersion` if the value is set to an empty string.
  - [c81534eb](https://www.github.com/tauri-apps/tauri/commit/c81534ebd873c358e0346c7949aeb171803149a5) feat(cli): use default macOS minimum system version when it is empty ([#3658](https://www.github.com/tauri-apps/tauri/pull/3658)) on 2022-03-13

- Replace multiple dependencies who's C code compiled concurrently and caused
  the other ones to bloat compile time significantly.

- `zstd` -> `brotli`

- `blake3` -> a vendored version of the blake3 reference

- `ring` -> `getrandom`

See https://github.com/tauri-apps/tauri/pull/3773 for more information about
these specific choices.

- [8661e3e2](https://www.github.com/tauri-apps/tauri/commit/8661e3e24d96c399bfbcdee5d8e9d6beba2265a7) replace dependencies with long build times when used together (closes [#3571](https://www.github.com/tauri-apps/tauri/pull/3571)) ([#3773](https://www.github.com/tauri-apps/tauri/pull/3773)) on 2022-03-27

## \[1.0.0-rc.3]

- Use `is_symlink` API compatible with Rust v1.57 instead of std/fs/struct.Metadata.html#method.is_symlink.
  - [73388119](https://www.github.com/tauri-apps/tauri/commit/73388119e653e7902b19beef2ab6d7c5f8a7b83a) use older symlink check function ([#3579](https://www.github.com/tauri-apps/tauri/pull/3579)) on 2022-03-01

## \[1.0.0-rc.2]

- Changed the default value for `tauri > bundle > macOS > minimumSystemVersion` to `10.13`.
  - [fce344b9](https://www.github.com/tauri-apps/tauri/commit/fce344b90b7227f8f5514853c2f885fb24d3648e) feat(core): set default value for `minimum_system_version` to 10.13 ([#3497](https://www.github.com/tauri-apps/tauri/pull/3497)) on 2022-02-17

## \[1.0.0-rc.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.0]

- The `allowlist` configuration now includes a `clipboard` object, controlling the exposure of the `writeText` and `readText` APIs.
  - [d660cab3](https://www.github.com/tauri-apps/tauri/commit/d660cab38d7d703e8b2bb85a3e9462d9e28b086b) feat: enhance allowlist configuration \[TRI-027] ([#11](https://www.github.com/tauri-apps/tauri/pull/11)) on 2022-01-09
- The dialog allowlist now includes flags for the `message`, `ask` and `confirm` APIs.
  - [d660cab3](https://www.github.com/tauri-apps/tauri/commit/d660cab38d7d703e8b2bb85a3e9462d9e28b086b) feat: enhance allowlist configuration \[TRI-027] ([#11](https://www.github.com/tauri-apps/tauri/pull/11)) on 2022-01-09
- The `allowlist` configuration now includes a `process` object, controlling the exposure of the `relaunch` and `exit` APIs.
  - [d660cab3](https://www.github.com/tauri-apps/tauri/commit/d660cab38d7d703e8b2bb85a3e9462d9e28b086b) feat: enhance allowlist configuration \[TRI-027] ([#11](https://www.github.com/tauri-apps/tauri/pull/11)) on 2022-01-09
- The `window` allowlist now includes options to enable all window modification APIs: `center`, `close`, `create`, `hide`, `maximize`, `minimize`, `print`, `requestUserAttention`, `setAlwaysOnTop`, `setDecorations`, `setFocus`, `setFullscreen`, `setIcon`, `setMaxSize`, `setMinSize`, `setPosition`, `setResizable`, `setSize`, `setSkipTaskbar`, `setTitle`, `show`, `startDragging`, `unmaximize` and `unminimize`.
  - [d660cab3](https://www.github.com/tauri-apps/tauri/commit/d660cab38d7d703e8b2bb85a3e9462d9e28b086b) feat: enhance allowlist configuration \[TRI-027] ([#11](https://www.github.com/tauri-apps/tauri/pull/11)) on 2022-01-09
- Added `asset` allowlist configuration, which enables the `asset` protocol and defines it access scope.
  - [7920ff14](https://www.github.com/tauri-apps/tauri/commit/7920ff14e6424079c48ea5645d9aa13e7a272b87) feat: scope the `fs` API and the `asset` protocol \[TRI-026] \[TRI-010] \[TRI-011] ([#10](https://www.github.com/tauri-apps/tauri/pull/10)) on 2022-01-09
- Change `CliArg` numeric types from `u64` to `usize`.
  - [1f988535](https://www.github.com/tauri-apps/tauri/commit/1f98853573a837dd0cfc2161b206a5033ec2da5e) chore(deps) Update Tauri Core ([#2480](https://www.github.com/tauri-apps/tauri/pull/2480)) on 2021-08-24
- Apply `nonce` to `script` and `style` tags and set them on the `CSP` (`script-src` and `style-src` fetch directives).
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- The path returned from `tauri::api::process::current_binary` is now cached when loading the binary.
  - [7c3db7a3](https://www.github.com/tauri-apps/tauri/commit/7c3db7a3811fd4de3e71c78cfd00894fa51ab786) cache current binary path much sooner ([#45](https://www.github.com/tauri-apps/tauri/pull/45)) on 2022-02-01
- Added `dev_csp` to the `security` configuration object.
  - [cf54dcf9](https://www.github.com/tauri-apps/tauri/commit/cf54dcf9c81730e42c9171daa9c8aa474c95b522) feat: improve `CSP` security with nonces and hashes, add `devCsp` \[TRI-004] ([#8](https://www.github.com/tauri-apps/tauri/pull/8)) on 2022-01-09
- Fixes resource directory resolution on Linux.
  - [1a28904b](https://www.github.com/tauri-apps/tauri/commit/1a28904b8ebea92e143d5dc21ebd209e9edec531) fix(core): resource path resolution on Linux, closes [#2493](https://www.github.com/tauri-apps/tauri/pull/2493) on 2021-08-22
- Allow using a fixed version for the Webview2 runtime via the `tauri > bundle > windows > webviewFixedRuntimePath` config option.
  - [85df94f2](https://www.github.com/tauri-apps/tauri/commit/85df94f2b0d40255812b42c5e32a70c4b45392df) feat(core): config for fixed webview2 runtime version path ([#27](https://www.github.com/tauri-apps/tauri/pull/27)) on 2021-11-02
- The updater `pubkey` is now a required field for security reasons. Sign your updates with the `tauri signer` command.
  - [d95cc831](https://www.github.com/tauri-apps/tauri/commit/d95cc83105dda52df7514e30e54f3676cdb374ee) feat: enforce updater public key \[TRI-015] ([#42](https://www.github.com/tauri-apps/tauri/pull/42)) on 2022-01-09
- Added the `isolation` pattern.
  - [d5d6d2ab](https://www.github.com/tauri-apps/tauri/commit/d5d6d2abc17cd89c3a079d2ce01581193469dbc0) Isolation Pattern ([#43](https://www.github.com/tauri-apps/tauri/pull/43)) Co-authored-by: Ngo Iok Ui (Wu Yu Wei) <wusyong9104@gmail.com> Co-authored-by: Lucas Fernandes Nogueira <lucas@tauri.app> on 2022-01-17
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
- **Breaking change**\* Remove default webview window when `tauri.conf.json > tauri > windows` is not set.
  - [c119060e](https://www.github.com/tauri-apps/tauri/commit/c119060e3d9a5a824639fb6b3c45a87e7a62e4e2) refactor(core): empty default value for config > tauri > windows ([#3380](https://www.github.com/tauri-apps/tauri/pull/3380)) on 2022-02-10
- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Adds `scope` glob array config under `tauri > allowlist > fs`.
  Adds `assetScope` glob array config under `tauri > allowlist > protocol`.
  Adds `scope` URL array config under `tauri > allowlist > http`.
  - [7920ff14](https://www.github.com/tauri-apps/tauri/commit/7920ff14e6424079c48ea5645d9aa13e7a272b87) feat: scope the `fs` API and the `asset` protocol \[TRI-026] \[TRI-010] \[TRI-011] ([#10](https://www.github.com/tauri-apps/tauri/pull/10)) on 2022-01-09
  - [0ad1c651](https://www.github.com/tauri-apps/tauri/commit/0ad1c6515f696fadefddbf133a9561836b3d5934) feat(core): add `http` allowlist scope \[TRI-008] ([#24](https://www.github.com/tauri-apps/tauri/pull/24)) on 2021-10-29
- The `shell` allowlist now includes a `sidecar` flag, which enables the use of the `shell` API to execute sidecars.
  - [eed01728](https://www.github.com/tauri-apps/tauri/commit/eed017287fed2ade689af4268e8b63b9c9f2e585) feat(core): add `shell > sidecar` allowlist and `process` feature flag \[TRI-037] ([#18](https://www.github.com/tauri-apps/tauri/pull/18)) on 2021-10-24
- Force updater endpoint URL to use `https` on release builds.
  - [c077f449](https://www.github.com/tauri-apps/tauri/commit/c077f449270cffbf7956b1af81e1fb237ebf564a) feat: force endpoint URL to use https on release \[TRI-015] ([#41](https://www.github.com/tauri-apps/tauri/pull/41)) on 2022-01-09

## \[1.0.0-beta.3]

- Fixes minimum window height being used as maximum height.
  - [e3f99165](https://www.github.com/tauri-apps/tauri/commit/e3f9916526b226866137cb663e5cafab2b6a0e01) fix(core) minHeight being used as maxHeight ([#2247](https://www.github.com/tauri-apps/tauri/pull/2247)) on 2021-07-19
- Implement `Debug` on public API structs and enums.
  - [fa9341ba](https://www.github.com/tauri-apps/tauri/commit/fa9341ba18ba227735341530900714dba0f27291) feat(core): implement `Debug` on public API structs/enums, closes [#2292](https://www.github.com/tauri-apps/tauri/pull/2292) ([#2387](https://www.github.com/tauri-apps/tauri/pull/2387)) on 2021-08-11
- Keep original value on `config > package > productName` on Linux (previously converted to kebab-case).
  - [3f039cb8](https://www.github.com/tauri-apps/tauri/commit/3f039cb8a308b0f18deaa37d7cfb1cc50d308d0e) fix: keep original `productName` for .desktop `Name` field, closes [#2295](https://www.github.com/tauri-apps/tauri/pull/2295) ([#2384](https://www.github.com/tauri-apps/tauri/pull/2384)) on 2021-08-10
- Inject the invoke key on regular `<script></script>` tags.
  - [d0142e87](https://www.github.com/tauri-apps/tauri/commit/d0142e87ddf5231fd46e2cbe4769bb16f3fe01e9) fix(core): invoke key injection on regular JS scripts, closes [#2342](https://www.github.com/tauri-apps/tauri/pull/2342) ([#2344](https://www.github.com/tauri-apps/tauri/pull/2344)) on 2021-08-03

## \[1.0.0-beta.2]

- Inject invoke key on `script` tags with `type="module"`.
  - [f03eea9c](https://www.github.com/tauri-apps/tauri/commit/f03eea9c9b964709532afbc4d1dd343b3fd96081) feat(core): inject invoke key on `<script type="module">` ([#2120](https://www.github.com/tauri-apps/tauri/pull/2120)) on 2021-06-29
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

## \[1.0.0-beta.1]

- Allow `dev_path` and `dist_dir` to be an array of root files and directories to embed.
  - [6ec54c53](https://www.github.com/tauri-apps/tauri/commit/6ec54c53b504eec3873d326b1a45e450227d46ed) feat(core): allow `dev_path`, `dist_dir` as array of paths, fixes [#1897](https://www.github.com/tauri-apps/tauri/pull/1897) ([#1926](https://www.github.com/tauri-apps/tauri/pull/1926)) on 2021-05-31
- Validate `tauri.conf.json > build > devPath` and `tauri.conf.json > build > distDir` values.
  - [e97846aa](https://www.github.com/tauri-apps/tauri/commit/e97846aae933cad5cba284a2a133ae7aaee1107c) feat(core): validate `devPath` and `distDir` values ([#1848](https://www.github.com/tauri-apps/tauri/pull/1848)) on 2021-05-17
- Adds `file_drop_enabled` flag on `WindowConfig`.
  - [9cd10df4](https://www.github.com/tauri-apps/tauri/commit/9cd10df4d520de12f3b13fe88cc1c1a1b4bd48bf) feat(core): allow disabling file drop handler, closes [#2014](https://www.github.com/tauri-apps/tauri/pull/2014) ([#2030](https://www.github.com/tauri-apps/tauri/pull/2030)) on 2021-06-21
- Hide `phf` crate export (not public API).
  - [cd1a299a](https://www.github.com/tauri-apps/tauri/commit/cd1a299a7d5a9bd164063a32c87a27762b71e9a8) chore(core): hide phf, closes [#1961](https://www.github.com/tauri-apps/tauri/pull/1961) ([#1964](https://www.github.com/tauri-apps/tauri/pull/1964)) on 2021-06-09

## \[1.0.0-beta.0]

- **Breaking:** The `assets` field on the `tauri::Context` struct is now a `Arc<impl Assets>`.
  - [5110c70](https://www.github.com/tauri-apps/tauri/commit/5110c704be67e51d49fb83f3710afb593973dcf9) feat(core): allow users to access the Assets instance ([#1691](https://www.github.com/tauri-apps/tauri/pull/1691)) on 2021-05-03
- Reintroduce `csp` injection, configured on `tauri.conf.json > tauri > security > csp`.
  - [6132f3f](https://www.github.com/tauri-apps/tauri/commit/6132f3f4feb64488ef618f690a4f06adce864d91) feat(core): reintroduce CSP injection ([#1704](https://www.github.com/tauri-apps/tauri/pull/1704)) on 2021-05-04
- Added the \`#\[non_exhaustive] attribute where appropriate.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05
- The `platform::resource_dir` API now takes the `PackageInfo`.
  - [7bb7dda](https://www.github.com/tauri-apps/tauri/commit/7bb7dda7523bc1a81e890e0aeafffd35e3ed767f) refactor(core): resolve resource_dir using the package info ([#1762](https://www.github.com/tauri-apps/tauri/pull/1762)) on 2021-05-10

## \[1.0.0-beta-rc.1]

- The package info APIs now checks the `package` object on `tauri.conf.json`.
  - [8fd1baf](https://www.github.com/tauri-apps/tauri/commit/8fd1baf69b14bb81d7be9d31605ed7f02058b392) fix(core): pull package info from tauri.conf.json if set ([#1581](https://www.github.com/tauri-apps/tauri/pull/1581)) on 2021-04-22
  - [f575aaa](https://www.github.com/tauri-apps/tauri/commit/f575aaad71f23d44b2f89cf9ee5d84817dc3bb7a) fix: change files not referencing core packages ([#1619](https://www.github.com/tauri-apps/tauri/pull/1619)) on 2021-04-25

## \[1.0.0-beta-rc.0]

- The Tauri files are now read on the app space instead of the `tauri` create.
  Also, the `AppBuilder` `build` function now returns a Result.
  - [e02c941](https://www.github.com/tauri-apps/tauri/commit/e02c9419cb8c66f4e43ed598d2fc74d4b19384ec) refactor(tauri): support for building without environmental variables ([#850](https://www.github.com/tauri-apps/tauri/pull/850)) on 2021-02-09
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
