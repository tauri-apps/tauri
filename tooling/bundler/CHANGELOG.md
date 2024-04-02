# Changelog

## \[2.1.0-beta.0]

### New Features

- [`259d84529`](https://www.github.com/tauri-apps/tauri/commit/259d845290dde40639537258b2810567910f47f3)([#9209](https://www.github.com/tauri-apps/tauri/pull/9209)) Add suport for include `preinstall`, `postinstall`, `preremove` and `postremove` scripts into Debian and RPM packages.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.11`

## \[2.0.1-beta.7]

### Bug Fixes

- [`a799f24f9`](https://www.github.com/tauri-apps/tauri/commit/a799f24f97e12c3f3fcdd1864fdd7c6559263fb7)([#9185](https://www.github.com/tauri-apps/tauri/pull/9185)) Fixed an issue that caused the msi bundler to crash when deep link schemes were configured.

## \[2.0.1-beta.6]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.10`

## \[2.0.1-beta.5]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.9`

## \[2.0.1-beta.4]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.8`

## \[2.0.1-beta.3]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.7`

## \[2.0.1-beta.2]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.6`

## \[2.0.1-beta.1]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.5`

## \[2.0.1-beta.0]

### Bug Fixes

- [`84c783f6`](https://www.github.com/tauri-apps/tauri/commit/84c783f6bc46827032666b0cba100ed37560240c)([#8948](https://www.github.com/tauri-apps/tauri/pull/8948)) Fix NSIS installer always containing a license page even though `licenseFile` option is not set in the config.
- [`84c783f6`](https://www.github.com/tauri-apps/tauri/commit/84c783f6bc46827032666b0cba100ed37560240c)([#8948](https://www.github.com/tauri-apps/tauri/pull/8948)) Don't fallback to `licenseFile` and use only `license` field when building RPM.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.4`

## \[2.0.0-beta.3]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.3`

## \[2.0.0-beta.2]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.2`

## \[2.0.0-beta.1]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.1`

## \[2.0.0-beta.0]

### Enhancements

- [`d6c7568c`](https://www.github.com/tauri-apps/tauri/commit/d6c7568c27445653edf570f3969163bc358ba2ba)([#8720](https://www.github.com/tauri-apps/tauri/pull/8720)) Add `files` option to the AppImage Configuration.
- [`30be0e30`](https://www.github.com/tauri-apps/tauri/commit/30be0e305773edbd07d3834d2cad979ac67a4d23)([#8303](https://www.github.com/tauri-apps/tauri/pull/8303)) Added Russian language support to the NSIS bundler.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-beta.0`

### Breaking Changes

- [`8de308d1`](https://www.github.com/tauri-apps/tauri/commit/8de308d1bf6a855d7a26af58bd0e744938ba47d8)([#8723](https://www.github.com/tauri-apps/tauri/pull/8723)) -   Removed all license fields from `WixSettings`, `NsisSettings` and `MacOsSettings` and replaced with `license` and `license_file` fields in `BundlerSettings`.

## \[2.0.0-alpha.14]

### New Features

- [`27bad32d`](https://www.github.com/tauri-apps/tauri/commit/27bad32d4d4acca8155b20225d529d540fb9aaf4)([#7798](https://www.github.com/tauri-apps/tauri/pull/7798)) Add `files` object on the `tauri > bundle > macOS` configuration option.
- [`27bad32d`](https://www.github.com/tauri-apps/tauri/commit/27bad32d4d4acca8155b20225d529d540fb9aaf4)([#7798](https://www.github.com/tauri-apps/tauri/pull/7798)) Add `files` map on the `MacOsSettings` struct to add custom files to the `.app` bundle.

### Enhancements

- [`091100ac`](https://www.github.com/tauri-apps/tauri/commit/091100acbb507b51de39fb1446f685926f888fd2)([#5202](https://www.github.com/tauri-apps/tauri/pull/5202)) Add RPM packaging
- [`8032b22f`](https://www.github.com/tauri-apps/tauri/commit/8032b22f2a5b866e4be8b9bd2165eedd09309e15)([#8596](https://www.github.com/tauri-apps/tauri/pull/8596)) Support using socks proxy from environment when downloading files.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.13`

## \[2.0.0-alpha.13]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.12`

## \[2.0.0-alpha.12]

### Bug Fixes

- [`34196e25`](https://www.github.com/tauri-apps/tauri/commit/34196e25c4ca2362bdfe1cf4598082aca71fe0a0)([#8182](https://www.github.com/tauri-apps/tauri/pull/8182)) Use original version string on WiX output file name.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.11`

## \[2.0.0-alpha.11]

### Enhancements

- [`c6c59cf2`](https://www.github.com/tauri-apps/tauri/commit/c6c59cf2373258b626b00a26f4de4331765dd487) Pull changes from Tauri 1.5 release.
- [`cfe6fa6c`](https://www.github.com/tauri-apps/tauri/commit/cfe6fa6c91a8cc177d4665ba04dad32ba545159d)([#8061](https://www.github.com/tauri-apps/tauri/pull/8061)) Added German language support to the NSIS bundler.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.10`

## \[2.0.0-alpha.10]

### New Features

- [`880266a7`](https://www.github.com/tauri-apps/tauri/commit/880266a7f697e1fe58d685de3bb6836ce5251e92)([#8031](https://www.github.com/tauri-apps/tauri/pull/8031)) Bump the MSRV to 1.70.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.9`

## \[2.0.0-alpha.9]

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.8`

## \[2.0.0-alpha.8]

### Enhancements

- [`04949d16`](https://www.github.com/tauri-apps/tauri/commit/04949d16586acddab97a3c083a61c81b18d6933e)([#7624](https://www.github.com/tauri-apps/tauri/pull/7624)) Added Bulgarian language support to the NSIS bundler.

## \[2.0.0-alpha.7]

### Bug Fixes

- [`3065c8ae`](https://www.github.com/tauri-apps/tauri/commit/3065c8aea375535763e1532951c4057a426fce80)([#7296](https://www.github.com/tauri-apps/tauri/pull/7296)) Enable `zip`'s `deflate` feature flag to fix issues when downloading nsis and wix tools.

### Dependencies

- Upgraded to `tauri-utils@2.0.0-alpha.7`

## \[2.0.0-alpha.6]

### Dependencies

- Updated to latest `tauri-utils`

## \[2.0.0-alpha.5]

- [`2d5378bf`](https://www.github.com/tauri-apps/tauri/commit/2d5378bfc1ba817ee2f331b41738a90e5997e5e8)([#6717](https://www.github.com/tauri-apps/tauri/pull/6717)) Removed the `UpdaterSettings::dialog` field.
- [`6a6b1388`](https://www.github.com/tauri-apps/tauri/commit/6a6b1388ea5787aea4c0e0b0701a4772087bbc0f)([#6853](https://www.github.com/tauri-apps/tauri/pull/6853)) Correctly escape XML for resource files in WiX bundler.
- [`3188f376`](https://www.github.com/tauri-apps/tauri/commit/3188f3764978c6d1452ee31d5a91469691e95094)([#6883](https://www.github.com/tauri-apps/tauri/pull/6883)) Bump the MSRV to 1.65.
- [`422b4817`](https://www.github.com/tauri-apps/tauri/commit/422b48179856504e980a156500afa8e22c44d357)([#6871](https://www.github.com/tauri-apps/tauri/pull/6871)) Added the following languages to the NSIS bundler:

  - `Spanish`
  - `SpanishInternational`
- [`2915bd06`](https://www.github.com/tauri-apps/tauri/commit/2915bd068ed40dc01a363b69212c6b6f2d3ec01e)([#6854](https://www.github.com/tauri-apps/tauri/pull/6854)) Correctly escape arguments in NSIS script to fix bundling apps that use non-default WebView2 install modes.

## \[2.0.0-alpha.4]

- Added `android` configuration object under `tauri > bundle`.
  - Bumped due to a bump in tauri-utils.
  - [db4c9dc6](https://www.github.com/tauri-apps/tauri/commit/db4c9dc655e07ee2184fe04571f500f7910890cd) feat(core): add option to configure Android's minimum SDK version ([#6651](https://www.github.com/tauri-apps/tauri/pull/6651)) on 2023-04-07

## \[2.0.0-alpha.3]

- Pull changes from Tauri 1.3 release.
  - [](https://www.github.com/tauri-apps/tauri/commit/undefined)  on undefined

## \[2.0.0-alpha.2]

- Added the `shadow` option to the window configuration and `set_shadow` option to the `window` allow list.
  - Bumped due to a bump in tauri-utils.
  - [a81750d7](https://www.github.com/tauri-apps/tauri/commit/a81750d779bc72f0fdb7de90b7fbddfd8049b328) feat(core): add shadow APIs ([#6206](https://www.github.com/tauri-apps/tauri/pull/6206)) on 2023-02-08

## \[2.0.0-alpha.1]

- Bump the MSRV to 1.64.
  - [7eb9aa75](https://www.github.com/tauri-apps/tauri/commit/7eb9aa75cfd6a3176d3f566fdda02d88aa529b0f) Update gtk to 0.16 ([#6155](https://www.github.com/tauri-apps/tauri/pull/6155)) on 2023-01-30

## \[2.0.0-alpha.0]

- First mobile alpha release!
  - [fa3a1098](https://www.github.com/tauri-apps/tauri/commit/fa3a10988a03aed1b66fb17d893b1a9adb90f7cd) feat(ci): prepare 2.0.0-alpha.0 ([#5786](https://www.github.com/tauri-apps/tauri/pull/5786)) on 2022-12-08

## \[1.5.0]

### New Features

- [`7aa30dec`](https://www.github.com/tauri-apps/tauri/commit/7aa30dec85a17c3d3faaf3841b93e10991b991b0)([#8620](https://www.github.com/tauri-apps/tauri/pull/8620)) Add `priority`, `section` and `changelog` options in Debian config.
- [`89911296`](https://www.github.com/tauri-apps/tauri/commit/89911296e475d5c36f3486b9b75232505846e767)([#8259](https://www.github.com/tauri-apps/tauri/pull/8259)) On macOS, support for signing nested .dylib, .app, .xpc and .framework under predefined directories inside the bundled frameworks ("MacOS", "Frameworks", "Plugins", "Helpers", "XPCServices" and "Libraries").
- [`8ce51cec`](https://www.github.com/tauri-apps/tauri/commit/8ce51cec3baf4ed88d80c59bf3bbe96fd369c7a0)([#7718](https://www.github.com/tauri-apps/tauri/pull/7718)) On Windows, NSIS installer now supports `/ARGS` flag to pass arguments to be used when launching the app after installation, only works if `/R` is used.

### Enhancements

- [`06890c70`](https://www.github.com/tauri-apps/tauri/commit/06890c70c643516b4e8037af87c8ee9103b977fa)([#8611](https://www.github.com/tauri-apps/tauri/pull/8611)) Support using socks proxy from environment when downloading files.

### Bug Fixes

- [`6bdba1f3`](https://www.github.com/tauri-apps/tauri/commit/6bdba1f330bedb5cdeda49eca1e295f281eb82eb)([#8585](https://www.github.com/tauri-apps/tauri/pull/8585)) Fix the `non-standard-file-perm` and `non-standard-dir-perm` issue in Debian packages

### Dependencies

- [`49266487`](https://www.github.com/tauri-apps/tauri/commit/4926648751ddbf764b8ffc46f3adc218afb2d472)([#8618](https://www.github.com/tauri-apps/tauri/pull/8618)) Replace `libflate` with `flate2` , this will help to provide additional functionalities and features.

## \[1.4.8]

### Enhancements

- [`b44e9c0f`](https://www.github.com/tauri-apps/tauri/commit/b44e9c0fcbb3f6994e38b8ef1ae18515db18ba7d)([#8431](https://www.github.com/tauri-apps/tauri/pull/8431)) Check if required files/tools for bundling are outdated or mis-hashed and redownload them.

### Dependencies

- Upgraded to `tauri-utils@1.5.2`

## \[1.4.7]

### Bug Fixes

- [`777ddf43`](https://www.github.com/tauri-apps/tauri/commit/777ddf434a966966dc8918322c1ec9ee3f822ee2)([#8376](https://www.github.com/tauri-apps/tauri/pull/8376)) Unset `NSISDIR` and `NSISCONFDIR` when running `makensis.exe` so it doesn't conflict with NSIS installed by the user.
- [`5ff9d459`](https://www.github.com/tauri-apps/tauri/commit/5ff9d4592a6dd8fc93165012ef367d78ea06e4ce)([#8390](https://www.github.com/tauri-apps/tauri/pull/8390)) NSIS perUser installers will now only check if the app is running on the current user.

## \[1.4.6]

### Bug Fixes

- [`1d5aa38a`](https://www.github.com/tauri-apps/tauri/commit/1d5aa38ae418ea31f593590b6d32cf04d3bfd8c1)([#8162](https://www.github.com/tauri-apps/tauri/pull/8162)) Fixes errors on command output, occuring when the output stream contains an invalid UTF-8 character, or ends with a multi-bytes UTF-8 character.
- [`977a39f4`](https://www.github.com/tauri-apps/tauri/commit/977a39f4f7fb5e47492b51df931643b1af4f92b0)([#8292](https://www.github.com/tauri-apps/tauri/pull/8292)) Migrate the WebView2 offline installer to use shorturl provided by Microsoft.
- [`f26d9f08`](https://www.github.com/tauri-apps/tauri/commit/f26d9f0884f63f61b9f4d4fac15e6b251163793e)([#8263](https://www.github.com/tauri-apps/tauri/pull/8263)) Fixes an issue in the NSIS installer which caused the uninstallation to leave empty folders on the system if the `resources` feature was used.
- [`92bc7d0e`](https://www.github.com/tauri-apps/tauri/commit/92bc7d0e16157434330a1bcf1eefda6f0f1e5f85)([#8233](https://www.github.com/tauri-apps/tauri/pull/8233)) Fixes an issue in the NSIS installer which caused the installation to take much longer than expected when many `resources` were added to the bundle.

## \[1.4.5]

### Enhancements

- [`cfe6fa6c`](https://www.github.com/tauri-apps/tauri/commit/cfe6fa6c91a8cc177d4665ba04dad32ba545159d)([#8061](https://www.github.com/tauri-apps/tauri/pull/8061)) Added German language support to the NSIS bundler.

## \[1.4.4]

### Enhancements

- [`3880b42d`](https://www.github.com/tauri-apps/tauri/commit/3880b42d18f218b89998bb5ead1aacfbed9d6202)([#7974](https://www.github.com/tauri-apps/tauri/pull/7974)) Include notarytool log output on error message in case notarization fails.

### Bug Fixes

- [`be8e5aa3`](https://www.github.com/tauri-apps/tauri/commit/be8e5aa3071d9bc5d0bd24647e8168f312d11c8d)([#8042](https://www.github.com/tauri-apps/tauri/pull/8042)) Fixes duplicated newlines on command outputs.

## \[1.4.3]

### Bug Fixes

- [`d0ae6750`](https://www.github.com/tauri-apps/tauri/commit/d0ae67503cdb2aeaadcea27af67285eea1cf3756)([#8012](https://www.github.com/tauri-apps/tauri/pull/8012)) Read `HTTP_PROXY` env var when downloading bundling resources on Windows.
- [`113bcd7b`](https://www.github.com/tauri-apps/tauri/commit/113bcd7b684a72eb0f421c663c6aa874197252bb)([#7980](https://www.github.com/tauri-apps/tauri/pull/7980)) In Debian packages, set `root` the owner of control files and package files.

## \[1.4.2]

### Bug Fixes

- [`f552c179`](https://www.github.com/tauri-apps/tauri/commit/f552c1796a61a5cfd51fad6d616bea3164b48a21)([#7998](https://www.github.com/tauri-apps/tauri/pull/7998)) Update the WebView2 offline installer GUIDs to resolve the 404 HTTP response status codes.

## \[1.4.1]

### Bug Fixes

- [`40d34002`](https://www.github.com/tauri-apps/tauri/commit/40d340021c0eab65aa1713807f7564e0698a321e)([#7972](https://www.github.com/tauri-apps/tauri/pull/7972)) The `APPLE_TEAM_ID` environment variable is now required for notarization authentication via Apple ID and app-specific password.
- [`cdd5516f`](https://www.github.com/tauri-apps/tauri/commit/cdd5516f339ad4345623a1e785c6e2c3a77477f8)([#7956](https://www.github.com/tauri-apps/tauri/pull/7956)) Fixes an app crash on app updates when the product name contained spaces.

## \[1.4.0]

### New Features

- [`4dd4893d`](https://www.github.com/tauri-apps/tauri/commit/4dd4893d7d166ac3a3b6dc2e3bd2540326352a78)([#5950](https://www.github.com/tauri-apps/tauri/pull/5950)) Allow using a resource map instead of a simple array in `BundleSettings::resources_map`.

### Enhancements

- [`764968ab`](https://www.github.com/tauri-apps/tauri/commit/764968ab383ec639e061986bc2411dd44e71b612)([#7398](https://www.github.com/tauri-apps/tauri/pull/7398)) Sign NSIS uninstaller as well.
- [`2f8881c0`](https://www.github.com/tauri-apps/tauri/commit/2f8881c010fa3493c092ddf3a343df08d7a79fc9)([#7775](https://www.github.com/tauri-apps/tauri/pull/7775)) Read the `APPLE_TEAM_ID` environment variable for macOS notarization arguments.
- [`cb1d4164`](https://www.github.com/tauri-apps/tauri/commit/cb1d4164e71e29f071b8438d02a7ec86a9fac67b)([#7487](https://www.github.com/tauri-apps/tauri/pull/7487)) On Windows, code sign the application binaries before trying to create the WiX and NSIS bundles to always sign the executables even if no bundle types are enabled.

  On Windows, code sign the sidecar binaries if they are not signed already.
- [`57f73f1b`](https://www.github.com/tauri-apps/tauri/commit/57f73f1b6a07e690d122d7534a0b3531c8c12c03)([#7486](https://www.github.com/tauri-apps/tauri/pull/7486)) On Windows, NSIS installer will write webview2 installer file to the well-known temp dir instead of the install dir, so we don't pollute the install dir.
- [`a7777ff4`](https://www.github.com/tauri-apps/tauri/commit/a7777ff485b725f177d08bbc00af607cd8ee8d6d)([#7626](https://www.github.com/tauri-apps/tauri/pull/7626)) Added Bulgarian language support to the NSIS bundler.
- [`e3bfb014`](https://www.github.com/tauri-apps/tauri/commit/e3bfb01411c3cc5e602c8f961f6cb5c9dd9524e1)([#7776](https://www.github.com/tauri-apps/tauri/pull/7776)) Add `compression` configuration option under `tauri > bundle > windows > nsis`.

### Bug Fixes

- [`46df2c9b`](https://www.github.com/tauri-apps/tauri/commit/46df2c9b917096388695f72ca4c56791fe652ef6)([#7360](https://www.github.com/tauri-apps/tauri/pull/7360)) Fix bundler skipping updater artifacts if `updater` target shows before other updater-enabled targets in the list, see [#7349](https://github.com/tauri-apps/tauri/issues/7349).
- [`2d35f937`](https://www.github.com/tauri-apps/tauri/commit/2d35f937de59272d26556310155c0b15d849953c)([#7481](https://www.github.com/tauri-apps/tauri/pull/7481)) Fix bundler skipping updater artifacts if only a macOS DMG bundle target is specified.
- [`dcdbe3eb`](https://www.github.com/tauri-apps/tauri/commit/dcdbe3eb6cc7d8a43caef98dfce71a11a4597644)([#7774](https://www.github.com/tauri-apps/tauri/pull/7774)) Remove extended attributes on the macOS app bundle using `xattr -cr $PATH`.
- [`dcdbe3eb`](https://www.github.com/tauri-apps/tauri/commit/dcdbe3eb6cc7d8a43caef98dfce71a11a4597644)([#7774](https://www.github.com/tauri-apps/tauri/pull/7774)) Code sign sidecars and frameworks on macOS.
- [`eba8e131`](https://www.github.com/tauri-apps/tauri/commit/eba8e1315ed7078eb9a9479f9e0072b061067341)([#7386](https://www.github.com/tauri-apps/tauri/pull/7386)) On Windows, fix installation packages not showing correct copyright information.
- [`32218a6f`](https://www.github.com/tauri-apps/tauri/commit/32218a6f8c1d90c2503e7cbc4523e4ab464ba032)([#7326](https://www.github.com/tauri-apps/tauri/pull/7326)) On Windows, fix NSIS installer identifying a previous NSIS-installed app as WiX-installed app and then fails to uninstall it.
- [`ca977f4b`](https://www.github.com/tauri-apps/tauri/commit/ca977f4b87c66808b4eac31a6d1925842b4c1570)([#7591](https://www.github.com/tauri-apps/tauri/pull/7591)) On Windows, Fix NSIS uninstaller deleting the wrong application data if the delete the application data checkbox is checked.
- [`0ae53f41`](https://www.github.com/tauri-apps/tauri/commit/0ae53f413948c7b955e595aa9c6c9e777caa8666)([#7361](https://www.github.com/tauri-apps/tauri/pull/7361)) On Windows, fix NSIS installer showing an error dialog even when the previous version was uninstalled sucessfully.
- [`09f7f57e`](https://www.github.com/tauri-apps/tauri/commit/09f7f57eeadbf94d8e9e14f3ab2b115a4c4aa473)([#7711](https://www.github.com/tauri-apps/tauri/pull/7711)) On Windows, fix NSIS installer trying to kill itself if the installer file name and the app `productName` are the same.
- [`6e36ebbf`](https://www.github.com/tauri-apps/tauri/commit/6e36ebbf84dee11a98d8df916c316c7d6f67b2a8)([#7342](https://www.github.com/tauri-apps/tauri/pull/7342)) On Windows, fix NSIS uninstaller failing to remove Start Menu shortcut if `perMachine` mode is used.

### Dependencies

- Upgraded to `tauri-utils@1.5.0`
- [`a2be88a2`](https://www.github.com/tauri-apps/tauri/commit/a2be88a21db76e9fa063c527031f3849f066eecd)([#7405](https://www.github.com/tauri-apps/tauri/pull/7405)) Removed the `bitness` dependency to speed up compile time.

### Breaking Changes

- [`964d81ff`](https://www.github.com/tauri-apps/tauri/commit/964d81ff01a076516d323546c169b2ba8156e55a)([#7616](https://www.github.com/tauri-apps/tauri/pull/7616)) The macOS notarization now uses `notarytool` as `altool` will be discontinued on November 2023. When authenticating with an API key, the key `.p8` file path must be provided in the `APPLE_API_KEY_PATH` environment variable. To prevent a breaking change, we will try to find the key path in the `altool` default search paths.

## \[1.3.0]

### New Features

- [`35cd751a`](https://www.github.com/tauri-apps/tauri/commit/35cd751adc6fef1f792696fa0cfb471b0bf99374)([#5176](https://www.github.com/tauri-apps/tauri/pull/5176)) Added `desktop_template` option on `DebianSettings`.
- [`29488205`](https://www.github.com/tauri-apps/tauri/commit/2948820579d20dfaa0861c2f0a58bd7737a7ffd1)([#6867](https://www.github.com/tauri-apps/tauri/pull/6867)) Allow specifying custom language files of Tauri's custom messages for the NSIS installer
- [`e092f799`](https://www.github.com/tauri-apps/tauri/commit/e092f799469ff32c7d1595d0f07d06fd2dab5c29)([#6887](https://www.github.com/tauri-apps/tauri/pull/6887)) Add `nsis > template` option to specify custom NSIS installer template.
- [`df89ccc1`](https://www.github.com/tauri-apps/tauri/commit/df89ccc1912db6b81d43d56c9e6d66980ece2e8d)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) For NSIS, Add support for `/P` to install or uninstall in passive mode, `/R` to (re)start the app and `/NS` to disable creating shortcuts in `silent` and `passive` modes.

### Enhancements

- [`3327dd64`](https://www.github.com/tauri-apps/tauri/commit/3327dd641d929ff7457a37cd7e557b45314b9755)([#7081](https://www.github.com/tauri-apps/tauri/pull/7081)) Remove macOS app bundles from the output if they are not requested by the user.
- [`fc7f9eba`](https://www.github.com/tauri-apps/tauri/commit/fc7f9ebada959e555f4cff617286f52ce463f29e)([#7001](https://www.github.com/tauri-apps/tauri/pull/7001)) Added Copyright field as BrandingText to the NSIS bundler.
- [`540ddd4e`](https://www.github.com/tauri-apps/tauri/commit/540ddd4e6a4926c26507a8d06fbe72a5895f8509)([#6906](https://www.github.com/tauri-apps/tauri/pull/6906)) Added Dutch language support to the NSIS bundler.
- [`b257bebf`](https://www.github.com/tauri-apps/tauri/commit/b257bebf9e478d616690ba1c357dc7f071ea7b31)([#6906](https://www.github.com/tauri-apps/tauri/pull/6906)) Added Japanese language support to the NSIS bundler.
- [`61e3ad89`](https://www.github.com/tauri-apps/tauri/commit/61e3ad89e93b060236d94459fb2d15938c7ce092)([#7010](https://www.github.com/tauri-apps/tauri/pull/7010)) Added Korean language support to the NSIS bundler.
- [`21d5eb84`](https://www.github.com/tauri-apps/tauri/commit/21d5eb84ab2372466a3254e656e2ec9b346a2db7)([#6965](https://www.github.com/tauri-apps/tauri/pull/6965)) Added Persian language support to the NSIS bundler.
- [`df89ccc1`](https://www.github.com/tauri-apps/tauri/commit/df89ccc1912db6b81d43d56c9e6d66980ece2e8d)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) NSIS `silent` and `passive` installer/updater will auto-kill the app if its running.
- [`43858a31`](https://www.github.com/tauri-apps/tauri/commit/43858a319725d696a298770982cc04e6dd0befb3)([#7038](https://www.github.com/tauri-apps/tauri/pull/7038)) Added Swedish language support to the NSIS bundler.
- [`ac183948`](https://www.github.com/tauri-apps/tauri/commit/ac183948d610a01ac602846cddd80563c86c45af)([#7018](https://www.github.com/tauri-apps/tauri/pull/7018)) Added Turkish language support to the NSIS bundler.
- [`60334f9e`](https://www.github.com/tauri-apps/tauri/commit/60334f9e02f6fdf1510e234aacc8c23382a05554)([#6859](https://www.github.com/tauri-apps/tauri/pull/6859)) NSIS installer will now check if a previous WiX `.msi` installation exist and will prompt users to uninstall it.
- [`db7c5fbf`](https://www.github.com/tauri-apps/tauri/commit/db7c5fbf2e86f3694720f65834eb2c258b7c1291)([#7143](https://www.github.com/tauri-apps/tauri/pull/7143)) Remove `attohttpc` in favor of `ureq`.

### Bug Fixes

- [`0302138f`](https://www.github.com/tauri-apps/tauri/commit/0302138f2fa8c869888f07006eda333a8dd2ba7f)([#6992](https://www.github.com/tauri-apps/tauri/pull/6992)) -   Updated the AppImage bundler to follow symlinks for `/usr/lib*`.
  - Fixes AppImage bundling for Void Linux, which was failing to bundle webkit2gtk because the `/usr/lib64` is a symlink to `/usr/lib`.
- [`1b8001b8`](https://www.github.com/tauri-apps/tauri/commit/1b8001b8b8757b6914e9b61a71ad1e092a5fa754)([#7056](https://www.github.com/tauri-apps/tauri/pull/7056)) Fix incorrect estimated app size for NSIS bundler when installed to a non-empty directory.
- [`df89ccc1`](https://www.github.com/tauri-apps/tauri/commit/df89ccc1912db6b81d43d56c9e6d66980ece2e8d)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) Fix NSIS installer disabling `do not uninstall` button and silent installer aborting, if `allowDowngrades` was disabled even when we are not downgrading.
- [`17da87d3`](https://www.github.com/tauri-apps/tauri/commit/17da87d3cd7922da18e8e2e41fcb8519f5c45f50)([#7036](https://www.github.com/tauri-apps/tauri/pull/7036)) Fix NSIS bundler failing to build when `productName` contained chinsese characters.
- [`4d4b72ba`](https://www.github.com/tauri-apps/tauri/commit/4d4b72ba38c5cea5cade501e383d946208e21deb)([#7086](https://www.github.com/tauri-apps/tauri/pull/7086)) Fix missing quote in Japanese NSIS language file.
- [`3cc295e9`](https://www.github.com/tauri-apps/tauri/commit/3cc295e99762d39a4a118db9b1f811f0503fe9e1)([#6928](https://www.github.com/tauri-apps/tauri/pull/6928)) Fix NSIS installer not using the old installation path as a default when using `perMachine` or `currentUser` install modes. Also fixes NSIS not respecting the `/D` flag which used to set the installation directory from command line.
- [`df89ccc1`](https://www.github.com/tauri-apps/tauri/commit/df89ccc1912db6b81d43d56c9e6d66980ece2e8d)([#6955](https://www.github.com/tauri-apps/tauri/pull/6955)) Fix NSIS silent installer not creating Desktop and StartMenu shortcuts. Pass `/NS` to disable creating them.

## \[1.2.1]

- Correctly escape XML for resource files in WiX bundler.
  - [6a6b1388](https://www.github.com/tauri-apps/tauri/commit/6a6b1388ea5787aea4c0e0b0701a4772087bbc0f) fix(bundler): correctly escape resource xml, fixes [#6853](https://www.github.com/tauri-apps/tauri/pull/6853) ([#6855](https://www.github.com/tauri-apps/tauri/pull/6855)) on 2023-05-04

- Added the following languages to the NSIS bundler:

- `Spanish`

- `SpanishInternational`

- [422b4817](https://www.github.com/tauri-apps/tauri/commit/422b48179856504e980a156500afa8e22c44d357) Add Spanish and SpanishInternational languages ([#6871](https://www.github.com/tauri-apps/tauri/pull/6871)) on 2023-05-06

- Correctly escape arguments in NSIS script to fix bundling apps that use non-default WebView2 install modes.
  - [2915bd06](https://www.github.com/tauri-apps/tauri/commit/2915bd068ed40dc01a363b69212c6b6f2d3ec01e) fix(bundler): Fix webview install modes in NSIS bundler ([#6854](https://www.github.com/tauri-apps/tauri/pull/6854)) on 2023-05-04

## \[1.2.0]

- Add dylib support to `tauri.bundle.macOS.frameworks`.
  - [ce76d95a](https://www.github.com/tauri-apps/tauri/commit/ce76d95ab186f4556166a2a22360067eab000fc8) feat(tauri-cli): add dylib support to `tauri.bundle.macOS.frameworks`, closes [#4615](https://www.github.com/tauri-apps/tauri/pull/4615) ([#5732](https://www.github.com/tauri-apps/tauri/pull/5732)) on 2022-12-31
- Added support for pre-release identifiers and build numbers for the `.msi` bundle target. Only one of each can be used and it must be numeric only. The version must still be semver compatible according to https://semver.org/.
  - [20ff1f45](https://www.github.com/tauri-apps/tauri/commit/20ff1f45968f1cfd30d093bb58255b348fbdfb98) feat(bundler): Add support for numeric-only build numbers in msi version ([#6096](https://www.github.com/tauri-apps/tauri/pull/6096)) on 2023-01-19
- On Windows, printing consistent paths on Windows with backslashs only.
  - [9da99607](https://www.github.com/tauri-apps/tauri/commit/9da996073ff07d4b59668a5315d40e9bc578e340) fix(cli): fix printing paths on Windows ([#6137](https://www.github.com/tauri-apps/tauri/pull/6137)) on 2023-01-26
- Fixed error during bundling process for the appimage target on subsequent bundling attempts.
  - [2f70d8da](https://www.github.com/tauri-apps/tauri/commit/2f70d8da2bc079400bb49e6793f755306049aab2) fix: symlink issue bundling for linux [#5781](https://www.github.com/tauri-apps/tauri/pull/5781) ([#6391](https://www.github.com/tauri-apps/tauri/pull/6391)) on 2023-03-17
- Fixes DMG bundling not finding bundle to set icon position.
  - [7489f966](https://www.github.com/tauri-apps/tauri/commit/7489f9669734a48be063907696b0f200a293ccb6) fix(bundler): fix problem of macOS bunder while i18n is set, closes [#6614](https://www.github.com/tauri-apps/tauri/pull/6614) ([#6615](https://www.github.com/tauri-apps/tauri/pull/6615)) on 2023-04-03
- Use escaping on Handlebars templates.
  - [6d6b6e65](https://www.github.com/tauri-apps/tauri/commit/6d6b6e653ea70fc02794f723092cdc860995c259) feat: configure escaping on handlebars templates ([#6678](https://www.github.com/tauri-apps/tauri/pull/6678)) on 2023-05-02
- Bump minimum supported Rust version to 1.60.
  - [5fdc616d](https://www.github.com/tauri-apps/tauri/commit/5fdc616df9bea633810dcb814ac615911d77222c) feat: Use the zbus-backed of notify-rust ([#6332](https://www.github.com/tauri-apps/tauri/pull/6332)) on 2023-03-31
- Add initial support for building `nsis` bundles on non-Windows platforms.
  - [60e6f6c3](https://www.github.com/tauri-apps/tauri/commit/60e6f6c3f1605f3064b5bb177992530ff788ccf0) feat(bundler): Add support for creating NSIS bundles on unix hosts ([#5788](https://www.github.com/tauri-apps/tauri/pull/5788)) on 2023-01-19
- Add `nsis` bundle target
  - [c94e1326](https://www.github.com/tauri-apps/tauri/commit/c94e1326a7c0767a13128a8b1d327a00156ece12) feat(bundler): add `nsis`, closes [#4450](https://www.github.com/tauri-apps/tauri/pull/4450), closes [#2319](https://www.github.com/tauri-apps/tauri/pull/2319) ([#4674](https://www.github.com/tauri-apps/tauri/pull/4674)) on 2023-01-03
- On Windows, the `msi` installer's `Launch App` checkbox will be checked by default.
  - [89602cdc](https://www.github.com/tauri-apps/tauri/commit/89602cdce34f3ea5b4a9b8921dc04a197f2d7de8) feat(bundler): check `Launch app` by default for WiX, closes [#5859](https://www.github.com/tauri-apps/tauri/pull/5859) ([#5871](https://www.github.com/tauri-apps/tauri/pull/5871)) on 2022-12-26

## \[1.1.2]

- Fixes blank taskbar icon on WiX updates.
  - [9093ef33](https://www.github.com/tauri-apps/tauri/commit/9093ef3314c27d2295b4266893fd2290c1bdfb6a) fix(bundler): blank taskbar icon on WiX update, closes [#5631](https://www.github.com/tauri-apps/tauri/pull/5631) ([#5779](https://www.github.com/tauri-apps/tauri/pull/5779)) on 2022-12-08

## \[1.1.1]

- Fix `allowlist > app > show/hide` always disabled when `allowlist > app > all: false`.
  - Bumped due to a bump in tauri-utils.
  - [bb251087](https://www.github.com/tauri-apps/tauri/commit/bb2510876d0bdff736d36bf3a465cdbe4ad2b90c) fix(core): extend allowlist with `app`'s allowlist, closes [#5650](https://www.github.com/tauri-apps/tauri/pull/5650) ([#5652](https://www.github.com/tauri-apps/tauri/pull/5652)) on 2022-11-18

## \[1.1.0]

- Use correct code `ja-JP` for japanese instead of `jp-JP`.
  - [d4cac202](https://www.github.com/tauri-apps/tauri/commit/d4cac202923fc34962721413f7051bca50002809) fix(bundler): fix japanese lang code, closes [#5342](https://www.github.com/tauri-apps/tauri/pull/5342) ([#5346](https://www.github.com/tauri-apps/tauri/pull/5346)) on 2022-10-04
- Fix WiX DLL load on Windows Server.
  - [7aaf27ce](https://www.github.com/tauri-apps/tauri/commit/7aaf27ce5f026547ed490731e37cfc917458bbd6) fix(bundler): load WiX DLLs on Github Actions ([#5552](https://www.github.com/tauri-apps/tauri/pull/5552)) on 2022-11-04
- - [7d9aa398](https://www.github.com/tauri-apps/tauri/commit/7d9aa3987efce2d697179ffc33646d086c68030c) feat: bump MSRV to 1.59 ([#5296](https://www.github.com/tauri-apps/tauri/pull/5296)) on 2022-09-28
- Add `tauri.conf.json > bundle > publisher` field to specify the app publisher.
  - [628285c1](https://www.github.com/tauri-apps/tauri/commit/628285c1cf43f03ed62378f3b6cc0c991317526f) feat(bundler): add `publisher` field, closes [#5273](https://www.github.com/tauri-apps/tauri/pull/5273) ([#5283](https://www.github.com/tauri-apps/tauri/pull/5283)) on 2022-09-28
- Clear environment variables on the WiX light.exe and candle.exe commands to avoid "Windows Installer Service could not be accessed" error. Variables prefixed with `TAURI` are propagated.
  - [7c0fa1f3](https://www.github.com/tauri-apps/tauri/commit/7c0fa1f3f93943b87a0042b5ba3bd6bb4099304a) fix(bundler): clear env before calling wix, closes [#4791](https://www.github.com/tauri-apps/tauri/pull/4791) ([#4819](https://www.github.com/tauri-apps/tauri/pull/4819)) on 2022-10-03

## \[1.0.7]

- Add missing allowlist config for `set_cursor_grab`, `set_cursor_visible`, `set_cursor_icon` and `set_cursor_position` APIs.
  - Bumped due to a bump in tauri-utils.
  - [c764408d](https://www.github.com/tauri-apps/tauri/commit/c764408da7fae123edd41115bda42fa75a4731d2) fix: Add missing allowlist config for cursor apis, closes [#5207](https://www.github.com/tauri-apps/tauri/pull/5207) ([#5211](https://www.github.com/tauri-apps/tauri/pull/5211)) on 2022-09-16

## \[1.0.6]

- Avoid re-downloading AppImage build tools on every build.
  - [02462052](https://www.github.com/tauri-apps/tauri/commit/024620529ed7c6cc601501db45abb7257f0b58f4) fix(bundler): cache appimage bundle tools ([#4790](https://www.github.com/tauri-apps/tauri/pull/4790)) on 2022-07-30
- Add `fips_compliant` configuration option for WiX.
  - [d88b9de7](https://www.github.com/tauri-apps/tauri/commit/d88b9de7aaeaaa2e42e4795dbc2b8642b5ae7a50) feat(core): add `fips_compliant` wix config option, closes [#4541](https://www.github.com/tauri-apps/tauri/pull/4541) ([#4843](https://www.github.com/tauri-apps/tauri/pull/4843)) on 2022-08-04

## \[1.0.5]

- Correctly fill the architecture when building Debian packages targeting ARM64 (aarch64).
  - [635f23b8](https://www.github.com/tauri-apps/tauri/commit/635f23b88adbb8726d628f67840709cd870836dc) fix(bundler): correctly set debian architecture for aarch64 ([#4700](https://www.github.com/tauri-apps/tauri/pull/4700)) on 2022-07-17

## \[1.0.4]

- Reduce the amount of allocations when converting cases.
  - [bc370e32](https://www.github.com/tauri-apps/tauri/commit/bc370e326810446e15b1f50fb962b980114ba16b) feat: reduce the amount of `heck`-related allocations ([#4634](https://www.github.com/tauri-apps/tauri/pull/4634)) on 2022-07-11
- Automatically load WiX extensions referenced in fragments.
  - [261d1bc9](https://www.github.com/tauri-apps/tauri/commit/261d1bc9d4a0ecf4dda16a47a1652efeabffc378) feat(bundler): load WiX extensions used on fragments, closes [#4546](https://www.github.com/tauri-apps/tauri/pull/4546) ([#4656](https://www.github.com/tauri-apps/tauri/pull/4656)) on 2022-07-12
- Fix AppImage builds by pinning the linuxdeploy version.
  - [89cb2526](https://www.github.com/tauri-apps/tauri/commit/89cb2526409d3b88e0aa15b93e4d26b09d9c0373) fix(bundler): pin linuxdeploy version on 2022-07-14
- Use `Bin_${sidecarFilename}` as the `Id` of sidecar file on WiX so you can reference it in your WiX fragments.
  - [597c9820](https://www.github.com/tauri-apps/tauri/commit/597c98203cad9b45949816659f5f59976328585a) feat(bundler): use known Id for the sidecar files on WiX, ref [#4546](https://www.github.com/tauri-apps/tauri/pull/4546) ([#4658](https://www.github.com/tauri-apps/tauri/pull/4658)) on 2022-07-12

## \[1.0.3]

- Build AppImages inside the `src-tauri/target` folder rather than `~/.cache/tauri`. Making it easier to clean and rebuild from scratch.
  - [8dd03e69](https://www.github.com/tauri-apps/tauri/commit/8dd03e69b0766eaef0b9f9fdcfe2ccbc9d0a10d1) fix(bundler): Build AppImages inside the target folder ([#4521](https://www.github.com/tauri-apps/tauri/pull/4521)) on 2022-07-03
- Ensure the notarization `RequestUUID` and `Status` parser works on macOS 10.13.6+.
  - [23d3d847](https://www.github.com/tauri-apps/tauri/commit/23d3d847d1b2d0c394d729a84fbeae2936111116) fix(bundler): ensure RequestUUID and Status parser adds a \n, closes [#4549](https://www.github.com/tauri-apps/tauri/pull/4549) ([#4559](https://www.github.com/tauri-apps/tauri/pull/4559)) on 2022-07-03
  - [f7c59ecf](https://www.github.com/tauri-apps/tauri/commit/f7c59ecfc85a90a0ff5c22da1a8b0e93d3663c86) fix(bundler): support macOS 10.13.6+ on notarization, closes [#4549](https://www.github.com/tauri-apps/tauri/pull/4549) ([#4593](https://www.github.com/tauri-apps/tauri/pull/4593)) on 2022-07-05

## \[1.0.2]

- Enhance the `DownloadedBootstrapper` Webview2 install mode compatibility with Windows 8.
  - [3df6c8c6](https://www.github.com/tauri-apps/tauri/commit/3df6c8c6454a052047b9f766691048860b50ea70) feat(bundler): enable TLS 1.2 before downloading webview2 bootstrapper ([#4543](https://www.github.com/tauri-apps/tauri/pull/4543)) on 2022-06-30

## \[1.0.1]

- Fix AppImage bundling when appimagelauncher is enabled.
  - [b0133083](https://www.github.com/tauri-apps/tauri/commit/b0133083dd4d22b0b7fdee02000ef8ecab26694b) Fix appimage creation in container when host has appimagelauncher enabled ([#4457](https://www.github.com/tauri-apps/tauri/pull/4457)) on 2022-06-27
- Fixes AppImage bundler crashes when the file path contains whitespace.
  - [82eb6e79](https://www.github.com/tauri-apps/tauri/commit/82eb6e79e8098bccd2b3d3581056b5350beb46c6) fix(bundler): Fix appimage bundler crashing if path has spaces ([#4471](https://www.github.com/tauri-apps/tauri/pull/4471)) on 2022-06-26
- Ensure `usr/lib` is a directory in the AppImage bundle.
  - [aa0336d6](https://www.github.com/tauri-apps/tauri/commit/aa0336d6c5764f1357d845f2bf3763a89a3771a1) fix(bundler): ensure AppImage usr/lib is a dir ([#4419](https://www.github.com/tauri-apps/tauri/pull/4419)) on 2022-06-21
- AppImage bundling will now prefer bundling correctly named appindicator library (including `.1` version suffix). With a symlink for compatibility with the old naming.
  - [bf45ca1d](https://www.github.com/tauri-apps/tauri/commit/bf45ca1df6691c05bdf72c5716cc01e89a7791d4) fix(cli,bundler): prefer AppImage libraries with ABI version ([#4505](https://www.github.com/tauri-apps/tauri/pull/4505)) on 2022-06-29
- Fix language code for korean (ko-KR).
  - [08a73acd](https://www.github.com/tauri-apps/tauri/commit/08a73acde877453ca5b45ea7548cdd3d407366a2) fix(bundler): fix language code. closes [#4437](https://www.github.com/tauri-apps/tauri/pull/4437) ([#4444](https://www.github.com/tauri-apps/tauri/pull/4444)) on 2022-06-24
- Use the plist crate instead of the `PlistBuddy` binary to merge user Info.plist file.
  - [45076b3e](https://www.github.com/tauri-apps/tauri/commit/45076b3ede4c5a3c14ffc0e4277c2c87639690cb) refactor(bundler): use the `plist` crate to create and merge Info.plist ([#4412](https://www.github.com/tauri-apps/tauri/pull/4412)) on 2022-06-21
- Validate app version before bundling WiX.
  - [672174b8](https://www.github.com/tauri-apps/tauri/commit/672174b822fcd2dff4a4aeeab370be3748e13843) feat(bundler): validate version before bundling with WiX ([#4429](https://www.github.com/tauri-apps/tauri/pull/4429)) on 2022-06-21
- Check if `$HOME\AppData\Local\tauri\WixTools` directory has all the required files and redownload WiX if something is missing.
  - [956af4f3](https://www.github.com/tauri-apps/tauri/commit/956af4f30f665a1d059aad15d070b4bab9ca49b3) feat(bundler): validate wix toolset files, ref [#4474](https://www.github.com/tauri-apps/tauri/pull/4474) ([#4475](https://www.github.com/tauri-apps/tauri/pull/4475)) on 2022-06-26
- Added webview install mode options.
  - [2ca762d2](https://www.github.com/tauri-apps/tauri/commit/2ca762d207030a892a6d128b519e150e2d733468) feat(bundler): extend webview2 installation options, closes [#2882](https://www.github.com/tauri-apps/tauri/pull/2882) [#2452](https://www.github.com/tauri-apps/tauri/pull/2452) ([#4466](https://www.github.com/tauri-apps/tauri/pull/4466)) on 2022-06-26

## \[1.0.0]

- Upgrade to `stable`!
  - [f4bb30cc](https://www.github.com/tauri-apps/tauri/commit/f4bb30cc73d6ba9b9ef19ef004dc5e8e6bb901d3) feat(covector): prepare for v1 ([#4351](https://www.github.com/tauri-apps/tauri/pull/4351)) on 2022-06-15

## \[1.0.0-rc.10]

- Bundle the tray library file (`.so` extension) in the AppImage if the `TRAY_LIBRARY_PATH` environment variable is set.
  - [34552444](https://www.github.com/tauri-apps/tauri/commit/3455244436578003a5fbb447b039e5c8971152ec) feat(cli): bundle appindicator library in the AppImage, closes [#3859](https://www.github.com/tauri-apps/tauri/pull/3859) ([#4267](https://www.github.com/tauri-apps/tauri/pull/4267)) on 2022-06-07
- Bundle additional gstreamer files needed for audio and video playback if the `APPIMAGE_BUNDLE_GSTREAMER` environment variable is set.
  - [d335fae9](https://www.github.com/tauri-apps/tauri/commit/d335fae92cdcbb0ee18aad4e54558914afa3e778) feat(bundler): bundle additional gstreamer files, closes [#4092](https://www.github.com/tauri-apps/tauri/pull/4092) ([#4271](https://www.github.com/tauri-apps/tauri/pull/4271)) on 2022-06-10
- Cache bundling tools in a directory shared by all tauri projects. Usually in `$XDG_HOME/.cache/tauri` on Linux and `$HOME\AppData\Local\tauri` on Windows.
  - [f48b1b0b](https://www.github.com/tauri-apps/tauri/commit/f48b1b0b3bec6a2a85c7dac67a128fa578f7ee15) feat(bundler): cache bundling tools in a common dir for all projects ([#4305](https://www.github.com/tauri-apps/tauri/pull/4305)) on 2022-06-09
- Pull correct linuxdeploy AppImage when building for 32-bit targets.
  - [53ae13d9](https://www.github.com/tauri-apps/tauri/commit/53ae13d99a06b786fb4fbeb1a10e934f7be3008c) fix(bundler): Pull correct 32bit linuxdeploy appimage, closes [#4260](https://www.github.com/tauri-apps/tauri/pull/4260) ([#4269](https://www.github.com/tauri-apps/tauri/pull/4269)) on 2022-06-04
- Copy the `/usr/bin/xdg-open` binary if it exists and the `APPIMAGE_BUNDLE_XDG_OPEN` environment variable is set.
  - [2322ac11](https://www.github.com/tauri-apps/tauri/commit/2322ac11cf6290c6bf65413048a049c8072f863b) fix(bundler): bundle `/usr/bin/xdg-open` in appimage if open API enabled ([#4265](https://www.github.com/tauri-apps/tauri/pull/4265)) on 2022-06-04
- On Linux, high-dpi icons are now placed in the correct directory
  and should be recognized by the desktop environment.
  - [c2b7c775](https://www.github.com/tauri-apps/tauri/commit/c2b7c7751764d0963362a4b271cd921d6424e5e9) fix: put linux high dpi icons in the correct dir ([#4281](https://www.github.com/tauri-apps/tauri/pull/4281)) on 2022-06-10
- Only png files from tauri.conf.json > tauri.bundle.icon are used for app icons for linux targets.
  Previously, all sizes from all source files (10 files using tauricon defaults) were used.
  This also prevents unexpectedly mixed icons in cases where platform-specific icons are used.
  - [a6f45d52](https://www.github.com/tauri-apps/tauri/commit/a6f45d5248ec0f4d3876afd8f106998306ea931e) Debian icon no fallback, fixes [#4280](https://www.github.com/tauri-apps/tauri/pull/4280) ([#4282](https://www.github.com/tauri-apps/tauri/pull/4282)) on 2022-06-09
- Log command output in real time instead of waiting for it to finish.
  - [76d1eaae](https://www.github.com/tauri-apps/tauri/commit/76d1eaaebda5c8f6b0d41bf6587945e98cd441f3) feat(cli): debug command output in real time ([#4318](https://www.github.com/tauri-apps/tauri/pull/4318)) on 2022-06-12

## \[1.0.0-rc.9]

- Statically link the Visual C++ runtime instead of using a merge module on the installer.
  - [bb061509](https://www.github.com/tauri-apps/tauri/commit/bb061509fb674bef86ecbc1de3aa8f3e367a9907) refactor(core): statically link vcruntime, closes [#4122](https://www.github.com/tauri-apps/tauri/pull/4122) ([#4227](https://www.github.com/tauri-apps/tauri/pull/4227)) on 2022-05-27

## \[1.0.0-rc.8]

- Use binary arch instead of `x86_64` on the AppImage bundle script.
  - [6830a739](https://www.github.com/tauri-apps/tauri/commit/6830a739535e920f1857a1946fb69750a6d7b92f) fix(bundler): use binary arch on appimage bundle script ([#4194](https://www.github.com/tauri-apps/tauri/pull/4194)) on 2022-05-23
- Fixes lost files on upgrade due to wrong implementation to keep shortcuts.
  - [8539e02f](https://www.github.com/tauri-apps/tauri/commit/8539e02f7fd56cc47b25ed45c8403d66abe262ac) fix(bundler): wix upgrade do not installing new files, closes [#4182](https://www.github.com/tauri-apps/tauri/pull/4182) on 2022-05-21

## \[1.0.0-rc.7]

- Change `tsp` value from `Option<bool>` to `bool`.
  - [29d8e768](https://www.github.com/tauri-apps/tauri/commit/29d8e768aa0b52e1997c0f5c9e447b80eff47b93) feat(config): adjust schema for documentation website, closes [#4139](https://www.github.com/tauri-apps/tauri/pull/4139) ([#4142](https://www.github.com/tauri-apps/tauri/pull/4142)) on 2022-05-17
- Fixes processing of resources with glob patterns when there are nested directories on Windows.
  - [3e702cf8](https://www.github.com/tauri-apps/tauri/commit/3e702cf8b15762cdca43c8d7ff6f6e8ee9670244) fix(bundler): ignore duplicated files in resource iter, closes [#4126](https://www.github.com/tauri-apps/tauri/pull/4126) ([#4129](https://www.github.com/tauri-apps/tauri/pull/4129)) on 2022-05-15
- Fixes resource bundling on Windows when the resource path includes root or parent directory components.
  - [787ea09a](https://www.github.com/tauri-apps/tauri/commit/787ea09adc40644b89926e2b629261065141d16c) fix: generate windows resource directories using resource_relpath, closes [#4087](https://www.github.com/tauri-apps/tauri/pull/4087). ([#4111](https://www.github.com/tauri-apps/tauri/pull/4111)) on 2022-05-13
- Set the application name when signing the Windows MSI.
  - [8e1daad1](https://www.github.com/tauri-apps/tauri/commit/8e1daad1537e93c6a969c03328b8502b92bf5b89) fix(bundler): set app name when signing MSI, closes [#3945](https://www.github.com/tauri-apps/tauri/pull/3945) ([#3950](https://www.github.com/tauri-apps/tauri/pull/3950)) on 2022-05-17
- Change WiX MajorUpgrade element's `Schedule` to `afterInstallExecute` to prevent removal of existing configuration such as the executable's pin to taskbar.
  - [d965b921](https://www.github.com/tauri-apps/tauri/commit/d965b92174a1e6a01fc1a080254402d52145af1e) fix(bundler): prevent removal of `pin to taskbar` on Windows ([#4144](https://www.github.com/tauri-apps/tauri/pull/4144)) on 2022-05-17
- Change the MSI reinstall mode so it only reinstall missing or different version files.
  - [1948ae53](https://www.github.com/tauri-apps/tauri/commit/1948ae53fdcd0ef99ef302066792d779a62c5065) fix(bundler): only reinstall missing or != version files, closes [#4122](https://www.github.com/tauri-apps/tauri/pull/4122) ([#4125](https://www.github.com/tauri-apps/tauri/pull/4125)) on 2022-05-15
- Allow configuring the display options for the MSI execution allowing quieter updates.
  - [9f2c3413](https://www.github.com/tauri-apps/tauri/commit/9f2c34131952ea83c3f8e383bc3cec7e1450429f) feat(core): configure msiexec display options, closes [#3951](https://www.github.com/tauri-apps/tauri/pull/3951) ([#4061](https://www.github.com/tauri-apps/tauri/pull/4061)) on 2022-05-15

## \[1.0.0-rc.6]

- Remove `Settings::verbose` option. You may now bring your own `log` frontend to receive logging output from the bundler while remaining in control of verbosity and formatting.
  - [35f21471](https://www.github.com/tauri-apps/tauri/commit/35f2147161e6697cbd2824681eeaf870b5a991c2) feat(cli): Improve CLI logging ([#4060](https://www.github.com/tauri-apps/tauri/pull/4060)) on 2022-05-07
- Ignore errors when loading `icns` files in the `.deb` package generation.
  - [de444b15](https://www.github.com/tauri-apps/tauri/commit/de444b15d222a65861b099a7536318bad000110e) fix(bundler): debian failing to load icns icon, closes [#3062](https://www.github.com/tauri-apps/tauri/pull/3062) ([#4009](https://www.github.com/tauri-apps/tauri/pull/4009)) on 2022-04-30
- Fix app downgrades when using the Windows installer.
  - [72e577dc](https://www.github.com/tauri-apps/tauri/commit/72e577dcc6a6733182059ab51b28a03c6077edc1) fix(bundler): properly reinstall files on MSI downgrades, closes [#3868](https://www.github.com/tauri-apps/tauri/pull/3868) ([#4044](https://www.github.com/tauri-apps/tauri/pull/4044)) on 2022-05-04

## \[1.0.0-rc.5]

- Set the Debian control file `Priority` field to `optional`.
  - [3bd3d923](https://www.github.com/tauri-apps/tauri/commit/3bd3d923d32144b4e28f9f597edf74ee422a9b54) fix: add priority field in debian/control ([#3865](https://www.github.com/tauri-apps/tauri/pull/3865)) on 2022-04-20
- Fixes DLL resource usage on Windows.
  - [f66bc3c2](https://www.github.com/tauri-apps/tauri/commit/f66bc3c2b8360b8b685dcadeb373852abe43d9e5) fix(bundler): DLL resources, closes [#3948](https://www.github.com/tauri-apps/tauri/pull/3948) ([#3949](https://www.github.com/tauri-apps/tauri/pull/3949)) on 2022-04-23
- **Breaking change:** Removed the `useBootstrapper` option. Use https://github.com/tauri-apps/fix-path-env-rs instead.
  - [6a5ff08c](https://www.github.com/tauri-apps/tauri/commit/6a5ff08ce9052b656aa40accedfd4315825164a3) refactor: remove bootstrapper, closes [#3786](https://www.github.com/tauri-apps/tauri/pull/3786) ([#3832](https://www.github.com/tauri-apps/tauri/pull/3832)) on 2022-03-31

## \[1.0.0-rc.4]

- Fixes DMG bundling on macOS 12.3.
  - [348a1ab5](https://www.github.com/tauri-apps/tauri/commit/348a1ab59d2697478a594016016f1fccbf1ac054) fix(bundler): DMG bundling on macOS 12.3 cannot use bless, closes [#3719](https://www.github.com/tauri-apps/tauri/pull/3719) ([#3721](https://www.github.com/tauri-apps/tauri/pull/3721)) on 2022-03-18

## \[1.0.0-rc.3]

- Added `tsp` config option under `tauri > bundle > windows`, which enables Time-Stamp Protocol (RFC 3161) for the timestamping
  server under code signing on Windows if set to `true`.
  - [bdd5f7c2](https://www.github.com/tauri-apps/tauri/commit/bdd5f7c2f03af4af8b60a9527e55bb18525d989b) fix: add support for Time-Stamping Protocol for Windows codesigning (fix [#3563](https://www.github.com/tauri-apps/tauri/pull/3563)) ([#3570](https://www.github.com/tauri-apps/tauri/pull/3570)) on 2022-03-07
- Properly create the updater bundle for all generated Microsoft Installer files.
  - [6a6f1e7b](https://www.github.com/tauri-apps/tauri/commit/6a6f1e7bf922bc6fa56db2e8e40affbb0849731d) fix(bundler): build updater bundle for all .msi files ([#3520](https://www.github.com/tauri-apps/tauri/pull/3520)) on 2022-02-24
- Fixes the Microsoft Installer launch path.
  - [8d699283](https://www.github.com/tauri-apps/tauri/commit/8d699283a4741c83b476fb079dc0333c7bf4f919) fix(bundler): Auto-launch app from install location, closes [#3547](https://www.github.com/tauri-apps/tauri/pull/3547) ([#3553](https://www.github.com/tauri-apps/tauri/pull/3553)) on 2022-02-24

## \[1.0.0-rc.2]

- Fixes sidecar bundling on Windows.
  - [2ecbed8d](https://www.github.com/tauri-apps/tauri/commit/2ecbed8d59d9e1788170aa87b90ed9556a473425) fix(bundler): sidecar on Windows, closes [#3446](https://www.github.com/tauri-apps/tauri/pull/3446) ([#3482](https://www.github.com/tauri-apps/tauri/pull/3482)) on 2022-02-16

## \[1.0.0-rc.1]

- Change default value for the `freezePrototype` configuration to `false`.
  - Bumped due to a bump in tauri-utils.
  - [3a4c0160](https://www.github.com/tauri-apps/tauri/commit/3a4c01606184be762adee055ddac803de0d28527) fix(core): change default `freezePrototype` to false, closes [#3416](https://www.github.com/tauri-apps/tauri/pull/3416) [#3406](https://www.github.com/tauri-apps/tauri/pull/3406) ([#3423](https://www.github.com/tauri-apps/tauri/pull/3423)) on 2022-02-12

## \[1.0.0-rc.0]

- Provide a provider short name for macOS app notarization if your Apple developer account is connected to more than one team.
  - [8ab8d529](https://www.github.com/tauri-apps/tauri/commit/8ab8d529426b1ed7926daced16a45b077033bfe2) Fix [#3288](https://www.github.com/tauri-apps/tauri/pull/3288): Add provider_short_name for macOS ([#3289](https://www.github.com/tauri-apps/tauri/pull/3289)) on 2022-01-27

- Allow building AppImages on systems without FUSE setup.
  - [28dd9adb](https://www.github.com/tauri-apps/tauri/commit/28dd9adb266b97db0bf7179268269f8614ce78e8) feat(bundler): support building AppImage without FUSE ([#3259](https://www.github.com/tauri-apps/tauri/pull/3259)) on 2022-01-21

- Fixes AppImage crashes caused by missing WebKit runtime files.
  - [bec7833a](https://www.github.com/tauri-apps/tauri/commit/bec7833a6c29ed9d1a5ab53e1c2cdd80e66cd272) fix(bundler): bundle additional webkit files. patch absolute paths in libwebkit\*.so files. fixes [#2805](https://www.github.com/tauri-apps/tauri/pull/2805),[#2689](https://www.github.com/tauri-apps/tauri/pull/2689) ([#2940](https://www.github.com/tauri-apps/tauri/pull/2940)) on 2021-12-09

- Initialize the preselected installation path with the location of the previous installation.
  - [ac1dfd8c](https://www.github.com/tauri-apps/tauri/commit/ac1dfd8c3039d87bef1fa2d815876903d5cc07ae) feat(bundler): initialize msi install path with previous location ([#3158](https://www.github.com/tauri-apps/tauri/pull/3158)) on 2022-01-07

- Replaces usage of the nightly command `RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json` with the stable command `rustc --print cfg`, improving target triple detection.
  - [839daec7](https://www.github.com/tauri-apps/tauri/commit/839daec7ab79c3ff2f552dd7579069bc64e83625) fix(bundler): Use `arch` instead of `llvm_target`. fix [#3285](https://www.github.com/tauri-apps/tauri/pull/3285) ([#3286](https://www.github.com/tauri-apps/tauri/pull/3286)) on 2022-02-05

- Fixes a deadlock on the `ResourcePaths` iterator.
  - [4c1be451](https://www.github.com/tauri-apps/tauri/commit/4c1be451066612862363bc481910bd6c3da1e185) fix(bundler): deadlock on `ResourcePaths` iterator, closes [#3146](https://www.github.com/tauri-apps/tauri/pull/3146) ([#3152](https://www.github.com/tauri-apps/tauri/pull/3152)) on 2022-01-02

- Move the copying of resources and sidecars from `cli.rs` to `tauri-build` so using the Cargo CLI directly processes the files for the application execution in development.
  - [5eb72c24](https://www.github.com/tauri-apps/tauri/commit/5eb72c24deddf5a01093bea96b90c0d8806afc3f) refactor: copy resources and sidecars on the Cargo build script ([#3357](https://www.github.com/tauri-apps/tauri/pull/3357)) on 2022-02-08

- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22

- **Breaking change:** The sidecar's target triple suffix is now removed at build time.
  - [3035e458](https://www.github.com/tauri-apps/tauri/commit/3035e4581c161ec7f0bd6d9b42e9015cf1dd1d77) Remove target triple from sidecar bin paths, closes [#3355](https://www.github.com/tauri-apps/tauri/pull/3355) ([#3356](https://www.github.com/tauri-apps/tauri/pull/3356)) on 2022-02-07

- When building Universal macOS Binaries through the virtual target `universal-apple-darwin`:

- Expect a universal binary to be created by the user

- Ensure that binary is bundled and accessed correctly at runtime

- [3035e458](https://www.github.com/tauri-apps/tauri/commit/3035e4581c161ec7f0bd6d9b42e9015cf1dd1d77) Remove target triple from sidecar bin paths, closes [#3355](https://www.github.com/tauri-apps/tauri/pull/3355) ([#3356](https://www.github.com/tauri-apps/tauri/pull/3356)) on 2022-02-07

- Allow setting the localization file for WiX.
  - [af329f27](https://www.github.com/tauri-apps/tauri/commit/af329f2722d6194c6d70e976fc970dc2c9e4de2b) feat(bundler): wix localization, closes [#3174](https://www.github.com/tauri-apps/tauri/pull/3174) ([#3179](https://www.github.com/tauri-apps/tauri/pull/3179)) on 2022-02-05

- Fix registry keys on the WiX template.
  - [2be1abd1](https://www.github.com/tauri-apps/tauri/commit/2be1abd112cc3d927c235b6d00a508e6d35be49e) fix(bundler) wix template escape character ([#2608](https://www.github.com/tauri-apps/tauri/pull/2608)) on 2021-09-21

- Configure WiX to add an option to launch the application after finishing setup.
  - [feb3a8f8](https://www.github.com/tauri-apps/tauri/commit/feb3a8f896802ff274333012c3b399beb5c86f41) feat(bundler): configure WiX to add launch option, closes [#3015](https://www.github.com/tauri-apps/tauri/pull/3015) ([#3043](https://www.github.com/tauri-apps/tauri/pull/3043)) on 2021-12-09

- Sign WiX installer in addition to the executable file.
  - [d801cc89](https://www.github.com/tauri-apps/tauri/commit/d801cc89b8bfa9beba347ebcd48cfccf890ff5bb) wix installer is also signed ([#3266](https://www.github.com/tauri-apps/tauri/pull/3266)) on 2022-01-23

## \[1.0.0-beta.4]

- Merge Tauri-generated Info.plist with the contents of `src-tauri/Info.plist` if it exists.
  - [537ab1b6](https://www.github.com/tauri-apps/tauri/commit/537ab1b6d5a792c550a535619965c9e4126292e6) feat(core): inject src-tauri/Info.plist file on dev and merge on bundle, closes [#1570](https://www.github.com/tauri-apps/tauri/pull/1570) [#2338](https://www.github.com/tauri-apps/tauri/pull/2338) ([#2444](https://www.github.com/tauri-apps/tauri/pull/2444)) on 2021-08-15
- Change the WiX config to allow upgrading installation with same version instead of duplicating the application.
  - [dd5e1ede](https://www.github.com/tauri-apps/tauri/commit/dd5e1ede32ab8c10849fe6583d93ef493dd6f184) fix(bundler): `AllowSameVersionUpgrades` on WiX, closes [#2211](https://www.github.com/tauri-apps/tauri/pull/2211) ([#2428](https://www.github.com/tauri-apps/tauri/pull/2428)) on 2021-08-14
- Check target architecture at runtime running `$ RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json` and parsing the `llvm-target` field, fixing macOS M1 sidecar check until we can compile the CLI with M1 target on GitHub Actions.
  - [5ebf389f](https://www.github.com/tauri-apps/tauri/commit/5ebf389f6c6805ccd2b15d81fe12416770e39222) feat(bundler): check target arch at runtime, closes [#2286](https://www.github.com/tauri-apps/tauri/pull/2286) ([#2422](https://www.github.com/tauri-apps/tauri/pull/2422)) on 2021-08-13
- Added `banner_path` field to the `WixSettings` struct.
  - [13003ec7](https://www.github.com/tauri-apps/tauri/commit/13003ec761b1530705d6129519dc4e226eb992c7) feat(bundler): add config for WiX banner path, closes [#2175](https://www.github.com/tauri-apps/tauri/pull/2175) ([#2448](https://www.github.com/tauri-apps/tauri/pull/2448)) on 2021-08-16
- Added `dialog_image_path` field to the `WixSettings` struct.
  - [9bfdeb42](https://www.github.com/tauri-apps/tauri/commit/9bfdeb42effeeec27aa15bbc5b05040eadfda5ba) feat(bundler): add config for WiX dialog image path ([#2449](https://www.github.com/tauri-apps/tauri/pull/2449)) on 2021-08-16
- Only convert package name and binary name to kebab-case, keeping the `.desktop` `Name` field with the original configured value.
  - [3f039cb8](https://www.github.com/tauri-apps/tauri/commit/3f039cb8a308b0f18deaa37d7cfb1cc50d308d0e) fix: keep original `productName` for .desktop `Name` field, closes [#2295](https://www.github.com/tauri-apps/tauri/pull/2295) ([#2384](https://www.github.com/tauri-apps/tauri/pull/2384)) on 2021-08-10
- Use `linuxdeploy` instead of `appimagetool` for `AppImage` bundling.
  - [397710b2](https://www.github.com/tauri-apps/tauri/commit/397710b2c5699dab78118f58760dda07e276d4c2) refactor(bundler): use linuxdeploy instead of appimagetool, closes [#1986](https://www.github.com/tauri-apps/tauri/pull/1986) ([#2437](https://www.github.com/tauri-apps/tauri/pull/2437)) on 2021-08-15

## \[1.0.0-beta.3]

- Fix WIX uninstaller by using unique `GUID` shortcut.
  - [caa8fcc9](https://www.github.com/tauri-apps/tauri/commit/caa8fcc93e5b56dacf042b9e7c6e7c56a1609310) fix(windows): use random `Guid` for uninstaller (wix) ([#2208](https://www.github.com/tauri-apps/tauri/pull/2208)) on 2021-07-14
- Run powershell commands with `-NoProfile` flag
  - [3e6f3416](https://www.github.com/tauri-apps/tauri/commit/3e6f34160deab4f774d90aba28122e5b6b6f9db2) fix(cli.rs): run powershell kill command without profile ([#2130](https://www.github.com/tauri-apps/tauri/pull/2130)) on 2021-06-30

## \[1.0.0-beta.2]

- Properly detect target platform's architecture.
  - [628a53eb](https://www.github.com/tauri-apps/tauri/commit/628a53eb6176f811d22d7730f08a99e5c370dbf4) fix(cli): properly detect target architecture, closes [#2040](https://www.github.com/tauri-apps/tauri/pull/2040) ([#2102](https://www.github.com/tauri-apps/tauri/pull/2102)) on 2021-06-28
- Properly bundle resources with nested folder structure.
  - [a6157212](https://www.github.com/tauri-apps/tauri/commit/a61572127df839ed23e34e9b49b2bada5f18f7fb) fix(bundler): resources bundling on Windows with nested folder structure ([#2081](https://www.github.com/tauri-apps/tauri/pull/2081)) on 2021-06-25

## \[1.0.0-beta.1]

- The process of copying binaries and resources to `project_out_directory` was moved to the Tauri CLI.
  - [8f29a260](https://www.github.com/tauri-apps/tauri/commit/8f29a260e67aa111f6aeb262bd846a46d2858ce9) fix(cli.rs): copy resources and binaries on dev, closes [#1298](https://www.github.com/tauri-apps/tauri/pull/1298) ([#1946](https://www.github.com/tauri-apps/tauri/pull/1946)) on 2021-06-04
- Allow setting a path to a license file for the Windows Installer (`tauri.conf.json > bundle > windows > wix > license`).
  - [b769c7f7](https://www.github.com/tauri-apps/tauri/commit/b769c7f7da4064b6133bf39a82127863d0d35531) feat(bundler): windows installer license, closes [#2009](https://www.github.com/tauri-apps/tauri/pull/2009) ([#2027](https://www.github.com/tauri-apps/tauri/pull/2027)) on 2021-06-21
- Configure app shortcut on the Windows Installer.
  - [f0603fcc](https://www.github.com/tauri-apps/tauri/commit/f0603fccb389620e105a5927a9e4b84b5e6853f4) feat(bundler): desktop shortcut on Windows ([#2052](https://www.github.com/tauri-apps/tauri/pull/2052)) on 2021-06-23
- Allow setting the Windows installer language and using project names that contains non-Unicode characters.
  - [47919619](https://www.github.com/tauri-apps/tauri/commit/47919619815900fc3af47ec5873e31afb778b0ad) feat(bundler): allow setting wix language, closes [#1976](https://www.github.com/tauri-apps/tauri/pull/1976) ([#1988](https://www.github.com/tauri-apps/tauri/pull/1988)) on 2021-06-15
- Fixes resource bundling on Windows when there is nested resource folders.
  - [35a20527](https://www.github.com/tauri-apps/tauri/commit/35a2052771fc0897064ed146d9557527a0a76453) fix(bundler): windows resources bundling with nested folders ([#1878](https://www.github.com/tauri-apps/tauri/pull/1878)) on 2021-05-21

## \[1.0.0-beta.0]

- Fixes the `Installed-Size` value on the debian package.
  - [8e0d4f6](https://www.github.com/tauri-apps/tauri/commit/8e0d4f666c2fbcc3452db9edf87aa726c9ebe4b8) fix(bundler): debian package `Installed-Size` value ([#1735](https://www.github.com/tauri-apps/tauri/pull/1735)) on 2021-05-07
- Use `armhf` as Debian package architecture on `arm` CPUs.
  - [894643c](https://www.github.com/tauri-apps/tauri/commit/894643cdcd7446f63c4a0ab157be3cb8c242952a) feat(bundler): use `armhf` as Debian package architecture on arm CPUs ([#1663](https://www.github.com/tauri-apps/tauri/pull/1663)) on 2021-04-30
- Adds basic library documentation.
  - [fcee4c2](https://www.github.com/tauri-apps/tauri/commit/fcee4c25fc2e83a587e096b26d9f7c374c3db057) refactor(bundler): finish initial documentation, reorganize modules ([#1662](https://www.github.com/tauri-apps/tauri/pull/1662)) on 2021-04-30
- The `PackageTypes` enum now includes all options, including Windows packages.
  - [fcee4c2](https://www.github.com/tauri-apps/tauri/commit/fcee4c25fc2e83a587e096b26d9f7c374c3db057) refactor(bundler): finish initial documentation, reorganize modules ([#1662](https://www.github.com/tauri-apps/tauri/pull/1662)) on 2021-04-30
- Adds `icon_path` field to the `WindowsSettings` struct (defaults to `icons/icon.ico`).
  - [314936e](https://www.github.com/tauri-apps/tauri/commit/314936efbeb3ecaece244da5a1a4a59341d4f76f) feat(bundler): add icon path config instead of enforcing icons/icon.ico ([#1698](https://www.github.com/tauri-apps/tauri/pull/1698)) on 2021-05-03
- Pull latest changes from `create-dmg`, fixing unmount issue.
  - [f1aa120](https://www.github.com/tauri-apps/tauri/commit/f1aa12075f9f649ff6baebc0f8e7a10f1e616cc6) fix(bundler): update create-dmg, fixes [#1571](https://www.github.com/tauri-apps/tauri/pull/1571) ([#1729](https://www.github.com/tauri-apps/tauri/pull/1729)) on 2021-05-06
- Fixes DMG volume icon.
  - [e37e187](https://www.github.com/tauri-apps/tauri/commit/e37e187d4a3c7568463c2546099d03dd5a314f40) fix(bundler): dmg volume icon ([#1730](https://www.github.com/tauri-apps/tauri/pull/1730)) on 2021-05-06
- Added the \`#\[non_exhaustive] attribute where appropriate.
  - [e087f0f](https://www.github.com/tauri-apps/tauri/commit/e087f0f9374355ac4b4a48f94727ef8b26b1c4cf) feat: add `#[non_exhaustive]` attribute ([#1725](https://www.github.com/tauri-apps/tauri/pull/1725)) on 2021-05-05

## \[1.0.0-beta-rc.1]

- Find best available icon for AppImage, follow `.DirIcon` spec.
  - [fbf73f3](https://www.github.com/tauri-apps/tauri/commit/fbf73f3ab53387e68c8cbf9e788820bea0f2f111) fix(bundler): find icon for AppImage, define `.DirIcon`, closes [#749](https://www.github.com/tauri-apps/tauri/pull/749) ([#1594](https://www.github.com/tauri-apps/tauri/pull/1594)) on 2021-04-23
- Allow including custom files on the debian package.
  - [9e87fe6](https://www.github.com/tauri-apps/tauri/commit/9e87fe6a69a8f74c8e61221e36e15b7eb1d19432) feat(bundler): allow including custom files on debian package, fix [#1428](https://www.github.com/tauri-apps/tauri/pull/1428) ([#1613](https://www.github.com/tauri-apps/tauri/pull/1613)) on 2021-04-25
- Adds support to custom WiX template.
  - [ebe755a](https://www.github.com/tauri-apps/tauri/commit/ebe755ac5c37025bae0cf8860e9b04b507f71949) feat: [#1528](https://www.github.com/tauri-apps/tauri/pull/1528) wix supports custom templates ([#1529](https://www.github.com/tauri-apps/tauri/pull/1529)) on 2021-04-25
- Adds support to `wix` fragments for custom .msi installer functionality.
  - [69ea51e](https://www.github.com/tauri-apps/tauri/commit/69ea51ec93a6d4fa90f3482a51f0c6d20c97fa29) feat(bundler): implement wix fragments, closes [#1528](https://www.github.com/tauri-apps/tauri/pull/1528) ([#1601](https://www.github.com/tauri-apps/tauri/pull/1601)) on 2021-04-23
- Adds `skip_webview_install` config under `windows > wix` to disable Webview2 runtime installation after the app install.
  - [d13afec](https://www.github.com/tauri-apps/tauri/commit/d13afec20402b8ddbbf3ceb4349edb1956ed79bc) feat(bundler): add option to skip webview2 runtime installation, closes [#1606](https://www.github.com/tauri-apps/tauri/pull/1606) ([#1612](https://www.github.com/tauri-apps/tauri/pull/1612)) on 2021-04-24

## \[1.0.0-beta-rc.0]

- Append app version and OS architecture on AppImage output filename.
  - [ae76c60](https://www.github.com/tauri-apps/tauri/commit/ae76c60a615602fcb8c1dd824a6ad9fa8f48fe69) fix(bundler): appimage paths and filename ([#1227](https://www.github.com/tauri-apps/tauri/pull/1227)) on 2021-02-13
- The Tauri bundler is now a general purpose library instead of a Cargo custom subcommand.
  - [b1e6b74](https://www.github.com/tauri-apps/tauri/commit/b1e6b74a4f624b623a840686fb1abe1d23593867) refactor(cli): decouple bundler from cargo ([#1269](https://www.github.com/tauri-apps/tauri/pull/1269)) on 2021-02-21
- Rename macOS bundle settings from `osx` to `macOS`.
  - [080f639](https://www.github.com/tauri-apps/tauri/commit/080f6391bac4fd59e9e71b9785d7a2f835703805) refactor(bundler): specific settings on dedicated structs, update README ([#1380](https://www.github.com/tauri-apps/tauri/pull/1380)) on 2021-03-25
- The `dev` and `build` pipeline is now written in Rust.
  - [3e8abe3](https://www.github.com/tauri-apps/tauri/commit/3e8abe376407bb0ca8893602590ed9edf7aa71a1) feat(cli) rewrite the core CLI in Rust ([#851](https://www.github.com/tauri-apps/tauri/pull/851)) on 2021-01-30
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Update all code files to have our license header.
  - [bf82136](https://www.github.com/tauri-apps/tauri/commit/bf8213646689175f8a158b956911f3a43e360690) feat(license): SPDX Headers ([#1449](https://www.github.com/tauri-apps/tauri/pull/1449)) on 2021-04-11
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Alpha version of tauri-updater. Please refer to the `README` for more details.
  - [6d70c8e](https://www.github.com/tauri-apps/tauri/commit/6d70c8e1e256fe839c4a947375bb529d7b4f7301) feat(updater): Alpha version ([#643](https://www.github.com/tauri-apps/tauri/pull/643)) on 2021-04-05
  - [a6def70](https://www.github.com/tauri-apps/tauri/commit/a6def7066eec19c889b0f14cc1e475bf209a332e) Refactor(tauri): move tauri-api and tauri-updater to tauri ([#1455](https://www.github.com/tauri-apps/tauri/pull/1455)) on 2021-04-11
  - [aea6145](https://www.github.com/tauri-apps/tauri/commit/aea614587bddab930d552512b54e18624fbf573e) refactor(repo): add /tooling folder ([#1457](https://www.github.com/tauri-apps/tauri/pull/1457)) on 2021-04-12
- Bundle Visual C++ redistributable files with VC142\_CRT merge modules.
  - [3047a18](https://www.github.com/tauri-apps/tauri/commit/3047a18975db07abdf49985f531c3925f68a0db9) feat(bundler): add visual c++ redistributable files with MSM ([#1368](https://www.github.com/tauri-apps/tauri/pull/1368)) on 2021-03-22
- Automatically install Webview2 runtime alongside app.
  - [8e9752b](https://www.github.com/tauri-apps/tauri/commit/8e9752bb8bad5c56b55a3750876e0073efdc6d39) feat(bundler/wix): install webview2 runtime ([#1329](https://www.github.com/tauri-apps/tauri/pull/1329)) on 2021-03-07
- Fixes the bundler workspace detection.
  - [e34ee4c](https://www.github.com/tauri-apps/tauri/commit/e34ee4c29c7fde02e09685a3100f0b2ef6380c98) fix(bundler): workspace detection, closes [#1007](https://www.github.com/tauri-apps/tauri/pull/1007) ([#1235](https://www.github.com/tauri-apps/tauri/pull/1235)) on 2021-02-14

## \[0.9.4]

- `dirs` crate is unmaintained, now using `dirs-next` instead.
  - [82cda98](https://www.github.com/tauri-apps/tauri/commit/82cda98532975c6d4b1c93bf2f326173f39e0964) chore(tauri) `dirs` crate is unmaintained, use `dirst-next` instead ([#1057](https://www.github.com/tauri-apps/tauri/pull/1057)) on 2020-10-17
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Force IPv4 on `wget` so AppImage bundling doesn't hang.
  - [6f5667b](https://www.github.com/tauri-apps/tauri/commit/6f5667bf72d58972b8d05ee2e42a031c85f95ed4) fix: [#1018](https://www.github.com/tauri-apps/tauri/pull/1018) Force IPv4 on wget requests ([#1019](https://www.github.com/tauri-apps/tauri/pull/1019)) on 2020-10-11
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21
- Set the Windows installer (WiX) `WorkingDirectory` field to `INSTALLDIR` so the app can read paths relatively (previously resolving to `C:\Windows\System32`).
  - [5cf3402](https://www.github.com/tauri-apps/tauri/commit/5cf3402735ac2e88fc4aae5fe39fc0a41262b6fa) fix: add working directory to wix's shortcut ([#1021](https://www.github.com/tauri-apps/tauri/pull/1021)) on 2020-09-24
  - [72996be](https://www.github.com/tauri-apps/tauri/commit/72996be1bd3eb878c4cf30bfec79058071c26d7a) apply version updates ([#1024](https://www.github.com/tauri-apps/tauri/pull/1024)) on 2020-10-21

## \[0.9.3]

- Improve checking for Xcode command line tools to allow builds on mac
  - [7a788fd](https://www.github.com/tauri-apps/tauri/commit/7a788fdceebc2bf6b7b46ebe54e98597d4a71529) fix: improve checking for Rez (fix [#994](https://www.github.com/tauri-apps/tauri/pull/994)) ([#995](https://www.github.com/tauri-apps/tauri/pull/995)) on 2020-08-28
- add a newline after Categories in deb .desktop file generation to fix issues #899 and #925.
  - [37bcf5f](https://www.github.com/tauri-apps/tauri/commit/37bcf5fea154bdefbca2692b69aafaabba8c23e2) fix(bundler) missing newline in deb desktop file generation (fix: [#899](https://www.github.com/tauri-apps/tauri/pull/899), [#925](https://www.github.com/tauri-apps/tauri/pull/925)) ([#998](https://www.github.com/tauri-apps/tauri/pull/998)) on 2020-08-27

## \[0.9.2]

- Bump all deps as noted in #975, #976, #977, #978, and #979.
  - [06dd75b](https://www.github.com/tauri-apps/tauri/commit/06dd75b68a15d388808c51ae2bf50554ae761d9d) chore: bump all js/rust deps ([#983](https://www.github.com/tauri-apps/tauri/pull/983)) on 2020-08-20

## \[0.9.1]

- Hide external scripts output unless `--verbose` is passed.
  - [78add1e](https://www.github.com/tauri-apps/tauri/commit/78add1e79ef42ed61e988a0012be87b428439332) feat(bundler): hide output from shell scripts unless --verbose is passed (fixes [#888](https://www.github.com/tauri-apps/tauri/pull/888)) ([#893](https://www.github.com/tauri-apps/tauri/pull/893)) on 2020-07-26
- Fixes the target directory detection, respecting the `CARGO_TARGET_DIR` and `.cargo/config (build.target-dir)` options to set the Cargo output directory.
  - [63b9c64](https://www.github.com/tauri-apps/tauri/commit/63b9c6457233d777b698b53cd6661c6cd9a0d67b) fix(bundler) properly detect the target directory ([#895](https://www.github.com/tauri-apps/tauri/pull/895)) on 2020-07-25
- Bundling every DLL file on the binary directory.
  - [a00ac02](https://www.github.com/tauri-apps/tauri/commit/a00ac023eef9f7b3a508ca9acd664470382d7d06) fix(bundler) webview dll not being bundled, fixes [#875](https://www.github.com/tauri-apps/tauri/pull/875) ([#889](https://www.github.com/tauri-apps/tauri/pull/889)) on 2020-07-24

## \[0.9.0]

- Fixes the AppImage bundling on containers.
  - [53e8dc1](https://www.github.com/tauri-apps/tauri/commit/53e8dc1880b78994e17bf769d60e13f9e13dbf99) fix(bundler) support AppImage bundling on containers [#822](https://www.github.com/tauri-apps/tauri/pull/822) on 2020-07-13
  - [bd0118f](https://www.github.com/tauri-apps/tauri/commit/bd0118f160360e588180de3f3518ef47a4d86a46) fix(changes) covector status pass on 2020-07-14
- Bundler output refactor: move Windows artifacts to the `bundle/wix` folder and use a standard output name `${bundleName}_${version}_${arch}.${extension}`.
  - [9130f1b](https://www.github.com/tauri-apps/tauri/commit/9130f1b1a422121fa9f3afbeeb87e851568e995f) refactor(bundler) standard output names and path ([#823](https://www.github.com/tauri-apps/tauri/pull/823)) on 2020-07-13

## \[0.8.5]

- Ignoring non UTF-8 characters on the loopback command output.
  - [f340b29](https://www.github.com/tauri-apps/tauri/commit/f340b2914dc9c3a94ca8606f4663964fa87b95ea) fix(tauri) addition to the previous commit on 2020-07-10

## \[0.8.4]

- Properly run the loopback command on Windows.

## \[0.8.3]

- Fixes the unbound variable issue on the DMG bundler script.

## \[0.8.2]

- Fixes the AppImage bundler script.

## \[0.8.1]

- Improves the logging of child processes like bundle_appimage.sh and bundle_dmg.sh.

## \[0.8.0]

- The bundler now bundles all binaries from your project (undefined) and undefined.
  When multiple binaries are used, make sure to use the undefined config field.
- Check if mksquashfs is installed before bundling AppImage

## \[0.7.0]

- Fixes AppImage bundler (appimagetool usage, build script running properly, proper AppRun and .desktop files).
