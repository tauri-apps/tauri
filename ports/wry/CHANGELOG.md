# Changelog

## \[0.55.1]

- [`3860359`](https://github.com/tauri-apps/wry/commit/3860359e3dc75260fb9865f7a8eae00a9ec4733d) ([#1721](https://github.com/tauri-apps/wry/pull/1721) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Tweak ProGuard rules to keep the getId required activity method.

## \[0.55.0]

- [`a535fd9`](https://github.com/tauri-apps/wry/commit/a535fd95025d126dd1d6ad10b101b81ed9ae40fe) ([#1699](https://github.com/tauri-apps/wry/pull/1699) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) On Android, fixed panics on custom protocol timeouts. Increased timeout for non-internal requests to 30 seconds.
- [`781f371`](https://github.com/tauri-apps/wry/commit/781f371893d58c9dc81f2adce84adfad9e390833) ([#1633](https://github.com/tauri-apps/wry/pull/1633) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Update to latest tao to support multi-window on Android and iOS.
- [`848aff7`](https://github.com/tauri-apps/wry/commit/848aff722a72cc30be01c992734000933f74c2d0) ([#1660](https://github.com/tauri-apps/wry/pull/1660) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Run `new_window_req_handler` on main thread and remove the `Send + Sync` restriction on it
- [`92114b2`](https://github.com/tauri-apps/wry/commit/92114b2efb531ec8191734817ebcda952e204210) ([#1634](https://github.com/tauri-apps/wry/pull/1634) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Call `onNewIntent` for tao's 0.35 Opened event.
- [`05440b8`](https://github.com/tauri-apps/wry/commit/05440b80ed79a84faa264a91dbcd952d9e77f78f) ([#1688](https://github.com/tauri-apps/wry/pull/1688) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Removed `drag-drop` feature, previously gated `WebViewAttributes::drag_drop_handler` and `WebViewBuilder::with_drag_drop_handler` are now always available
- [`222c63d`](https://github.com/tauri-apps/wry/commit/222c63d69f03639bbd1f456ae2d9d2f26a2d549c) ([#1698](https://github.com/tauri-apps/wry/pull/1698) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Remove `WebViewAttributes`, `WebView::new` and `WebView::new_as_child` from public API. Use `WebViewBuilder` instead.

## \[0.54.4]

- [`47d9470`](https://github.com/tauri-apps/wry/commit/47d9470e40160af8209da094364141917ec677a7) ([#1693](https://github.com/tauri-apps/wry/pull/1693) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Updated dependency dom_query to 0.27.0, this fixed a bug where initialization scripts were escaped incorrectly on Android
- [`1540788`](https://github.com/tauri-apps/wry/commit/1540788a0161235e1b89c70912725fd2f18cb813) ([#1692](https://github.com/tauri-apps/wry/pull/1692) by [@Sbenazar](https://github.com/tauri-apps/wry/../../Sbenazar)) fix(webkitgtk): normalize background color values to 0.0–1.0

## \[0.54.3]

- [`40a7032`](https://github.com/tauri-apps/wry/commit/40a703214d3e66942edf6f6a96074580798d5046) ([#1677](https://github.com/tauri-apps/wry/pull/1677) by [@russellmcc](https://github.com/tauri-apps/wry/../../russellmcc)) Fix bug where wry would crash when loaded by multiple dylibs in the same process on macOS.

### Dependencies

- [`146e36e`](https://github.com/tauri-apps/wry/commit/146e36e68b5da3c8e4488a67c8661695e76e60eb) ([#1671](https://github.com/tauri-apps/wry/pull/1671) by [@thomaseizinger](https://github.com/tauri-apps/wry/../../thomaseizinger)) Refactors HTML manipulation on Android to use `dom_query` instead of `kuchikiki`.

### enhance

- [`e430558`](https://github.com/tauri-apps/wry/commit/e430558f4be90096a1bf786cdb59c49a0754beec) ([#1649](https://github.com/tauri-apps/wry/pull/1649) by [@aoxiangtianyu-go](https://github.com/tauri-apps/wry/../../aoxiangtianyu-go)) Add an option to control browser-level autofill behavior on Windows.

### bug

- [`a7de137`](https://github.com/tauri-apps/wry/commit/a7de137ecc321bf3c3a3a10c857b5839741b4b5c) ([#1680](https://github.com/tauri-apps/wry/pull/1680) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Fix infinite loop on main thread if creating the webview failed or getting cookie failed

## \[0.54.2]

- [`1d1b870`](https://github.com/tauri-apps/wry/commit/1d1b8707665b4638e040dde7a88cd33d77ba2ebc) ([#1662](https://github.com/tauri-apps/wry/pull/1662) by [@JSKitty](https://github.com/tauri-apps/wry/../../JSKitty)) On macOS, implement `background_color` support for WKWebView behind the `transparent` feature. Disables the default white background via the `drawsBackground` KVC key at init and applies `underPageBackgroundColor` on macOS 12+ for both initial creation and runtime `set_background_color` calls.
- [`e3761e5`](https://github.com/tauri-apps/wry/commit/e3761e5e5ab684ae277dfa7df931ed9ff4c5bc38) ([#1668](https://github.com/tauri-apps/wry/pull/1668) by [@zeta-gundam](https://github.com/tauri-apps/wry/../../zeta-gundam)) Fixed an issue that caused "Symbol not found: \_NSHTTPCookieSameSite..." panics on macOS `10.13` & `10.14`.
- [`e3761e5`](https://github.com/tauri-apps/wry/commit/e3761e5e5ab684ae277dfa7df931ed9ff4c5bc38) ([#1668](https://github.com/tauri-apps/wry/pull/1668) by [@zeta-gundam](https://github.com/tauri-apps/wry/../../zeta-gundam)) Fixed an issue that caused "\[\<WKWebViewConfiguration 0x6000003f4d00> setValue:forUndefinedKey:]: this class is not key value coding-compliant for the key drawsBackground." panics on macOS `10.13`

## \[0.54.1]

- [`6704b20`](https://github.com/tauri-apps/wry/commit/6704b207ac985e968be0b5c26b898ca152ae47d9) ([#1655](https://github.com/tauri-apps/wry/pull/1655)) Adds a `PartialEq` derive to `Rect`.

## \[0.54.0]

- [`5fdd1a9`](https://github.com/tauri-apps/wry/commit/5fdd1a9840e24ce77ea4a57ce3746c3dbee28c9c) ([#1625](https://github.com/tauri-apps/wry/pull/1625)) Use OnBackPressedCallback instead of the deprecated onKeyDown for back navigation on Android.
- [`51d06d0`](https://github.com/tauri-apps/wry/commit/51d06d0cecdf69652a0ae636b1b875e427534bc3) ([#1646](https://github.com/tauri-apps/wry/pull/1646)) Remove redundant clones in WebView and download handling. No user facing changes.
- [`7b77322`](https://github.com/tauri-apps/wry/commit/7b77322f886c440d035674b8efa586671dc0a3db) ([#1647](https://github.com/tauri-apps/wry/pull/1647)) Fix clippy io_other_error. No user facing changes.
- [`ff8876d`](https://github.com/tauri-apps/wry/commit/ff8876d92b9681b139d6637220cefe3248b7d3fb) ([#1652](https://github.com/tauri-apps/wry/pull/1652)) Improve wkwebview performance. No user-facing changes.
- [`27ba73f`](https://github.com/tauri-apps/wry/commit/27ba73f07720b80a21b6b876f1229cad8871aa33) Add handler for web content process termination.
- [`4fcff14`](https://github.com/tauri-apps/wry/commit/4fcff147b5022af3ee006ed077759b1648b32aa6) ([#1547](https://github.com/tauri-apps/wry/pull/1547)) Set `WebsiteDataManagerBuilder::base_cache_directory` with the same path as `base_data_directory`.

  This change allows the cache directory to be changed instead of using the default one [from WebKitGTK](https://webkitgtk.org/reference/webkit2gtk/stable/property.WebsiteDataManager.base-cache-directory.html).
- [`64d9296`](https://github.com/tauri-apps/wry/commit/64d92960b960a4b2fdf9205b6188f55650ddd62f) ([#1640](https://github.com/tauri-apps/wry/pull/1640)) Update webkit2gtk-rs crates to `2.0.2`.

## \[0.53.5]

- [#1622](https://github.com/tauri-apps/wry/pull/1626) Fixed an issue that caused docs.rs builds to fail. No user facing changes.

## \[0.53.4]

- [`093856a`](https://github.com/tauri-apps/wry/commit/093856a2a53a6fc1aaa759e048c7e1fe31bb09fa) ([#1622](https://github.com/tauri-apps/wry/pull/1622) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Add flag to opt out of automatic back navigation handling on Android via `WryActivity#handleBackNavigation`.
- [`0f51d67`](https://github.com/tauri-apps/wry/commit/0f51d67485d84fd9c72391379a67567eea3cbbfe) ([#1605](https://github.com/tauri-apps/wry/pull/1605) by [@dgerhardt](https://github.com/tauri-apps/wry/../../dgerhardt)) On Linux, removed a workaround which forced inital requests for multiple webviews to be handled sequentially.
  The workaround was intended to fix a concurrency bug with loading multiple URIs at the same time on WebKitGTK.
  But it prevented parallelization and could cause a deadlock in certain situations.
  It is no longer needed with newer WebKitGTK versions.

## \[0.53.3]

- [`6aa5854`](https://github.com/tauri-apps/wry/commit/6aa5854b0371a4828638cd722e18d9b1ab235a8b) ([#1609](https://github.com/tauri-apps/wry/pull/1609) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Enhance error handling of the `webview_version` function on macOS.

## \[0.53.2]

- [`1743e7f`](https://github.com/tauri-apps/wry/commit/1743e7f9c1113db1639d544f1f12b8835f91e20b) ([#1608](https://github.com/tauri-apps/wry/pull/1608) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix new_window_req_handler craashing when creating new window for webview that has a custom data_store_identifier.

## \[0.53.1]

- [`ccdf912`](https://github.com/tauri-apps/wry/commit/ccdf912621f5e12698bdb2cb75af81dd794ab751) ([#1603](https://github.com/tauri-apps/wry/pull/1603) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix the build when not enabling the `x11` feature.

## \[0.53.0]

- [`1456f8e`](https://github.com/tauri-apps/wry/commit/1456f8e33ad3d05239c2ee71f64e49ef32d48f03) ([#1602](https://github.com/tauri-apps/wry/pull/1602) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Enabled all downloads by default to match WebView2 and browser behavior on all platforms. To disable this, provide a custom `download_started_handler`.
- [`43e78ff`](https://github.com/tauri-apps/wry/commit/43e78ff6b09fe9cac84ff7d73460f170a8c32081) ([#1588](https://github.com/tauri-apps/wry/pull/1588) by [@zphrs](https://github.com/tauri-apps/wry/../../zphrs)) Add `WebViewBuilder::with_limit_navigations_to_app_bound_domains` only on iOS.
  Function is a no-op if iOS version is less than iOS 14.
- [`60dba38`](https://github.com/tauri-apps/wry/commit/60dba38ddcc01c428feccea2957adf69128373ef) ([#1569](https://github.com/tauri-apps/wry/pull/1569) by [@WSH032](https://github.com/tauri-apps/wry/../../WSH032)) Add `WebView::set_cookie` and `WebView::delete_cookie` APIs.
- [`88cbb01`](https://github.com/tauri-apps/wry/commit/88cbb019a8d38fbcf7ad015d5f34f9a838a88f98) ([#1561](https://github.com/tauri-apps/wry/pull/1561) by [@dgerhardt](https://github.com/tauri-apps/wry/../../dgerhardt)) On Linux, fix a deadlock, which could occur when destroying a WebView before loading has finished.
- [`eb562ca`](https://github.com/tauri-apps/wry/commit/eb562ca99346fabf2c28d65252d176e2612f3178) ([#1596](https://github.com/tauri-apps/wry/pull/1596) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Do not fire `new_window_req_handler` on navigation events on macOS and iOS.
- [`78634b3`](https://github.com/tauri-apps/wry/commit/78634b3d4dd31651f9e6a3959cbb5f994502a92b) ([#1594](https://github.com/tauri-apps/wry/pull/1594) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) On Linux, macOS and iOS, the `download_started_handler` will now get the correct suggested destination path.
- [`5399823`](https://github.com/tauri-apps/wry/commit/5399823a1ecfca4476e7c585c694380c41951766) ([#1600](https://github.com/tauri-apps/wry/pull/1600) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes `new_window_req_handler` not creating the webview on Linux when executing `window.open()`.
- [`eb562ca`](https://github.com/tauri-apps/wry/commit/eb562ca99346fabf2c28d65252d176e2612f3178) ([#1596](https://github.com/tauri-apps/wry/pull/1596) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes `new_window_req_handler` not fired on macOS when executing `window.open()`.
- [`3f978d3`](https://github.com/tauri-apps/wry/commit/3f978d3290aa5cebd6b5feea506557ac0f6e07a7) ([#1601](https://github.com/tauri-apps/wry/pull/1601) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Refactor `WebViewBuilder::with_new_window_req_handler` to allow creating the webview manually.
- [`3f978d3`](https://github.com/tauri-apps/wry/commit/3f978d3290aa5cebd6b5feea506557ac0f6e07a7) ([#1601](https://github.com/tauri-apps/wry/pull/1601) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Disable new window creation by default on Windows to match behavior of other platforms. Use `WebViewBuilder::with_new_window_req_handler` to enable.

## \[0.52.1]

- [`63eaab8`](https://github.com/tauri-apps/wry/commit/63eaab80bad7c5f888893c79690c5b626d015eb3) ([#1573](https://github.com/tauri-apps/wry/pull/1573) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Fix `wry::DragDropEvent::Drop::paths` returns random data on Windows

## \[0.52.0]

- [`5e6b0e6`](https://github.com/tauri-apps/wry/commit/5e6b0e689e38068c817ed1beb87af60b0fcbe0e2) ([#1555](https://github.com/tauri-apps/wry/pull/1555) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Added iOS support for `set_background_color`.
- [`99dab51`](https://github.com/tauri-apps/wry/commit/99dab51fef0dfd49647a8e568f383553bbab6551) ([#1570](https://github.com/tauri-apps/wry/pull/1570) by [@renovate](https://github.com/tauri-apps/wry/../../renovate)) Updated `webview2-com` to `0.38`.
- [`41f4a3a`](https://github.com/tauri-apps/wry/commit/41f4a3a65e0c2ec8e63eedaee9b7fa4d6d3f4fa7) ([#1572](https://github.com/tauri-apps/wry/pull/1572) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) On Windows on systems running WebView2 v137+ wry now uses a new default background color API which should reduce white flashes. The use of the `RemoveRedirectionBitmap` browser flag (v134+) was removed due to crashes on Insider builds.
- [`eb40ac8`](https://github.com/tauri-apps/wry/commit/eb40ac8b2ab691bbc0e75bc9c27ba53e2f645e03) ([#1528](https://github.com/tauri-apps/wry/pull/1528) by [@aurelj](https://github.com/tauri-apps/wry/../../aurelj)) Added `x11` feature flag (enabled by default).

### breaking

- [`1567635`](https://github.com/tauri-apps/wry/commit/1567635ba5f660827d4ae20f4e226ed7fa595f12) ([#1558](https://github.com/tauri-apps/wry/pull/1558) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Rename `WebViewBuilder` methods for consistency and clarity:

  - Renamed `WebViewBuilder::with_web_context` to `WebViewBuilder::new_with_web_context`
  - Renamed `WebViewBuilder::with_attributes` to `WebViewBuilder::new_with_attributes`
- [`f868658`](https://github.com/tauri-apps/wry/commit/f868658d6ffbf0a6b944faebb3b7565726c82f57) ([#1556](https://github.com/tauri-apps/wry/pull/1556) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) `WebContext::is_custom_protocol_registered` now takes `&str` instead of `String`

## \[0.51.2]

- [`f7781a7`](https://github.com/tauri-apps/wry/commit/f7781a788dbb0d07bea27f18daf844d70a3958a3) ([#1544](https://github.com/tauri-apps/wry/pull/1544) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Allow modifying or removing the input accessory view on iOS via `WebViewBuilderExtIos::with_input_accessory_view_builder`.

## \[0.51.1]

- [`3e24d6b`](https://github.com/tauri-apps/wry/commit/3e24d6b7b191ce37298ad6be99614ccdf04c7d3f) ([#1543](https://github.com/tauri-apps/wry/pull/1543) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix async custom protocol not responding.

## \[0.51.0]

- [`f99bbd9`](https://github.com/tauri-apps/wry/commit/f99bbd97c6f46492a2f8b34894532629ce17550d) ([#1507](https://github.com/tauri-apps/wry/pull/1507) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) Renamed `Error::UrlPrase` to `Error::UrlParse` to fix typo.
- [`2d753c6`](https://github.com/tauri-apps/wry/commit/2d753c64822e05ed28621009ef9383ffb70b1b94) ([#1531](https://github.com/tauri-apps/wry/pull/1531) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) **Breaking Change**: `WebViewAttributes::initialization_scripts` takes `Vec::<InitializationScript>` now instead of `Vec::<(String, bool)>`
- [`78b83a0`](https://github.com/tauri-apps/wry/commit/78b83a0d8a5db3c58140d7a3f00a52616049251e) ([#1537](https://github.com/tauri-apps/wry/pull/1537) by [@Brendonovich](https://github.com/tauri-apps/wry/../../Brendonovich)) Moved protocol handler functions to a thread local instead of storing them as ivars to prevent a race condition between webview close and custom protocol handling.
- [`5f1e4ba`](https://github.com/tauri-apps/wry/commit/5f1e4bafeee6aab8cdc1d7ebc5d13bb6cfeb94ad) ([#1538](https://github.com/tauri-apps/wry/pull/1538) by [@syrel](https://github.com/tauri-apps/wry/../../syrel)) macOS: Handle flipped coordinates when adding a WebView as a child
- [`4ec951a`](https://github.com/tauri-apps/wry/commit/4ec951a7b5d0484e19f2d94665c79380171ec9ba) ([#1526](https://github.com/tauri-apps/wry/pull/1526) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Updated `webview2-com` to `0.37`, `windows` to `0.61`.
- [`2d753c6`](https://github.com/tauri-apps/wry/commit/2d753c64822e05ed28621009ef9383ffb70b1b94) ([#1531](https://github.com/tauri-apps/wry/pull/1531) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Init scripts are always executed on all frames on Windows WebView2
- [`2d753c6`](https://github.com/tauri-apps/wry/commit/2d753c64822e05ed28621009ef9383ffb70b1b94) ([#1531](https://github.com/tauri-apps/wry/pull/1531) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Fix init script sometimes get executed too late on Windows WebView2

### feat

- [`ecbced2`](https://github.com/tauri-apps/wry/commit/ecbced25e5e41c6a9f7816ea6517763d30c5b060) ([#1534](https://github.com/tauri-apps/wry/pull/1534) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) macOS/iOS: add option to disable link previews when building a webview (the webkit API has it enabled by default)

  - `WebViewBuilderExtDarwin.with_allow_link_preview(bool)`

## \[0.50.5]

### enhance

- [`353bd95`](https://github.com/tauri-apps/wry/commit/353bd9573a1ff80aa1e547e82f8c32ab2f984f9f) ([#1517](https://github.com/tauri-apps/wry/pull/1517) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Added a Windows-only setting to disable all Webview2 context menus.

### bug

- [`4f4ade3`](https://github.com/tauri-apps/wry/commit/4f4ade3c9564bb33b6e54488c228a4c5b054204f) ([#1520](https://github.com/tauri-apps/wry/pull/1520) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix crash setting macOS traffic light buttons inset when window is undecorated or buttons have been removed.

## \[0.50.4]

- [`349dfe3`](https://github.com/tauri-apps/wry/commit/349dfe37d25f71ed60143f7ec36153bedc296cfa) ([#1512](https://github.com/tauri-apps/wry/pull/1512) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) Added `WebViewExtDarwin` to expose WebView functions available to both macOS and iOS.
- [`349dfe3`](https://github.com/tauri-apps/wry/commit/349dfe37d25f71ed60143f7ec36153bedc296cfa) ([#1512](https://github.com/tauri-apps/wry/pull/1512) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) fix: crash when using `WebViewBuilderExtDarwin.with_data_store_identifier`
- [`349dfe3`](https://github.com/tauri-apps/wry/commit/349dfe37d25f71ed60143f7ec36153bedc296cfa) ([#1512](https://github.com/tauri-apps/wry/pull/1512) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) feat: macOS: add `WebViewExtDarwin::fetch_data_store_identifiers` and `WebViewExtDarwin::remove_data_store`.
- [`d0b16a7`](https://github.com/tauri-apps/wry/commit/d0b16a7b89dfed084405bd7058020da1a4b6708a) ([#1491](https://github.com/tauri-apps/wry/pull/1491) by [@neilmcguire](https://github.com/tauri-apps/wry/../../neilmcguire)) On Windows, Add support for iframe requests in custom protocols. Requires WebView2 1.0.2365.46 or higher.
- [`148d7cd`](https://github.com/tauri-apps/wry/commit/148d7cdd23de91aba541f935aa23d2faf5558199) ([#1513](https://github.com/tauri-apps/wry/pull/1513) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Wry by default now passes `--enable-features=RemoveRedirectionBitmap` to WebView2 to hide the initial white flash of newly created webviews. Only takes effect on WebView2 Runtime versions 134 and above.

## \[0.50.3]

- [`94ecadb`](https://github.com/tauri-apps/wry/commit/94ecadbc73d6357484a4df7358c96e72111ce4be) ([#1496](https://github.com/tauri-apps/wry/pull/1496) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) Add `WebViewBuilder.with_javascript_disabled` api to disable JavaScript.
- [`5120a5c`](https://github.com/tauri-apps/wry/commit/5120a5cc0fd4b9d1df32812d3df121c632821395) ([#1486](https://github.com/tauri-apps/wry/pull/1486) by [@charrondev](https://github.com/tauri-apps/wry/../../charrondev)) Fix `Webview::cookies` and `Webview::cookies_for_url` deadlock on macOS.
- [`8dfeb76`](https://github.com/tauri-apps/wry/commit/8dfeb7650213fc10edbf7b03acc9a95107c7e342) ([#1500](https://github.com/tauri-apps/wry/pull/1500) by [@Simon-Laux](https://github.com/tauri-apps/wry/../../Simon-Laux)) feat: add `Webview.reload`

## \[0.50.2]

- [`cef818f`](https://github.com/tauri-apps/wry/commit/cef818f63f36b9acfc4d31f17b5e9fe7c3058612) ([#1505](https://github.com/tauri-apps/wry/pull/1505) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix `already mutably borrowed: BorrowError` panic on webview initialization on Android.

## \[0.50.1]

- [`f9abf6b`](https://github.com/tauri-apps/wry/commit/f9abf6b4464acc91926236366a347f04d741b15d) ([#1501](https://github.com/tauri-apps/wry/pull/1501) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Fix webview not resized with parent when it gets maximized on Windows

## \[0.50.0]

- [`933de78`](https://github.com/tauri-apps/wry/commit/933de788bc0fbab1914b7c5ce29c209281996b03) ([#1492](https://github.com/tauri-apps/wry/pull/1492) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Updated `webview2-com` to `0.36`, `windows` to `0.60`.

## \[0.49.0]

- [`0395df5`](https://github.com/tauri-apps/wry/commit/0395df504ffaea2320d706501c89859a6e319d30) ([#1484](https://github.com/tauri-apps/wry/pull/1484) by [@alexmoon](https://github.com/tauri-apps/wry/../../alexmoon)) Removed `obj-exception` feature.
- [`c27b4ff`](https://github.com/tauri-apps/wry/commit/c27b4ffc05b6bc9bb3748a1203116342cfec680d) ([#1468](https://github.com/tauri-apps/wry/pull/1468) by [@madsmtm](https://github.com/tauri-apps/wry/../../madsmtm)) Update to `objc2` v0.6.

  This bumps MSRV on macOS/iOS to 1.71.
- [`95a9319`](https://github.com/tauri-apps/wry/commit/95a9319f24cb62add69d468ec0f60530b608fe6b) ([#1454](https://github.com/tauri-apps/wry/pull/1454) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Updated `webview2-com` to `0.35`, `windows` to `0.59`.
- [`9df094a`](https://github.com/tauri-apps/wry/commit/9df094aa79210c6743b5d295069931afaee596db) ([#1483](https://github.com/tauri-apps/wry/pull/1483) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) On Windows, fix webview slightly larger than the host window causing a pixel or two to be obscured.
- [`ce98c34`](https://github.com/tauri-apps/wry/commit/ce98c3401d6bc3079900ced9007f212f4f30099c) ([#1480](https://github.com/tauri-apps/wry/pull/1480) by [@ahqsoftwares](https://github.com/tauri-apps/wry/../../ahqsoftwares)) Fixed an issue that could cause `Return type mismatch: expected 'kotlin.String', actual 'kotlin.String?'` errors.

### enhance

- [`0185644`](https://github.com/tauri-apps/wry/commit/0185644040184c43084814cb8692acb4e1004d86) ([#1452](https://github.com/tauri-apps/wry/pull/1452) by [@mzdk100](https://github.com/tauri-apps/wry/../../mzdk100)) Allow the use of TAB to cycle through focus elements in an HTML document.

## \[0.48.1]

- [`cbbcccc`](https://github.com/tauri-apps/wry/commit/cbbcccc38af7d900a0f8f7fa5ea5e6667765ed81) ([#1446](https://github.com/tauri-apps/wry/pull/1446) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Add functionality to set the traffic light inset on macOS. This is required to prevent flickers if the WebView is injected via `build()` instead of `build_as_child()`.

### enhance

- [`26abf63`](https://github.com/tauri-apps/wry/commit/26abf63d79c4a0fcff8ea39d0ff677686f5b546c) ([#1445](https://github.com/tauri-apps/wry/pull/1445) by [@bastiankistner](https://github.com/tauri-apps/wry/../../bastiankistner)) Add an option to change the default background throttling policy (currently for WebKit only).

## \[0.48.0]

- [`eb0c816`](https://github.com/tauri-apps/wry/commit/eb0c8163f1e2a11115ef1928b5e4bf0a8f083f3b) ([#1423](https://github.com/tauri-apps/wry/pull/1423) by [@SpikeHD](https://github.com/tauri-apps/wry/../../SpikeHD)) Rename `{WebViewBuilderExtWindows, WebViewBuilderExtUnix}::with_extension_path` to `with_extensions_path`.
- [`b4ba5b4`](https://github.com/tauri-apps/wry/commit/b4ba5b47f235df503085150671ff16189d3d9ea9) ([#1426](https://github.com/tauri-apps/wry/pull/1426) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) On Windows, Wry will now log PostMessage errors instead of panicking.
- [`f0c6b94`](https://github.com/tauri-apps/wry/commit/f0c6b947dddc8f1b208157a664a6cc940f123b3c) ([#1441](https://github.com/tauri-apps/wry/pull/1441) by [@renovate](https://github.com/tauri-apps/wry/../../renovate)) Updated `webview2-com` to `0.34`.

## \[0.47.2]

- [`7bb4f49`](https://github.com/tauri-apps/wry/commit/7bb4f4929eddbde8f36472a55ec3713d6d51c0e3) ([#1421](https://github.com/tauri-apps/wry/pull/1421) by [@SpikeHD](https://github.com/tauri-apps/wry/../../SpikeHD)) Fix extension loading on Windows.

## \[0.47.1]

- [`59c1eef`](https://github.com/tauri-apps/wry/commit/59c1eef0805ecbb1f70a7c78578ff4e03b09a204) ([#1418](https://github.com/tauri-apps/wry/pull/1418) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Fix initialization scripts running twice on Windows.

## \[0.47.0]

- [`7221256`](https://github.com/tauri-apps/wry/commit/72212568cb4d815463fc035969f9cac60fe28ba6) ([#1365](https://github.com/tauri-apps/wry/pull/1365) by [@Norbiros](https://github.com/tauri-apps/wry/../../Norbiros)) Add `WebViewBuilder::with_initialization_script_for_main_only` to enable injecting JavaScript code into main frame only or all subframes.
- [`c1b26b9`](https://github.com/tauri-apps/wry/commit/c1b26b9612bf5c5a9e4e0185f73739a2444343cd) ([#1394](https://github.com/tauri-apps/wry/pull/1394) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Add `WebView::cookies` and `WebView::cookies_for_url` APIs.
- [`c193e2a`](https://github.com/tauri-apps/wry/commit/c193e2a04c369b7699cd0d73049e84b6851b5c06) ([#1408](https://github.com/tauri-apps/wry/pull/1408) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Fix `DragDropEvent::Drop` event never fired on Wayland (and sometimes on X11).
- [`1d63fa3`](https://github.com/tauri-apps/wry/commit/1d63fa325327a02a0a8be9ee50ce1eb7a0e8e04f) ([#1403](https://github.com/tauri-apps/wry/pull/1403) by [@SpikeHD](https://github.com/tauri-apps/wry/../../SpikeHD)) Add `WebViewBuilder::with_extension_path` API to Windows and Linux.
- [`0c192f4`](https://github.com/tauri-apps/wry/commit/0c192f4fda1d9c0020bd3ad09a9090bca25ef04f) ([#1414](https://github.com/tauri-apps/wry/pull/1414) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix Android static handlers not being replaced when the application UI is relaunched while still running in the foreground.
- [`9a2a2d4`](https://github.com/tauri-apps/wry/commit/9a2a2d42b635a1e27bd6a8f67f7f7b1c59acc7db) ([#1412](https://github.com/tauri-apps/wry/pull/1412) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Fix icons of dragged items getting stuck when using `WebViewBuilder::with_drag_drop_handler` on some distros like Gnome.
- [`fa9875b`](https://github.com/tauri-apps/wry/commit/fa9875bb16dd967520c41e50c562a5aabccc2cbc) ([#1409](https://github.com/tauri-apps/wry/pull/1409) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) On Windows, disable Webview2's file drop when using `WebViewBuilder::with_drag_drop_handler` which fix drag events for files from "Recent files" view.
- [`6007608`](https://github.com/tauri-apps/wry/commit/600760827735696e4099eff1ad500baa69d84d2f) ([#1400](https://github.com/tauri-apps/wry/pull/1400) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) On Windows, fix webview slightly larger than the window inner size, which resulted in a hidden 1px in the right and bottom borders of the webview

## \[0.46.3]

- [`be122f6`](https://github.com/tauri-apps/wry/commit/be122f667f9f5516b4bc25f9e8c61cb99dbe1440) ([#1397](https://github.com/tauri-apps/wry/pull/1397) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fix `with_user_agent` regression.

## \[0.46.2]

- [`1189e2a`](https://github.com/tauri-apps/wry/commit/1189e2a2d5ee18ad83281eea92b05a93f9184ebf) ([#1392](https://github.com/tauri-apps/wry/pull/1392) by [@chrox](https://github.com/tauri-apps/wry/../../chrox)) Fix malformed headers in custom protocol response on macOS.

## \[0.46.1]

### bug

- [`33c0193`](https://github.com/tauri-apps/wry/commit/33c01931a08b2178bb37b03b762d82f20f10aa3b) ([#1389](https://github.com/tauri-apps/wry/pull/1389) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Fix crash on macOS versions below 14.
- [`a57719e`](https://github.com/tauri-apps/wry/commit/a57719e6d63c8dbb002905e54c19d9d2f82c12de) ([#1385](https://github.com/tauri-apps/wry/pull/1385) by [@huacnlee](https://github.com/tauri-apps/wry/../../huacnlee)) Add `WebView::focus_parent` method.

## \[0.46.0]

- [`8cc2a7f`](https://github.com/tauri-apps/wry/commit/8cc2a7f6570085da5d6a5ea57ad7f127520b778f) ([#1384](https://github.com/tauri-apps/wry/pull/1384) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) This release contains quite the breaking changes, because even though wry@0.44, ignored duplicate custom protocols, On Linux when using a shared web context, the custom protocol handler can only be registered once so we are bringing the duplicate custom protocols on Linux again, Windows and macOS are not affected. If using a shared web context, make sure to register a protocol only once on Linux (other platforms should be registed multiple times), use `WebContext::is_custom_protocol_registered` with `#[cfg(target_os = "linux")]`.

  We also noticed that it is hard to know which webview made a request to the custom protocol so we added a method to attach an ID to a webview, and changed relevant custom protocol APIs to take a new argument that passes the specified id back to protocol handler.

  We also made a few changes to the builder, specifically `WebViewBuilder::new` and `WebViewBuilder::build` methods to make them more ergonomic to work with.

  - Added `Error::DuplicateCustomProtocol` enum variant.
  - Added `Error::ContextDuplicateCustomProtocol` enum variant.
  - On Linux, return an error in `WebViewBuilder::build` if registering a custom protocol multiple times.
  - Added `WebContext::is_custom_protocol_registered` to check if a protocol has been regsterd for this web context.
  - Added `WebViewId` alias type.
  - **Breaking** Changed `WebViewAttributes` to have a lifetime parameter.
  - Added `WebViewAttributes.id` field to specify an id for the webview.
  - Added `WebViewBuilder::with_id` method to specify an id for the webview.
  - Added `WebViewAttributes.context` field to specify a shared context for the webview.
  - **Breaking** Changed `WebViewAttributes.custom_protocols` field,`WebViewBuilder::with_custom_protocol` method and `WebViewBuilder::with_asynchronous_custom_protocol` method handler function to take `WebViewId` as the first argument to check which webview made the request to the protocol.
  - **Breaking** Changed `WebViewBuilder::with_web_context` to be a static method to create a builder with a webcontext, instead of it being a setter method. It is now an alternative to `WebviewBuilder::new`
  - Added `WebViewBuilder::with_attributes` to create a webview builder with provided attributes.
  - **Breaking** Changed `WebViewBuilder::new` to take no arguments.
  - **Breaking** Changed `WebViewBuilder::build` method to take a reference to a window to create the webview in it.
  - **Breaking** Removed `WebViewBuilder::new_as_child`.
  - Added `WebViewBuilder::build_as_child` method, which takes a reference to a window to create the webview in it.
  - **Breaking** Removed `WebViewBuilderExtUnix::new_gtk`.
  - Added `WebViewBuilderExtUnix::build_gtk`.
- [`0abc221`](https://github.com/tauri-apps/wry/commit/0abc221ca0edf3482518a68af024f9988b10f50b) ([#1316](https://github.com/tauri-apps/wry/pull/1316) by [@pewsheen](https://github.com/tauri-apps/wry/../../pewsheen)) Migrate to obj2.
- [`b01eac3`](https://github.com/tauri-apps/wry/commit/b01eac35f521e4e763fc15753e2fbee68aa31a02) ([#1386](https://github.com/tauri-apps/wry/pull/1386) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Use unescaped Android package identifier for the proguard rules.

## \[0.45.0]

- [`0fd1229`](https://github.com/tauri-apps/wry/commit/0fd12297997f598e4893e8f5b6e235b09cedec09) ([#1369](https://github.com/tauri-apps/wry/pull/1369) by [@lloydzhou](https://github.com/tauri-apps/wry/../../lloydzhou)) On Linux, fixed incorrect path for indexeddb database directory which made apps using `wry@0.24` and `tauri@1` migrating to `wry@>=0.38` and `tauri@2` lose their indexeddb data.
- [`e332eff`](https://github.com/tauri-apps/wry/commit/e332eff6ac41ff0f4ed6cf5196c3a78c776912d3) ([#1368](https://github.com/tauri-apps/wry/pull/1368) by [@zephraph](https://github.com/tauri-apps/wry/../../zephraph)) Add `Webview::load_html`.

## \[0.44.1]

- [`5111eb0`](https://github.com/tauri-apps/wry/commit/5111eb013e1b049d12aad38b96b2017a4fc54c72) ([#1362](https://github.com/tauri-apps/wry/pull/1362) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes `WebView::clear_all_browsing_data` crashing with a segfault on macOS.

## \[0.44.0]

- [`b863d38`](https://github.com/tauri-apps/wry/commit/b863d38ff705e037b297c1651e17775d3dbe473c) ([#1356](https://github.com/tauri-apps/wry/pull/1356) by [@SpikeHD](https://github.com/tauri-apps/wry/../../SpikeHD)) Expose ability to enable browser extensions in WebView2
- [`9220793`](https://github.com/tauri-apps/wry/commit/92207933c9de4a0c1ef65ecbacb0c303619ced4c) ([#1361](https://github.com/tauri-apps/wry/pull/1361) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Ignore duplicate custom protocols in `WebviewBuilder::with_custom_protocol` and `WebviewBuilder::with_async_custom_protocol` and use the last registered one.
- [`9220793`](https://github.com/tauri-apps/wry/commit/92207933c9de4a0c1ef65ecbacb0c303619ced4c) ([#1361](https://github.com/tauri-apps/wry/pull/1361) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Removed `Error::DuplicateCustomProtocol` variant.
- [`5915341`](https://github.com/tauri-apps/wry/commit/59153413d87549964b885ad3ed84473acf4b70df) ([#1354](https://github.com/tauri-apps/wry/pull/1354) by [@millermk](https://github.com/tauri-apps/wry/../../millermk)) Fixes Android webview error page flashing when a redirect to the app is performed.
- [`170095b`](https://github.com/tauri-apps/wry/commit/170095b9a5829ac9279cfeea3fac0da2c922d448) ([#1360](https://github.com/tauri-apps/wry/pull/1360) by [@Steve-xmh](https://github.com/tauri-apps/wry/../../Steve-xmh)) Fix web resource loading in android binding by skip duplicate Content-Type/Content-Length headers.
- [`5915341`](https://github.com/tauri-apps/wry/commit/59153413d87549964b885ad3ed84473acf4b70df) ([#1354](https://github.com/tauri-apps/wry/pull/1354) by [@millermk](https://github.com/tauri-apps/wry/../../millermk)) Fix navigation error handling to trigger custom protocol on Android.

## \[0.43.1]

- [`5cea504`](https://github.com/tauri-apps/wry/commit/5cea5045c9e646ace1f94767d91b746c2afd3c14) ([#1352](https://github.com/tauri-apps/wry/pull/1352) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes Android file picker result processing.

## \[0.43.0]

- [`7b1c26a`](https://github.com/tauri-apps/wry/commit/7b1c26adff5fdf13a3364b6fc26c8444c120d5c1) ([#1344](https://github.com/tauri-apps/wry/pull/1344) by [@Themayu](https://github.com/tauri-apps/wry/../../Themayu)) Windows: Implement `WebViewBuilderExtWindows::with_scroll_bar_style` to allow opting into Fluent Overlay style scrollbars.
- [`98d1a83`](https://github.com/tauri-apps/wry/commit/98d1a835e2c818e9f733a55b074e7de3c0cd725f) ([#1326](https://github.com/tauri-apps/wry/pull/1326) by [@ollpu](https://github.com/tauri-apps/wry/../../ollpu)) Fix Linux IPC handler and initialization scripts when sharing a WebContext between multiple WebViews.

## \[0.42.0]

- [`556a359`](https://github.com/tauri-apps/wry/commit/556a359d3739ac3a3e671ab2d1c3dda2784c511e) ([#1301](https://github.com/tauri-apps/wry/pull/1301) by [@pewsheen](https://github.com/tauri-apps/wry/../../pewsheen)) On macOS, emit an error when the URL scheme registration fails.
- [`19d83d7`](https://github.com/tauri-apps/wry/commit/19d83d7d01ef9704197c09e6197a6da31a4eec6c) ([#1332](https://github.com/tauri-apps/wry/pull/1332) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes a crash on the Android custom protocol handler when the request URL is invalid.
- [`38abcb9`](https://github.com/tauri-apps/wry/commit/38abcb95090a3917ad5092f4933acb95c721f893) ([#1340](https://github.com/tauri-apps/wry/pull/1340) by [@lucasfernog](https://github.com/tauri-apps/wry/../../lucasfernog)) Fixes custom protocols not triggered on Android on external redirects.
- [`d1f1e7e`](https://github.com/tauri-apps/wry/commit/d1f1e7e6fa82722d9848fbb540e6f9a3d5925cdd) ([#1299](https://github.com/tauri-apps/wry/pull/1299) by [@kanatapple](https://github.com/tauri-apps/wry/../../kanatapple)) Fix `Webview::bounds` returning logical values where it should have been physical.
- [`03cdf93`](https://github.com/tauri-apps/wry/commit/03cdf93f1e39469a1cd73565ba61138c58567fb6) ([#1311](https://github.com/tauri-apps/wry/pull/1311) by [@bukowa](https://github.com/tauri-apps/wry/../../bukowa)) Handle `webkit2gtk` close signal (when `window.close` is called from js)
- [`68413e8`](https://github.com/tauri-apps/wry/commit/68413e837f81a88e8de0df39d12ba2b405b89e5e) ([#1296](https://github.com/tauri-apps/wry/pull/1296) by [@MarijnS95](https://github.com/tauri-apps/wry/../../MarijnS95)) **Breaking change**: Upgrade `ndk` crate to `0.9` and delete unused `ndk-sys` and `ndk-context` dependencies.  Types from the `ndk` crate are used in public API surface.
  **Breaking change**: The public `android_setup()` function now takes `&ThreadLooper` instead of `&ForeignLooper`, signifying that the setup function must be called on the thread where the looper is attached (and the `JNIEnv` argument is already thread-local as well).
- [`39fc82c`](https://github.com/tauri-apps/wry/commit/39fc82c9276bd039a9130919645db149f067719a) ([#1306](https://github.com/tauri-apps/wry/pull/1306) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Support WebView2 version older than 101.0.1210.39 and document `incognito` and `theme` will not work for versions before it
- [`5231a37`](https://github.com/tauri-apps/wry/commit/5231a379d09c9eea27f490b3062129dd474cb44c) ([#1322](https://github.com/tauri-apps/wry/pull/1322) by [@Legend-Master](https://github.com/tauri-apps/wry/../../Legend-Master)) Support WebView2 version older than 86.0.616.0 and document version requirements for `back_forward_navigation_gestures`, `with_user_agent`, `with_hotkeys_zoom`, `with_browser_accelerator_keys`
- [`a23a28d`](https://github.com/tauri-apps/wry/commit/a23a28d32a01a0a4c8e62e83d34813ec7db4a004) ([#1341](https://github.com/tauri-apps/wry/pull/1341) by [@amrbashir](https://github.com/tauri-apps/wry/../../amrbashir)) Updated `windows` to 0.58.

## \[0.41.0]

- [`8b691df`](https://github.com/tauri-apps/wry/commit/8b691df1ac57eb5eb15082c5f6d72e871965c61e) ([#1285](https://github.com/tauri-apps/wry/pull/1285) by [@pewsheen](https://github.com/tauri-apps/wry/../../pewsheen)) On macOS, fix an issue that could cause a panic when running an async command.
- [`6c7f45e`](https://github.com/tauri-apps/wry/commit/6c7f45e1e3f89805a9fd6b58a9382a6bbc2b0c28) ([#1287](https://github.com/tauri-apps/wry/pull/1287) by [@FabianLars](https://github.com/tauri-apps/wry/../../FabianLars)) Fixed a regression causing autoplay on windows to require user gestures.
- [`24a7d27`](https://github.com/tauri-apps/wry/commit/24a7d275c9059dd9e54544b75a5996c9d0762f26) ([#1289](https://github.com/tauri-apps/wry/pull/1289) by [@renovate](https://github.com/tauri-apps/wry/../../renovate)) Update `windows` crate to `0.57` and `webview2-com` crate to `0.31`

## \[0.40.1]

- [`b6863ed`](https://github.com/tauri-apps/wry/commit/b6863ed1884fb190ae46f37ed72dcdd92de700cd)([#1275](https://github.com/tauri-apps/wry/pull/1275)) On Android, set `RustWebViewClient.currentUrl` field early in `onPageStarted` method instead of `onPageFinished`
- [`f089964`](https://github.com/tauri-apps/wry/commit/f089964a3cf3014987aca24a7e7d6cae83e67d8a)([#1276](https://github.com/tauri-apps/wry/pull/1276)) Fixes `with_asynchronous_custom_protocol` crashing when sending the response on Linux.
- [`637289d`](https://github.com/tauri-apps/wry/commit/637289dfb36150635177eb629a12b40fdaac1afe)([#1272](https://github.com/tauri-apps/wry/pull/1272)) On Android, make `WryActivity.setWebview` method public to prevent JNI crashes.

## \[0.40.0]

- [`a424a0b`](https://github.com/tauri-apps/wry/commit/a424a0b234cb20b3ca7305d87e82aba3c8b2bd41)([#1270](https://github.com/tauri-apps/wry/pull/1270)) On Windows, fix child webview invisible after creation because it was created with `0,0` size
- [`d6f8dd7`](https://github.com/tauri-apps/wry/commit/d6f8dd7b6c0485fbb96fed34717969540eef2b96)([#1271](https://github.com/tauri-apps/wry/pull/1271)) On Windows, create child webview at the top of z-order to align with other platforms.
- [`03d2535`](https://github.com/tauri-apps/wry/commit/03d25357d2c20a21640871cfca9d5f6a39c7afc8)([#1269](https://github.com/tauri-apps/wry/pull/1269)) On macOS, disable initialization script injection into subframes.
- [`1e65049`](https://github.com/tauri-apps/wry/commit/1e65049d4842947ced6a807b93211542c46ca771)([#1267](https://github.com/tauri-apps/wry/pull/1267)) On macOS, fixed a crash when sending empty body by IPC.
- [`0f3c886`](https://github.com/tauri-apps/wry/commit/0f3c886a224a1b52980ef90667860e58a6ad669a)([#1260](https://github.com/tauri-apps/wry/pull/1260)) On macOS, fixed an issue of not being able to listen to the cmd+key event in javascript in single WebView.
- [`0f14e2a`](https://github.com/tauri-apps/wry/commit/0f14e2a540a1d54f82bdee2a3c2f93c43c593959)([#1259](https://github.com/tauri-apps/wry/pull/1259)) Default the margin when printing on MacOS to 0 so it is closer to the behavior of when printing on the web.
- [`0f14e2a`](https://github.com/tauri-apps/wry/commit/0f14e2a540a1d54f82bdee2a3c2f93c43c593959)([#1259](https://github.com/tauri-apps/wry/pull/1259)) Add `WebViewExtMacOS::print_with_options` which allows to modify the margins that will be used on the print dialog.
- [`f516122`](https://github.com/tauri-apps/wry/commit/f5161225940c545dd457af1178c73f36dfe63710)([#1262](https://github.com/tauri-apps/wry/pull/1262)) On Windows, enable webview2 [non client region support](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings9?view=webview2-1.0.2478.35#get_isnonclientregionsupportenabled) which allows using `app-region` CSS style.

## \[0.39.5]

- [`4c88c66`](https://github.com/tauri-apps/wry/commit/4c88c66fb79fc3742f4592252c260e7e012d5fcf)([#1247](https://github.com/tauri-apps/wry/pull/1247)) Force the IPC and custom protocol tracing spans to have no parent.
- [`2d43d62`](https://github.com/tauri-apps/wry/commit/2d43d62a8e61514ade27ae63fa33c1dee2de6744)([#1254](https://github.com/tauri-apps/wry/pull/1254)) On Windows, fix webview having a bigger size than the actual window size after creation and until the window is resized.

## \[0.39.4]

- [`8bbc2bf`](https://github.com/tauri-apps/wry/commit/8bbc2bf388113af2e7d91250abe1569070b351a9)([#1237](https://github.com/tauri-apps/wry/pull/1237)) Fix `WebviewBuilder::with_transparent`, `WebviewBuilder::with_background_color`, and `Webview::set_background_color` always failing and causing the webview to fail to load.
- [`130c469`](https://github.com/tauri-apps/wry/commit/130c46965d0cd0ae2389d2fa9b683488a16e0cc8)([#1238](https://github.com/tauri-apps/wry/pull/1238)) Add `WebViewBuilderExtDarwin::with_data_store_identifier`.
- [`203604c`](https://github.com/tauri-apps/wry/commit/203604c519e4acb169676b20ddf5956ba21b4d57)([#1233](https://github.com/tauri-apps/wry/pull/1233)) On Windows, fix the webview not filling up the whole window if the parent window was resized during the webview initialization.

## \[0.39.3]

- [`c7ca3db`](https://github.com/tauri-apps/wry/commit/c7ca3db581bbeb4f16a28f47c3a1fd59889c0978)([#1221](https://github.com/tauri-apps/wry/pull/1221)) On Windows, fix data directory created next to the executable with a gibberish name even if it was explicitly provided in `WebConext::new`

## \[0.39.2]

- [`3e3d59c`](https://github.com/tauri-apps/wry/commit/3e3d59cd4f79c21571e503a5bf80d4d54a654a38)([#1215](https://github.com/tauri-apps/wry/pull/1215)) On macOS, prevent NSExceptions and invalid memory access panics when dropping the WebView while custom protocols handlers may still be running.
- [`ca6b5fb`](https://github.com/tauri-apps/wry/commit/ca6b5fbef6e5a5efe43b5cbebe6bfc4bc13930d3)([#1224](https://github.com/tauri-apps/wry/pull/1224)) Update `windows` crate to `0.56`

## \[0.39.1]

- [`f0e82d3`](https://github.com/tauri-apps/wry/commit/f0e82d3aa2da9da2b935d97c9a9b5e2dbd65b6ea)([#1217](https://github.com/tauri-apps/wry/pull/1217)) Fix target detection on build script to enhance cross compiling capabilities.
- [`ed9fa9b`](https://github.com/tauri-apps/wry/commit/ed9fa9b3950206548cdaf0bcdb6c2d5fb72619b3)([#1210](https://github.com/tauri-apps/wry/pull/1210)) On iOS, allows media plays inline.

## \[0.39.0]

- [`ddda455`](https://github.com/tauri-apps/wry/commit/ddda4556b36a41b1c6f3f4d200eb16612d5f3f12)([#1207](https://github.com/tauri-apps/wry/pull/1207)) Disable deprecated applicationCache web api. This api was completely removed upstream in webkitgtk 2.44.
- [`d7031ae`](https://github.com/tauri-apps/wry/commit/d7031aed8eebc6324e4b3db46ee53120ce24930b)([#1206](https://github.com/tauri-apps/wry/pull/1206)) On Windows, fix a crash due to a double-free when the host window is destroyed before the webview is dropped.
- [`34ae1ca`](https://github.com/tauri-apps/wry/commit/34ae1ca3af75c471f77b90fd342bbcc79ac7189a)([#1202](https://github.com/tauri-apps/wry/pull/1202)) Add `dpi` module which is a re-export of `dpi` crate.
- [`fdbd3d3`](https://github.com/tauri-apps/wry/commit/fdbd3d3c614acd42dddb49583d16de6b3f02e62d)([#1081](https://github.com/tauri-apps/wry/pull/1081)) Update `http` dependency to `1`
- [`34ae1ca`](https://github.com/tauri-apps/wry/commit/34ae1ca3af75c471f77b90fd342bbcc79ac7189a)([#1202](https://github.com/tauri-apps/wry/pull/1202)) **Breaking Change**: Removed `x`, `y`, `with` and `height` fields from `Rect` struct and replaced it with `size` and `position` fields.
- [`c033bd2`](https://github.com/tauri-apps/wry/commit/c033bd27f23953537520d17493c7b77ea146e7d5)([#1156](https://github.com/tauri-apps/wry/pull/1156)) On `macOS`, fix menu keyboard shortcuts when added `webview` as `child`.

## \[0.38.2]

- [`3e84a0e`](https://github.com/tauri-apps/wry/commit/3e84a0e276dfac0b28fb01f42460f9367fff9f22)([#1200](https://github.com/tauri-apps/wry/pull/1200)) Fixes compilation for 32bit Linux targets.

## \[0.38.1]

- [`7c9e71f`](https://github.com/tauri-apps/wry/commit/7c9e71f4692e94fd401ad3508ff3912d43880e2c)([#1192](https://github.com/tauri-apps/wry/pull/1192)) Fixes compilation failing on Windows with the `tracing` feature enabled.

## \[0.38.0]

- [`e6f0fbd`](https://github.com/tauri-apps/wry/commit/e6f0fbd33365070af46361605a922ba24e542fb5)([#1180](https://github.com/tauri-apps/wry/pull/1180)) Fixes a null pointer exception when running `window.ipc.postMessage(null)` on Android.
- [`5789bf7`](https://github.com/tauri-apps/wry/commit/5789bf759ce94e4dad5ff26a08fe81521658a4e4)([#1187](https://github.com/tauri-apps/wry/pull/1187)) **Breaking change**: Refactored the file-drop handling on the webview for better representation of the actual drag and drop operation:

  - Renamed `file-drop` cargo feature flag to `drag-drop`.
  - Removed `FileDropEvent` enum and replaced with a new `DragDropEvent` enum.
  - Renamed `WebViewAttributes::file_drop_handler` field to `WebViewAttributes::drag_drop_handler`.
  - Renamed `WebViewAttributes::with_file_drop_handler` method to `WebViewAttributes::with_drag_drop_handler`.
- [`b8fea39`](https://github.com/tauri-apps/wry/commit/b8fea396c2eca289e2f930ad635a15397b7c0dda)([#1183](https://github.com/tauri-apps/wry/pull/1183)) Changed `WebViewBuilder::with_ipc_handler` closure to take `http::Request` instead of `String` so the request URL is available.
- [`3a2026b`](https://github.com/tauri-apps/wry/commit/3a2026b37be67dea53535f0a7d78b32452ac8b40)([#1182](https://github.com/tauri-apps/wry/pull/1182)) **Breaking changes**: Changed a few methods on `WebView` type to return a `Result`:

  - `Webview::url`
  - `Webview::zoom`
  - `Webview::load_url`
  - `Webview::load_url_with_headers`
  - `Webview::bounds`
  - `Webview::set_bounds`
  - `Webview::set_visible`
  - `WebviewExtWindows::set_theme`
  - `WebviewExtWindows::set_memory_usage_level`
  - `WebviewExtWindows::reparent`
  - `WebviewExtUnix::reparent`
  - `WebviewExtMacOS::reparent`
- [`e1e2e07`](https://github.com/tauri-apps/wry/commit/e1e2e071e5329bc1a94864e368fdaa3041e79427)([#1190](https://github.com/tauri-apps/wry/pull/1190)) Update `webview2-com` crate to `0.29`
- [`e1e2e07`](https://github.com/tauri-apps/wry/commit/e1e2e071e5329bc1a94864e368fdaa3041e79427)([#1190](https://github.com/tauri-apps/wry/pull/1190)) Update `windows` crate to `0.54`
- [`00bc96d`](https://github.com/tauri-apps/wry/commit/00bc96d115879c841fc47242271db3761d19f746)([#1179](https://github.com/tauri-apps/wry/pull/1179)) Added `WryActivity::onWebViewCreate(android.webkit.WebView)` on Android.

## \[0.37.0]

- [`8c86fba`](https://github.com/tauri-apps/wry/commit/8c86fbaf51cd970737cc070583318d4b532349d2) **Breaking change**: Removed `data:` url support, as its native support in Windows and macOS are buggy and unreliable, use `Webview::with_html` instead.
- [`8c86fba`](https://github.com/tauri-apps/wry/commit/8c86fbaf51cd970737cc070583318d4b532349d2) On Linux, decode `FilDropEvent` paths before emitting them to make it consistent across all platforms.
- [`8c86fba`](https://github.com/tauri-apps/wry/commit/8c86fbaf51cd970737cc070583318d4b532349d2) Added `WebViewExtMacOS::reparent`,`WebViewExtWindows::reparent` and `WebViewExtUnix::reparent`.
- [`8c86fba`](https://github.com/tauri-apps/wry/commit/8c86fbaf51cd970737cc070583318d4b532349d2) Revert global keys shortcuts (wry#1156)
- [`8c86fba`](https://github.com/tauri-apps/wry/commit/8c86fbaf51cd970737cc070583318d4b532349d2) **Breaking change**: Removed internal url parsing which had a few side-effects such as encoded url content, now it is up to the user to pass a valid URL as a string. This also came with a few breaking changes:

  - Removed `Url` struct re-export
  - Removed `Error::UrlError` variant.
  - Changed `WebviewAttributes::url` field type to `String`.
  - Changed `WebviewBuilder::with_url` and `WebviewBuilder::with_url_and_headers` return type to `WebviewBuilder` instead of `Result<WebviewBuilder>`.
  - Changed `Webview::url` getter to return a `String` instead of `Url`.

## \[0.36.0]

- [`8646120`](https://github.com/tauri-apps/wry/commit/8646120339b8ed983582caa9e668fc286dc59cb3)([#1159](https://github.com/tauri-apps/wry/pull/1159)) On android, fix `no non-static method ".evalScript(ILjava/lang/String;)"` when calling `Window::eval`.
- [`8646120`](https://github.com/tauri-apps/wry/commit/8646120339b8ed983582caa9e668fc286dc59cb3)([#1159](https://github.com/tauri-apps/wry/pull/1159)) On macOS, fix a release build crashes with SEGV when calling `WebView::evaluate_script`. This crash bug was introduced at v0.35.2.
- [`8646120`](https://github.com/tauri-apps/wry/commit/8646120339b8ed983582caa9e668fc286dc59cb3)([#1159](https://github.com/tauri-apps/wry/pull/1159)) **Breaking change** Update [raw-window-handle](https://crates.io/crates/raw-window-handle) crate to v0.6.

  - `HasWindowHandle` trait is required for window types instead of `HasRawWindowHandle`.
  - `wry::raw_window_handle` now re-exports v0.6.
- [`8646120`](https://github.com/tauri-apps/wry/commit/8646120339b8ed983582caa9e668fc286dc59cb3)([#1159](https://github.com/tauri-apps/wry/pull/1159)) On `macOS`, fix menu keyboard shortcuts. This issue bug was introduced in `v2` when added `webview` as `child`.

## \[0.35.2]

- [`0ef041f`](https://github.com/tauri-apps/wry/commit/0ef041ffece143dcb5059ad43596c63b18a62928)([#1133](https://github.com/tauri-apps/wry/pull/1133)) On Linux, apply passed webview bounds when using `WebView::new_gtk` or `WebViewBuilder::new_gtk` with `gtk::Fixed` widget. This allows to create multiple webviews inside `gtk::Fixed` in the same window.
- [`0ef041f`](https://github.com/tauri-apps/wry/commit/0ef041ffece143dcb5059ad43596c63b18a62928)([#1133](https://github.com/tauri-apps/wry/pull/1133)) Added tracing spans for `evaluate_script`, `ipc_handler` and `custom_protocols` behind the `tracing` feature flag.

## \[0.35.1]

- [`a4a39b9`](https://github.com/tauri-apps/wry/commit/a4a39b9b23da3c562f27730dd0eab09b9459755b)([#1098](https://github.com/tauri-apps/wry/pull/1098)) Fix the API documentation cannot be built on docs.rs.
- [`e116d42`](https://github.com/tauri-apps/wry/commit/e116d427319d1adbc14d418e78c43ddb49b70d76)([#1111](https://github.com/tauri-apps/wry/pull/1111)) Fix screen share permissions dialog not showing up on macOS 14.0+
- [`a8c0d38`](https://github.com/tauri-apps/wry/commit/a8c0d384fc51b12d2436c11d10fd8c2dfdcd9d4a)([#1097](https://github.com/tauri-apps/wry/pull/1097)) Fix IPC crash on wkwebview if receiving invalid types.
- [`8fddbb6`](https://github.com/tauri-apps/wry/commit/8fddbb6d514de8fa0561bd6631ff8a3699911ddd)([#1091](https://github.com/tauri-apps/wry/pull/1091)) Add `WebView::bounds` getter.
- [`30a85f3`](https://github.com/tauri-apps/wry/commit/30a85f31141839a5284b1bfdd52b1cb690fcd10d)([#1122](https://github.com/tauri-apps/wry/pull/1122)) On Windows, fix file drop handler.

## \[0.35.0]

- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) **Breaking change** Consistently use `WebView` in API names. The following APIs were renamed:

  - `WebviewExtWindows` → `WebViewExtWindows`
  - `WebviewExtUnix` → `WebViewExtUnix`
  - `WebviewExtMacOS` → `WebViewExtMacOS`
  - `WebviewExtIOS` → `WebViewExtIOS`
  - `WebviewExtAndroid` → `WebViewExtAndroid`
  - `WebviewUriLoader` → `WebViewUriLoader`
- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) Add `WebViewExtWindows::set_memory_usage_level` API to set the [memory usage target level](https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2memoryusagetargetlevel) on Windows. Setting 'Low' memory usage target level when an application is going to inactive can significantly reduce the memory consumption. Please read the [guide for WebView2](https://github.com/MicrosoftEdge/WebView2Feedback/blob/main/specs/MemoryUsageTargetLevel.md) for more details.
- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) -   Add cfg_aliases for easier feature configuration. And add `os-webview` as default feature.
- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) Enhance initalization script implementation on Android supporting any kind of URL.
- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) Fix wkwebview crashed when received invalid UTF8 string from IPC.
- [`e61e7f8`](https://github.com/tauri-apps/wry/commit/e61e7f8474c18752f5c60d3f1f5ba33b27e41d52)([#1090](https://github.com/tauri-apps/wry/pull/1090)) Refactor new method to take raw window handle instead. Following are APIs got affected:

  - `application` module is removed, and `webivew` module is moved to root module.
  - `WebViewBuilder::new`, `WebView::new` now take `RawWindowHandle` instead.
  - Add `WebViewBuilder::new_as_child`, `WebView::new_as_child` to crate a webview as a child inside a parent window.
  - `Webview::inner_size` is removed.
  - Add `WebViewBuilderExtUnix` trait to extend `WebViewBuilder` on Unix platforms.
  - Add `new_gtk` functions to `WebViewBuilderExtUnix` and `WebviewExtUnix`.
  - [raw-window-handle](https://docs.rs/raw-window-handle/latest/raw_window_handle/) crate is re-exported as `wry::raw_window_handle`.

  This also means that we removed `tao` as a dependency completely which required some changes to the public APIs and to the Android backend:

  - Webview attributes `ipc_handler`, `file_drop_handler`, `document_change_handler` don't take the `Window` as first parameter anymore.
    Users should use closure to capture the types they want to use.
  - Position field in `FileDrop` event is now a tuple of `(x, y)` physical position instead of `PhysicalPosition`. Users need to handle scale factor
  - We exposed the `android_setup` function that needs to be called once to setup necessary logic.
  - Previously the `android_binding!` had internal call to `tao::android_binding` but now that `tao` has been removed,
    the macro signature has changed and you now need to call `tao::android_binding` yourself, checkout the crate documentation for more information.

## \[0.34.2]

- [`c2e6980`](https://github.com/tauri-apps/wry/commit/c2e6980b6cacf02b3f8c0b0285d391d010f4536b)([#1047](https://github.com/tauri-apps/wry/pull/1047)) Fix doc building by removing dox feature requirement from `webkit2gtk`.
- [`82908d4`](https://github.com/tauri-apps/wry/commit/82908d4e001d1be6fd5d692fcb2e08908c4b5e16)([#1045](https://github.com/tauri-apps/wry/pull/1045)) Fix docs.rs build.

## \[0.34.1]

- [`3de68e7`](https://github.com/tauri-apps/wry/commit/3de68e781d52f3c817473c1ee8cc73b392d60c98)([#1043](https://github.com/tauri-apps/wry/pull/1043)) Fix compilation with the `linux-body` feature.

## \[0.34.0]

- [`ce95730`](https://github.com/tauri-apps/wry/commit/ce957301566dfe33f576810982a3eb38813d22ea)([#1036](https://github.com/tauri-apps/wry/pull/1036)) Upgrade gtk to 0.18 and bump MSRV to 1.70.0.
- [`591fda8`](https://github.com/tauri-apps/wry/commit/591fda8045b88ea0edbc5676e2814fb9acb2d6f6)([#1042](https://github.com/tauri-apps/wry/pull/1042)) Use `gtk`'s re-exported modules instead.
- [`b22a19e`](https://github.com/tauri-apps/wry/commit/b22a19e1c19ce90aec6521a66587dac9b0351579)([#1037](https://github.com/tauri-apps/wry/pull/1037)) Update `windows` and `windows-implement` crate to `0.51`

## \[0.33.1]

- [`0582cdf`](https://github.com/tauri-apps/wry/commit/0582cdf4a195db5df9c4e21d24039c64b7474683)([#1033](https://github.com/tauri-apps/wry/pull/1033)) Fix documentation for macOS target not being generated on docs.rs.

## \[0.33.0]

- [`5adf9da`](https://github.com/tauri-apps/wry/commit/5adf9da2151800ec2431a1547cc0d970fc95b764)([#994](https://github.com/tauri-apps/wry/pull/994)) **Breaking change** Wry now defaults to `http://<scheme>.localhost/` for custom protocols on Android.
- [`844d95a`](https://github.com/tauri-apps/wry/commit/844d95a4035f68371d64f6b04151982481cdee70)([#1023](https://github.com/tauri-apps/wry/pull/1023)) Fixes async custom protocol resolver on Windows.
- [`5adf9da`](https://github.com/tauri-apps/wry/commit/5adf9da2151800ec2431a1547cc0d970fc95b764)([#994](https://github.com/tauri-apps/wry/pull/994)) Add `WebViewBuilderExtAndroid::with_https_scheme` to be able to choose between `http` and `https` for custom protocols on Android.
- [`c5c3731`](https://github.com/tauri-apps/wry/commit/c5c3731f2027802735f7b80c7ae5f4b64d0fb746)([#1024](https://github.com/tauri-apps/wry/pull/1024)) Add winit-gtk to support winit feature flag on Linux.

## \[0.32.0]

- [`4bdf1c3`](https://github.com/tauri-apps/wry/commit/4bdf1c366de5708b7626ca63eb39e134869c5bd4)([#1017](https://github.com/tauri-apps/wry/pull/1017)) Added `WebViewBuilder::with_asynchronous_custom_protocol` to allow implementing a protocol handler that resolves asynchronously.
- [`70d8ae0`](https://github.com/tauri-apps/wry/commit/70d8ae057c5e8b81db4aac28e5fa2dd3424b3307)([#1009](https://github.com/tauri-apps/wry/pull/1009)) Fixes Android freezing when handling request due to endless iteration when reading request headers.
- [`b5e1875`](https://github.com/tauri-apps/wry/commit/b5e1875230794502a8e74c74abe79ca63488e421)([#994](https://github.com/tauri-apps/wry/pull/994)) **Breaking change** Wry now defaults to `http://<scheme>.localhost/` for custom protocols on Windows.
- [`b5e1875`](https://github.com/tauri-apps/wry/commit/b5e1875230794502a8e74c74abe79ca63488e421)([#994](https://github.com/tauri-apps/wry/pull/994)) Add `WebViewBuilderExtWindows::with_https_scheme` to be able to choose between `http` and `https` for custom protocols on Windows.
- [`fa15076`](https://github.com/tauri-apps/wry/commit/fa15076207d9e678db4149210aba929044d0ff45)([#163](https://github.com/tauri-apps/wry/pull/163)) Add `winit` and `tao` feature flag with `tao` as default.
- [`4bdf1c3`](https://github.com/tauri-apps/wry/commit/4bdf1c366de5708b7626ca63eb39e134869c5bd4)([#1017](https://github.com/tauri-apps/wry/pull/1017)) **Breaking change:** `WebViewBuidler::with_custom_protocol` closure now returns `http::Response` instead of `Result<http::Response>`.
- [`ebc4a20`](https://github.com/tauri-apps/wry/commit/ebc4a20d218036b29b186aca1853d28d870fa2ef)([#1015](https://github.com/tauri-apps/wry/pull/1015)) Add `WebViewAtrributes.focused` and `WebViewBuilder::with_focused` to control whether to focus the webview upon creation or not. Supported on Windows and Linux only.

## \[0.31.0]

- [`e47562f`](https://github.com/tauri-apps/wry/commit/e47562f71284457ff77e4c8b6bf02fdbe19ab880)([#993](https://github.com/tauri-apps/wry/pull/993)) Update the unmaintained `kuchiki` crate to the maintained `kuchikiki` crate.
- [`7a353c7`](https://github.com/tauri-apps/wry/commit/7a353c7d8a474bfb14b92a272efc75ceb194ea90)([#980](https://github.com/tauri-apps/wry/pull/980)) Add `WebViewBuilder::with_on_page_load_handler` for providing a callback for handling various page loading events.
- [`b0a08b1`](https://github.com/tauri-apps/wry/commit/b0a08b165215823ed7a48a0a377e0f09832898df)([#997](https://github.com/tauri-apps/wry/pull/997)) Update `tao` to version `0.22` which has removed the global-shortcut, menus and tray features, see [tao@v0.22 release](https://github.com/tauri-apps/tao/releases/tag/tao-v0.22.0).

## \[0.30.0]

- [`17e04e2`](https://github.com/tauri-apps/wry/commit/17e04e2b4c0bd75f93bbc511234f0d3c93726b63)([#985](https://github.com/tauri-apps/wry/pull/985)) Make `WebViewBuilder::with_navigation_handler` apply to Android `loadUrl` calls.
- [`17e04e2`](https://github.com/tauri-apps/wry/commit/17e04e2b4c0bd75f93bbc511234f0d3c93726b63)([#985](https://github.com/tauri-apps/wry/pull/985)) Add support for `WebViewBuilder::with_navigation_handler` on Android.
- [`87b331a`](https://github.com/tauri-apps/wry/commit/87b331a7d4c169814d2b6a1f8a06d976ad7565bc)([#978](https://github.com/tauri-apps/wry/pull/978)) On Windows, avoid resizing the webview when the window gets minimized to avoid unnecessary `resize` event on JS side.
- [`17e04e2`](https://github.com/tauri-apps/wry/commit/17e04e2b4c0bd75f93bbc511234f0d3c93726b63)([#985](https://github.com/tauri-apps/wry/pull/985)) Update tao to 0.21.

## \[0.29.0]

- [`c09dd7b`](https://github.com/tauri-apps/wry/commit/c09dd7bebe3d00f989dff57f0414f1023653efe4)([#968](https://github.com/tauri-apps/wry/pull/968)) Remove ActionBar handling from wry. If you want to hide the action bar, hide it using the `themes.xml` file in your android project or inherit `WryActivity` class and use `getSupportActionBar()?.hide()` in the `onCreate` method.
- [`2b56bfa`](https://github.com/tauri-apps/wry/commit/2b56bfaaee5125f0dc48f4a9bedb53db0e679e5f)([#966](https://github.com/tauri-apps/wry/pull/966)) Add support for `WebViewBuilder::with_html` and `WebViewAttributes.html` on Android.
- [`d2c1819`](https://github.com/tauri-apps/wry/commit/d2c1819f81a7b03288348f1c3b195407400dfbde)([#969](https://github.com/tauri-apps/wry/pull/969)) On Linux, replace `linux-header` flag with `linux-body` flag. Request headers are enabled by default. Add request body on custom protocol but it's behind the flag.
- [`f7dded4`](https://github.com/tauri-apps/wry/commit/f7dded417c239c39ca4cad6f9d3f6b319c3f91f2)([#955](https://github.com/tauri-apps/wry/pull/955)) The bug was reported in tauri repo: https://github.com/tauri-apps/tauri/issues/5986

  With input method preedit disabled,fcitx can anchor at edit cursor position.
  the pre-edit text will not disappear,instead it shows in the fcitx selection window below the input area.
- [`2b56bfa`](https://github.com/tauri-apps/wry/commit/2b56bfaaee5125f0dc48f4a9bedb53db0e679e5f)([#966](https://github.com/tauri-apps/wry/pull/966)) Set base url and origin to null for `WebViewBuilder::with_html` and `WebViewAttributes.html` for consistency on all platforms.

## \[0.28.3]

- On iOS, fix panic at runtime due to setting webview ivar.
  - [c9002c1](https://github.com/tauri-apps/wry/commit/c9002c1e043e8a948fff2e671ccb04153a10dcd5) fix(macos): remove `webview` ivar in `WryWebView` ([#943](https://github.com/tauri-apps/wry/pull/943)) on 2023-04-26

## \[0.28.2]

- Adjust `cargo:rerun-if-changed` instruction for Android files.
  - [cc934fe](https://github.com/tauri-apps/wry/commit/cc934fe799836e4cc72d796f5eddba868a9b585e) refactor(build): adjust rerun-if-changed instruction for Android files ([#940](https://github.com/tauri-apps/wry/pull/940)) on 2023-04-24

## \[0.28.1]

- Fix unresolved reference in kotlin files when building for android.
  - [ed36c0b](https://github.com/tauri-apps/wry/commit/ed36c0b032cdf27c926577ee72658ad9f0785a5f) fix(android): fix unresolved reference in kotlin files ([#932](https://github.com/tauri-apps/wry/pull/932)) on 2023-04-19
- Support modifying user agent string on Android.
  - [4a320b0](https://github.com/tauri-apps/wry/commit/4a320b0bdef81d36a1f85a083c2abbabaf958521) feat(android): add support modifying user agent string ([#933](https://github.com/tauri-apps/wry/pull/933)) on 2023-04-20
- On Linux and macOS, add synthesized event for mouse backward and forward buttons.
  - [6ef820b](https://github.com/tauri-apps/wry/commit/6ef820b97dd505bacdc7d3f906112ffe0a6a1e60) feat: synthesize forward/backward mouse button on Linux and macOS ([#900](https://github.com/tauri-apps/wry/pull/900)) on 2023-04-18

## \[0.28.0]

- Add `Webview::clear_browsing_data` method.
  - [5f0c9e4](https://github.com/tauri-apps/wry/commit/5f0c9e4595baf5d60ec407b391f873ab52abf923) feat: add `Webview::clear_browsing_data` ([#915](https://github.com/tauri-apps/wry/pull/915)) on 2023-04-18
- On Android, generate a `proguard-wry.pro` file that could be used to keep the necessary symbols for wry when using minification.
  - [ced4c0b](https://github.com/tauri-apps/wry/commit/ced4c0b4459ceb0ff89d07b84d6396c60cfd75e5) feat: generate proguard rule file for android ([#927](https://github.com/tauri-apps/wry/pull/927)) on 2023-04-17
- Update `tao` to `0.19`
  - [d560981](https://github.com/tauri-apps/wry/commit/d56098113f9764e31f73aa84144ee84be8e2aead) refactor: rename `TauriActivity` to `WryActivity` ([#926](https://github.com/tauri-apps/wry/pull/926)) on 2023-04-17

## \[0.27.3]

- Adds a way to launch a WebView as incognito through a new API at WebViewBuilder named as `with_incognito`.
  - [8698836](https://github.com/tauri-apps/wry/commit/86988368a4e833b21089d119c934529ecfe306b7) feat: Add a way to launch WebViews as incognito `WebView::as_incognito`, closes [#908](https://github.com/tauri-apps/wry/pull/908) ([#916](https://github.com/tauri-apps/wry/pull/916)) on 2023-04-06
- On macOS and iOS, remove webcontext implementation since we don't actually use it. This also fix segfault if users drop webcontext early.
  - [3cc45cb](https://github.com/tauri-apps/wry/commit/3cc45cb86b93c56cf2444bfc37dc6ba229d4222e) Remove webcontext implementation on wkwebview ([#922](https://github.com/tauri-apps/wry/pull/922)) on 2023-04-07
- Use the new WKWebView `inspectable` property if available (iOS 16.4, macOS 13.3).
  - [c3f7304](https://github.com/tauri-apps/wry/commit/c3f7304dbfd45d1e1c27b53be2369c737e946b69) feat(macos): use WKWebView's inspectable property ([#923](https://github.com/tauri-apps/wry/pull/923)) on 2023-04-08

## \[0.27.2]

- On Android, Add support for native back button navigation.
  - [fc232a3](https://github.com/tauri-apps/wry/commit/fc232a32268a13ec89965450dd6cf0abca064b24) feat(android): add support for native back navigation ([#918](https://github.com/tauri-apps/wry/pull/918)) on 2023-04-03
- Fix `WebView::url` getter on Android.
  - [427cf92](https://github.com/tauri-apps/wry/commit/427cf9222d7152f911aa70eb778eb7aa90c83fac) Unify custom porotocol across Android/iOS ([#546](https://github.com/tauri-apps/wry/pull/546)) on 2022-04-11
  - [b89398a](https://github.com/tauri-apps/wry/commit/b89398a9bb17303544a1f04303783f311c6dc77f) Publish New Versions ([#547](https://github.com/tauri-apps/wry/pull/547)) on 2022-04-26
  - [c22744a](https://github.com/tauri-apps/wry/commit/c22744a0c11e9c78f548dc3786e6be30c1d6f46f) fix(android): use correct method signature ([#917](https://github.com/tauri-apps/wry/pull/917)) on 2023-03-31
- Add Webview attribute to enable/disable autoplay. Enabled by default.
  - [6a523cc](https://github.com/tauri-apps/wry/commit/6a523cc7a633236e1fb562e0626e0aedc67ec2fc) feat: Add setting to enable autoplay ([#913](https://github.com/tauri-apps/wry/pull/913)) on 2023-04-04
- Fix the `WebViewBuilder::with_url` when the projet use `mimalloc`
  - [c22744a](https://github.com/tauri-apps/wry/commit/c22744a0c11e9c78f548dc3786e6be30c1d6f46f) fix(android): use correct method signature ([#917](https://github.com/tauri-apps/wry/pull/917)) on 2023-03-31
- Revert [`51b49c54`](https://github.com/tauri-apps/wry/commit/51b49c54e41c71d1c5f03b568094d43fb9dc32ac) which hid the webview when minimized on Windows.
  - [f76568a](https://github.com/tauri-apps/wry/commit/f76568a1cc8f7e56f36633d2f6e700af684bb213) fix(windows): Ignore resize event when minimizing frameless windows ([#909](https://github.com/tauri-apps/wry/pull/909)) on 2023-03-24

## \[0.27.1]

- On Windows, Linux and macOS, add method `evaluate_script_with_callback` to execute javascipt with a callback.
  Evaluated result will be serialized into JSON string and pass to the callback.
  - [2647731](https://github.com/tauri-apps/wry/commit/2647731c1f084565895a5306fa6465ee6cd271c2) feat: support callback function in eval ([#778](https://github.com/tauri-apps/wry/pull/778)) on 2023-03-23
- On iOS, set webview scroll bounce default to NO.
  - [4d61cf1](https://github.com/tauri-apps/wry/commit/4d61cf122dc0e5b2cef818e0fd491dbd0fd47621) fix(ios): set scroll bounce default to NO ([#907](https://github.com/tauri-apps/wry/pull/907)) on 2023-03-20
- Update the value returned on a `None` value of `ClassDecl::new("WryDownloadDelegate", class!(NSObject))`
  from `UIViewController` to `WryDownloadDelegate`.
  - [7795356](https://github.com/tauri-apps/wry/commit/7795356a45b1bd015fad0e9973fc5af58c8c339b) fix: WryDownloadDelegate call after first time on 2023-02-20
- On Linux, disable system appearance for scrollbars.
  - [530a8b7](https://github.com/tauri-apps/wry/commit/530a8b73766dc54736ae6de9528683b27430eaa6) fix(linux): disable system appearance for scrollbars ([#897](https://github.com/tauri-apps/wry/pull/897)) on 2023-03-08
- On Windows and Linux, implement `WebviewBuilder::with_back_forward_navigation_gestures` and `WebviewAttributes::back_forward_navigation_gestures` to control swipe navigation. Disabled by default.
  - [15b4ddf](https://github.com/tauri-apps/wry/commit/15b4ddf7698cf04b90ffcc3164ccb7b62daf6ed0) feat(win\&linux): implement the option to control gesture navigation ([#896](https://github.com/tauri-apps/wry/pull/896)) on 2023-03-07

## \[0.27.0]

- Add function to dispatch closure with the Android context.
  - [a9e186c](https://github.com/tauri-apps/wry/commit/a9e186cab4456d7ac2c265e61e71b345f7d269c4) feat(android): add function to dispatch closure to the Android context ([#864](https://github.com/tauri-apps/wry/pull/864)) on 2023-02-06
- On macOS, fix crash when getting dragging position.
  - [a8f7cef](https://github.com/tauri-apps/wry/commit/a8f7cefaac72d3e9fd2f8901f790a777d9888357) Fix crash when getting drag position ([#867](https://github.com/tauri-apps/wry/pull/867)) on 2023-02-04
- On Android, `wry` can again load assets from the apk's `asset` folder via a custom protocol. This is set by `WebViewBuilder`'s method `with_asset_loader`, which is exclusive to Android (by virtue of existing within `WebViewBuilderExtAndroid`).
  - [077eb3a](https://github.com/tauri-apps/wry/commit/077eb3a7ca520d07e73f899da60ce23eef941e6f) fix(android): restore asset loading functionality to android (fix: [#846](https://github.com/tauri-apps/wry/pull/846)) ([#854](https://github.com/tauri-apps/wry/pull/854)) on 2023-02-07
- Update `webview2-com` to `0.22` and `windows-rs` to `0.44` which bumps the MSRV of this crate on Windows to `1.64`.
  - [496bfb5](https://github.com/tauri-apps/wry/commit/496bfb5c7be55e9c2bb674e241f9d7d2620e2acd) chore(deps): update to windows-rs 0.44 and webview2-com 0.22 ([#871](https://github.com/tauri-apps/wry/pull/871)) on 2023-02-06

## \[0.26.0]

- Added `WebViewBuilderExtAndroid` trait and with `on_webview_created` hook.
  - [08c0156](https://github.com/tauri-apps/wry/commit/08c0156c60e016bd77f6e0f1bd16ae31dc48d4a0) feat(android): add on_webview_created hook, expose find_class ([#855](https://github.com/tauri-apps/wry/pull/855)) on 2023-01-30
- Enable dox feature when building docs.
  - [c6e53c6](https://github.com/tauri-apps/wry/commit/c6e53c6fa007dcc2dc4771a94b7f312f95edd892) Enable dox feature when building docs ([#861](https://github.com/tauri-apps/wry/pull/861)) on 2023-01-31
- Expose `wry::webview::prelude::find_class` function to find an Android class in the app project scope.
  - [08c0156](https://github.com/tauri-apps/wry/commit/08c0156c60e016bd77f6e0f1bd16ae31dc48d4a0) feat(android): add on_webview_created hook, expose find_class ([#855](https://github.com/tauri-apps/wry/pull/855)) on 2023-01-30
- Added `WebviewExtIOS` trait to access the WKWebView and userContentController references.
  - [f546c44](https://github.com/tauri-apps/wry/commit/f546c44fce76faf04855a97b285bbdef8ae80f3d) feat(ios): add WebviewExtIOS ([#859](https://github.com/tauri-apps/wry/pull/859)) on 2023-01-30

## \[0.25.0]

- **Breaking Change:** Bump webkit2gtk to 0.19. This will use webkit2gtk-4.1 as dependency from now on. Also Bump gtk version: 0.15 -> 0.16.
  - [c5f3b36](https://github.com/tauri-apps/wry/commit/c5f3b36b7ac4613971ddf56397932c44a9c74878) Bump gtk version 0.15 -> 0.16 ([#851](https://github.com/tauri-apps/wry/pull/851)) on 2023-01-26
- **Breaking** Add position of the drop to `FileDropEvent` struct.
  - [bce39e2](https://github.com/tauri-apps/wry/commit/bce39e2be195194e547b0021e770e45a3df15fa1) feat: add file drop position ([#847](https://github.com/tauri-apps/wry/pull/847)) on 2023-01-17
- On Android, fix the injection of `intialization_scripts` for devServers where the `Content-Type` header includes more information than just `"text/plain"`.
  - [87216c7](https://github.com/tauri-apps/wry/commit/87216c7f01d5f65641422343dd0aa7f08ea61d0d) fix: make the Content-Type check spec compliant ([#844](https://github.com/tauri-apps/wry/pull/844)) on 2023-01-14

## \[0.24.1]

- Update `tao` to `0.16.0`
  - [a27a66b](https://github.com/tauri-apps/wry/commit/a27a66baccc86873110b0aa67ddad1f3a8dbd205) chore: update tao to 0.16.0 on 2023-01-11

## \[0.24.0]

- Changed env vars used when building for Android; changed `WRY_ANDROID_REVERSED_DOMAIN` to `WRY_ANDROID_PACKAGE` and `WRY_ANDROID_APP_NAME_SNAKE_CASE` to `WRY_ANDROID_LIBRARY`.
  - [dfe6a5e](https://github.com/tauri-apps/wry/commit/dfe6a5e78acca05d9e0808c8f4ed974a8657b847) refactor: improve android env vars naming ([#829](https://github.com/tauri-apps/wry/pull/829)) on 2022-12-30
- Fixes Android initialization scripts order.
  - [7f819c0](https://github.com/tauri-apps/wry/commit/7f819c0ec3d3aaaf582d9eecde09f5e539c45743) fix(android): initialization scripts order ([#808](https://github.com/tauri-apps/wry/pull/808)) on 2022-12-12
- Remove redundant `.clone()` calls and avoid unnecessary heap allocations.
  - [45f2b21](https://github.com/tauri-apps/wry/commit/45f2b2127e73718b71f349eae1847d1764c748f5) perf: remove redundant `.clone()` calls and avoid unnecessary heap allocations ([#812](https://github.com/tauri-apps/wry/pull/812)) on 2022-12-14
- Change return type of [custom protocol handlers](https://docs.rs/wry/latest/wry/webview/struct.WebViewBuilder.html#method.with_custom_protocol) from `Result<Response<Vec<u8>>>` to `Result<Response<Cow<'static, [u8]>>>`. This allows the handlers to return static resources without heap allocations. This is effective when you embed some large files like bundled JavaScript source as `&'static [u8]` using [`include_bytes!`](https://doc.rust-lang.org/std/macro.include_bytes.html).
  - [ddd3461](https://github.com/tauri-apps/wry/commit/ddd34614be8a0ba826eff8acbf4b06710ce2ba65) perf: Change return type of custom protocol handler from `Vec<u8>` to `Cow<'static, [u8]>`, closes [#796](https://github.com/tauri-apps/wry/pull/796) ([#797](https://github.com/tauri-apps/wry/pull/797)) on 2022-12-12
- Ensures that the script passed to `.with_initialization_script("here")` is not empty.
  - [ceb209e](https://github.com/tauri-apps/wry/commit/ceb209eddc20d284be748ee382ba8aef7686863b) fix empty string bug (fix: [#833](https://github.com/tauri-apps/wry/pull/833)) ([#836](https://github.com/tauri-apps/wry/pull/836)) on 2023-01-08
- Add APIs to process webview document title change.
  - [14a0ee3](https://github.com/tauri-apps/wry/commit/14a0ee323e8e596f45d4a57d2d86abcf0a848bc8) feat: add document title changed handler, closes [#804](https://github.com/tauri-apps/wry/pull/804) ([#825](https://github.com/tauri-apps/wry/pull/825)) on 2022-12-30
- Evaluate scripts after the page load starts on Linux and macOS.
  - [ca7c8e4](https://github.com/tauri-apps/wry/commit/ca7c8e44832b3236f08022f7ea3469be9a65aa3f) fix(unix): race condition on script eval ([#815](https://github.com/tauri-apps/wry/pull/815)) on 2022-12-14
- Improve panic error messages on the build script.
  - [5b9f21d](https://github.com/tauri-apps/wry/commit/5b9f21d38974881c2d6f4456990f5863484e7382) feat: improve build script panic messages ([#807](https://github.com/tauri-apps/wry/pull/807)) on 2022-12-12
- Add `WebViewBuilder::with_url_and_headers` and `WebView::load_url_with_headers` to navigate to urls with headers.
  - [8ae93b9](https://github.com/tauri-apps/wry/commit/8ae93b9c76b2efe14e93febd009e31fc459275a8) feat: add headers when loading URLs, closes [#816](https://github.com/tauri-apps/wry/pull/816) ([#826](https://github.com/tauri-apps/wry/pull/826)) on 2023-01-01
  - [e246bd1](https://github.com/tauri-apps/wry/commit/e246bd164eb9df1b0e48123a542bbd240958c9db) chore: update headers change file on 2023-01-01
- Change class declare name from `UIViewController` to `WryNavigationDelegate` to avoid class name conflict on iOS.
  - [fca42a0](https://github.com/tauri-apps/wry/commit/fca42a0730e75a142f7f354c6ac3f6d6a0f4711f) fix(ios): navigation delegate class name conflict ([#824](https://github.com/tauri-apps/wry/pull/824)) on 2022-12-27
- Rerun build script if the `WRY_ANDROID_KOTLIN_FILES_OUT_DIR` directory changes.
  - [1cf92e2](https://github.com/tauri-apps/wry/commit/1cf92e2b68b1d9109de3924a3cd1fd10cb8c7c17) feat(build): rerun if kotlin out directory changes ([#839](https://github.com/tauri-apps/wry/pull/839)) on 2023-01-10
- On Windows, Add `WebviewBuilderExtWindows::with_theme` and `WebviewExtWindows::set_theme` to change webview2 theme.
  - [563a497](https://github.com/tauri-apps/wry/commit/563a497d7f842c760ad05a0017059e7781c2b810) feat(webview2): add theme API, closes [#806](https://github.com/tauri-apps/wry/pull/806) ([#809](https://github.com/tauri-apps/wry/pull/809)) on 2022-12-13

## \[0.23.4]

- Fixes Android initialization scripts order.
  - [800cc48](https://github.com/tauri-apps/wry/commit/800cc48b46ba9e5ce968efd5708aeb71b63832f9) fix(android): initialization scripts order ([#808](https://github.com/tauri-apps/wry/pull/808)) on 2022-12-12
- Improve panic error messages on the build script.
  - [4ec7386](https://github.com/tauri-apps/wry/commit/4ec7386740ab2edb3b56d72668841af3f329cefd) feat: improve build script panic messages ([#807](https://github.com/tauri-apps/wry/pull/807)) on 2022-12-12

## \[0.23.3]

- Fix the beep sound on macOS
  - [94256c3](https://github.com/tauri-apps/wry/commit/94256c3adb1d6c005e0386f8b20f01d597b52f28) Fix beep sound, closes [#799](https://github.com/tauri-apps/wry/pull/799) ([#801](https://github.com/tauri-apps/wry/pull/801)) on 2022-12-10

## \[0.23.2]

- On macOS, remove all custom keydown implementations. This will bring back keydown regression but should allow all accelerator working.
  - [fee4bf2](https://github.com/tauri-apps/wry/commit/fee4bf2eb384d9c315530bd8f5af146909706cf6) Remove all keydown implementations ([#798](https://github.com/tauri-apps/wry/pull/798)) on 2022-12-10
- Suppress `unused_variables` warning reported only in release build.
  - [4e23c0f](https://github.com/tauri-apps/wry/commit/4e23c0f84b5a954be78418d56e37366395de030f) fix(macos): suppress `unused_variables` warning reported only in release build ([#790](https://github.com/tauri-apps/wry/pull/790)) on 2022-12-07
- Add `WebViewBuilderExtWindows::with_browser_accelerator_keys` method to allow disabling browser-specific accelerator keys enabled in WebView2 by default. When `false` is passed, it disables all accelerator keys that access features specific to a web browser. See [the official WebView2 document](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2\_core/corewebview2settings#arebrowseracceleratorkeysenabled) for more details.
  - [6e622ff](https://github.com/tauri-apps/wry/commit/6e622ffbdad2312bf3906d278a75956a3a6eeadd) feat(windows): Allow disabling browser-specific accelerator keys ([#792](https://github.com/tauri-apps/wry/pull/792)) on 2022-12-07

## \[0.23.1]

- Fixes usage of the `linux-headers` feature.
  - [64a72ff](https://github.com/tauri-apps/wry/commit/64a72ffd2369f51d36bdb00973f71326e8395016) fix(wry): correctly use the linux-headers feature on 2022-12-05

## \[0.23.0]

- Properly parse the content type header for the `android.webkit.WebResourceResponse` mime type.
  - [1db5ea6](https://github.com/tauri-apps/wry/commit/1db5ea68c2028db77788ec8c78ee0ab75a7a5f7f) fix(android): properly parse content-type for response mime type ([#772](https://github.com/tauri-apps/wry/pull/772)) on 2022-11-27
- Change typo in `WebViewBuilderExtWindows::with_additionl_browser_args`. to `WebViewBuilderExtWindows::with_additional_browser_args`.
  - [db1c290](https://github.com/tauri-apps/wry/commit/db1c290c0d8b58f6612ef9bef244a06261fb2a6e) fix(windows): Fix typo in method name of `WebViewBuilderExtWindows` ([#781](https://github.com/tauri-apps/wry/pull/781)) on 2022-12-02
- Add `Webiew::load_url`.
  - [a2b9531](https://github.com/tauri-apps/wry/commit/a2b9531b0e8397dcf74c049ccf6c7fa125288ca8) feat: add `Webiew::navigate_to_url`, closes [#776](https://github.com/tauri-apps/wry/pull/776) ([#777](https://github.com/tauri-apps/wry/pull/777)) on 2022-11-30
- Change the type of `WebViewBuilderExtWindows::with_additional_browser_args` argument from `AsRef<str>` to `Into<String>` to reduce extra allocation.
  - [b0ff06a](https://github.com/tauri-apps/wry/commit/b0ff06aba5aea77f067aee1e9bf8ac8c245ac5e8) perf: reduce extra allocation at `WebViewBuilderExtWindows::with_additional_browser_args` argument ([#783](https://github.com/tauri-apps/wry/pull/783)) on 2022-12-03
- Validate custom protocol response status code on Android.
  - [7f585c7](https://github.com/tauri-apps/wry/commit/7f585c7dc947936387faf565f3f5cbe62148daaf) feat(android): validate custom protocol response status code ([#779](https://github.com/tauri-apps/wry/pull/779)) on 2022-11-30
- \[https://github.com/tauri-apps/wry/commit/04422bc1b579d9388ce03c2388b8f415dbc0747b] On macOS, revert content view to native NSView (\[#782])(https://github.com/tauri-apps/wry/pull/782)

## \[0.22.6]

- Fixes usage of the `linux-headers` feature.
  - [14c5ae7](https://github.com/tauri-apps/wry/commit/14c5ae7d41b506c8a398d4735062b46cd0770447) fix(wry): correctly use the linux-headers feature on 2022-12-05

## \[0.22.5]

- On macOS, fix arrow keys misprint text on textarea.
  - [3005e54](https://github.com/tauri-apps/wry/commit/3005e5450339c6c3fbc1c7c67ab8008ed39ec864) On macOS, fix arrow keys misprint texts ([#769](https://github.com/tauri-apps/wry/pull/769)) on 2022-11-25

## \[0.22.4]

- On Linux, add `linux-headers` feature flag to fix version regression. The minimum webkit2gtk version remains v2.22.
  - [cf447f6](https://github.com/tauri-apps/wry/commit/cf447f64451fd8345f21440df31601265e0fde86) On Linux, add header feature flag to fix version regression ([#766](https://github.com/tauri-apps/wry/pull/766)) on 2022-11-24

## \[0.22.3]

- On macOS, fix keyinput missing by calling superclass methods.
  - [e40e55a](https://github.com/tauri-apps/wry/commit/e40e55a41d8d65ceda5e182c8915d37b5698c7b0) On macOS, fix keyinput missing by calling super class methods ([#764](https://github.com/tauri-apps/wry/pull/764)) on 2022-11-21

## \[0.22.2]

- On macOS, add an API to enable or disable backward and forward navigation gestures.
  - [487dff0](https://github.com/tauri-apps/wry/commit/487dff03a103df999e5e0c6286f75b4d1f419d25) Add the ability to navigate with swipe gesture ([#757](https://github.com/tauri-apps/wry/pull/757)) on 2022-11-16
  - [1a0ec19](https://github.com/tauri-apps/wry/commit/1a0ec19fd533c853b744c5e2346542d2e1e5805d) Update gesture change file to patch ([#763](https://github.com/tauri-apps/wry/pull/763)) on 2022-11-21
- On macOS, pass key event to menu if we have one on key press.
  - [2e5e138](https://github.com/tauri-apps/wry/commit/2e5e1381789c332654a5ffee47d578042a9be87b) On macOS, pass key event to menu on key press ([#760](https://github.com/tauri-apps/wry/pull/760)) on 2022-11-21

## \[0.22.1]

- Fix `WebViewBuilder::with_accept_first_mouse` taking behavior of first initalized webview.
  - [0647c0e](https://github.com/tauri-apps/wry/commit/0647c0efe131566ffbab0729e9d74355155c3c32) fix(macos): fix `acceptFirstMouse` for subsequent webviews, closes [#751](https://github.com/tauri-apps/wry/pull/751) ([#752](https://github.com/tauri-apps/wry/pull/752)) on 2022-11-13
- Fix download implementation on macOS older than 11.3.
  - [e69ddc6](https://github.com/tauri-apps/wry/commit/e69ddc6943770aa8baa02431bb037bbdcb3cbd80) fix(macos): download breaking app on macOS older than 11.3, closes [#755](https://github.com/tauri-apps/wry/pull/755) ([#756](https://github.com/tauri-apps/wry/pull/756)) on 2022-11-15
- On macOS, remove webview from window's NSView before dropping.
  - [3d3ea80](https://github.com/tauri-apps/wry/commit/3d3ea80808a327c546d8bbd97e06ef4b8feb32d0) On macOS, remove webview from window's NSView before dropping ([#754](https://github.com/tauri-apps/wry/pull/754)) on 2022-11-14

## \[0.22.0]

- Added `WebViewAttributes::with_accept_first_mouse` method for macOS.
  - [2c23440](https://github.com/tauri-apps/wry/commit/2c23440f9c194064caa907650df39bf9c96ed99c) feat(macos): add `accept_first_mouse` option, closes [#714](https://github.com/tauri-apps/wry/pull/714) ([#715](https://github.com/tauri-apps/wry/pull/715)) on 2022-10-04
- **Breaking change** Custom protocol now takes `Request` and returns `Response` types from `http` crate.
  - [1510e45](https://github.com/tauri-apps/wry/commit/1510e452547a95af2e42ff5199640877beecdbd7) refactor: use `http` crate primitives instead of a custom impl ([#706](https://github.com/tauri-apps/wry/pull/706)) on 2022-09-29
- Enabled devtools in debug mode by default.
  - [fea0638](https://github.com/tauri-apps/wry/commit/fea0638d9ad100c00b95468aa16fc44d6517ac0d) feat: enable devtools in debug mode by default ([#741](https://github.com/tauri-apps/wry/pull/741)) on 2022-10-27
- On Desktop, add `download_started_handler` and `download_completed_handler`. See `blob_download` and `download_event` example for their usages.
  - [3691c4f](https://github.com/tauri-apps/wry/commit/3691c4f6c88fe43e92597caf3003c8d57b447a7b) feat: Add download started and download completed callbacks ([#530](https://github.com/tauri-apps/wry/pull/530)) on 2022-10-19
- Fix double permission dialog on macOS 12+ and iOS 15+.
  - [8aa7d61](https://github.com/tauri-apps/wry/commit/8aa7d61cdc9fc584805b46c3ffd700aabb633649) Fix: Remove extra soft prompt asking for media permission on every app launch in macOS ([#694](https://github.com/tauri-apps/wry/pull/694)) on 2022-09-29
- Focus webview when window starts moving or resizing on Windows to automatically close `<select>` dropdowns. Also notify webview2 whenever the window position/size changes which fixes the `<select>` dropdown position
  - [a1001dd](https://github.com/tauri-apps/wry/commit/a1001dd6361a0629cd1ce2f8063b7c983bf29616) fix(windows): focus webview on `WM_ENTERSIZEMOVE` and call `NotifyParentChanged` on `WM_WINDOWPOSCHANGED`. ([#695](https://github.com/tauri-apps/wry/pull/695)) on 2022-09-16
- On Windows, hide the webview when the window is minimized to reduce memory and cpu usage.
  - [51b49c5](https://github.com/tauri-apps/wry/commit/51b49c54e41c71d1c5f03b568094d43fb9dc32ac) feat(webview2): hide the webview when the window is minimized ([#702](https://github.com/tauri-apps/wry/pull/702)) on 2022-09-27
- Internally return with error from custom protocol if an invalid uri was requseted such as `wry://` which doesn't contain a host.
  - [818ce99](https://github.com/tauri-apps/wry/commit/818ce9989d816bf970ebcf93009b2d693384e436) fix: don't panic on invalid uri ([#712](https://github.com/tauri-apps/wry/pull/712)) on 2022-09-30
- Support cross compiling ios on a non macos host.
  - [cd08410](https://github.com/tauri-apps/wry/commit/cd08410bce326c42e8fc25a74290d254468724fe) Fix cross compilation. ([#731](https://github.com/tauri-apps/wry/pull/731)) on 2022-10-29
- On Linux, Improve custom protocol with http headers / method added to request, and status code / http headers added to response. This feature is 2.36 only, version below it will fallback to previous implementation.
  - [2944d91](https://github.com/tauri-apps/wry/commit/2944d91c763ff105288aa6c1370ba42a54fa8caf) feat(linux): add headers to URL scheme request ([#721](https://github.com/tauri-apps/wry/pull/721)) on 2022-10-17
- On macOS, add WKWebview as subview of existing NSView directly.
  - [008eca8](https://github.com/tauri-apps/wry/commit/008eca871155f393e5de1053bb1a9f63e1eafe82) On macOS, add WKWebview as subview of existing NSView directly ([#745](https://github.com/tauri-apps/wry/pull/745)) on 2022-11-07
- Keypress on non-input element no longer triggers unsupported key feedback sound.
  - [51c7f12](https://github.com/tauri-apps/wry/commit/51c7f12d80e2b51a188fb644a323abaf5df1b3d1) fix(macos): do not trigger unsupported key feedback sound on keypress ([#742](https://github.com/tauri-apps/wry/pull/742)) on 2022-10-30
- Remove the IPC script message handler when the WebView is dropped on macOS.
  - [818ce99](https://github.com/tauri-apps/wry/commit/818ce9989d816bf970ebcf93009b2d693384e436) fix: don't panic on invalid uri ([#712](https://github.com/tauri-apps/wry/pull/712)) on 2022-09-30
- **Breaking change** Removed http error variants from `wry::Error` and replaced with generic `HttpError` variant that can be used to convert `http` crate errors.
  - [1510e45](https://github.com/tauri-apps/wry/commit/1510e452547a95af2e42ff5199640877beecdbd7) refactor: use `http` crate primitives instead of a custom impl ([#706](https://github.com/tauri-apps/wry/pull/706)) on 2022-09-29
- Disabled Microsoft SmartScreen by default on Windows.
  - [a617c5b](https://github.com/tauri-apps/wry/commit/a617c5b29da3d173d43aa814106e1c7ace08d27f) feat(webview2): disable smartscreen & allow disabling internal webview2 args, closes [#704](https://github.com/tauri-apps/wry/pull/704) ([#705](https://github.com/tauri-apps/wry/pull/705)) on 2022-09-28
- Add `WebView::url` to get the current url.
  - [38e49bd](https://github.com/tauri-apps/wry/commit/38e49bd5f1e26e9f9507d1f2af8b0b290aa515ad) feat: add `WebView::url()` to access the current url ([#732](https://github.com/tauri-apps/wry/pull/732)) on 2022-10-25
- **Breaking change** Removed `http` module and replaced with re-export of `http` crate.
  - [1510e45](https://github.com/tauri-apps/wry/commit/1510e452547a95af2e42ff5199640877beecdbd7) refactor: use `http` crate primitives instead of a custom impl ([#706](https://github.com/tauri-apps/wry/pull/706)) on 2022-09-29
- Add `WebviewBuilderExtWindows::with_additionl_browser_args` method to pass additional browser args to Webview2 On Windows. By default wry passes `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection` so if you use this method, you also need to disable these components by yourself if you want.
  - [683f866](https://github.com/tauri-apps/wry/commit/683f86665366bb333cb03e05a503a69d0f8eb734) feat(webview2): add method to pass additional args, closes [#415](https://github.com/tauri-apps/wry/pull/415) ([#711](https://github.com/tauri-apps/wry/pull/711)) on 2022-09-29
- On Windows, fix canonical reason for custom protocol response.
  - [9d5595c](https://github.com/tauri-apps/wry/commit/9d5595c9c723b3f8046d9582ac086ccebf460a83) fix(webview2): set response reason correctly, closes [#733](https://github.com/tauri-apps/wry/pull/733) ([#734](https://github.com/tauri-apps/wry/pull/734)) on 2022-10-24
- On macOS, make the webview first responder.
  - [e64ad21](https://github.com/tauri-apps/wry/commit/e64ad21ad5ab9bf0b7fb15aec0065c20b61a5a80) fix(wkwebview): make webview first responder ([#740](https://github.com/tauri-apps/wry/pull/740)) on 2022-10-28

## \[0.21.1]

- Fix transparency on Windows
  - [e31cd0a](https://github.com/tauri-apps/wry/commit/e31cd0adf4ba881a35dcccd9b5ee78bb5af8828a) fix: fix transparency on Windows, closes [#692](https://github.com/tauri-apps/wry/pull/692) on 2022-09-16

## \[0.21.0]

- Implement `<input type="file">` on Android.
  - [bf39d9d](https://github.com/tauri-apps/wry/commit/bf39d9de1e997170e9efb3bb7392710b57c2ae1f) feat(android): implement dialogs and permissions ([#685](https://github.com/tauri-apps/wry/pull/685)) on 2022-09-05
- Add `WebviewExtAndroid::handle` which can be used to execute some code using JNI context.
  - [2bfc6c3](https://github.com/tauri-apps/wry/commit/2bfc6c3d2e0cc6c3922d125f678ab30c00b89483) feat(android): JNI execution handle ([#689](https://github.com/tauri-apps/wry/pull/689)) on 2022-09-07
- Enable JS alert, confirm, prompt on Android.
  - [bf39d9d](https://github.com/tauri-apps/wry/commit/bf39d9de1e997170e9efb3bb7392710b57c2ae1f) feat(android): implement dialogs and permissions ([#685](https://github.com/tauri-apps/wry/pull/685)) on 2022-09-05
- Prompt for permissions on Android when needed.
  - [bf39d9d](https://github.com/tauri-apps/wry/commit/bf39d9de1e997170e9efb3bb7392710b57c2ae1f) feat(android): implement dialogs and permissions ([#685](https://github.com/tauri-apps/wry/pull/685)) on 2022-09-05
- Implement `webview_version` on Android.
  - [9183de4](https://github.com/tauri-apps/wry/commit/9183de4f9d3129e7cba332eebca2afc846f727d0) feat(android): implement webview_version ([#687](https://github.com/tauri-apps/wry/pull/687)) on 2022-09-05
- Enable storage, geolocation, media playback, `window.open`.
  - [9dfffcf](https://github.com/tauri-apps/wry/commit/9dfffcfe12199d7f28bf4b8a837e28253958ac17) feat(android): enable storage, geolocation, media playback, window.open ([#684](https://github.com/tauri-apps/wry/pull/684)) on 2022-09-04
- Improve Android initialization script implementation.
  - [1b26d60](https://github.com/tauri-apps/wry/commit/1b26d605d6e33f5417eb6566a7381d8feb239c8b) feat(android): improve initialization scripts implementation ([#670](https://github.com/tauri-apps/wry/pull/670)) on 2022-08-24
- WRY will now generate the needed kotlin files at build time but you need to set `WRY_ANDROID_REVERSED_DOMAIN`, `WRY_ANDROID_APP_NAME_SNAKE_CASE` and `WRY_ANDROID_KOTLIN_FILES_OUT_DIR` env vars.
  - [b478903](https://github.com/tauri-apps/wry/commit/b4789034dc4d10ab83f6acce6b4152d79f702940) feat(android): generate kotlin files at build time ([#671](https://github.com/tauri-apps/wry/pull/671)) on 2022-08-24
  - [103f255](https://github.com/tauri-apps/wry/commit/103f255903bdf728bf5124fb323293d172c8dd12) chore: change bump to patch on 2022-08-25
- **Breaking change** Removed `WebView::focus`.
  - [f338df7](https://github.com/tauri-apps/wry/commit/f338df7a2716cbbde357b81d9baa108ce679eaa5) feat(windows): auto-focus the webview ([#676](https://github.com/tauri-apps/wry/pull/676)) on 2022-08-27
- Updated tao to `0.14`
  - [483bad0](https://github.com/tauri-apps/wry/commit/483bad0fc7e7564500f7183547c15604fa387258) feat: tao as window dependency ([#230](https://github.com/tauri-apps/wry/pull/230)) on 2021-05-03
  - [51430e9](https://github.com/tauri-apps/wry/commit/51430e97dfb6589c5ff71e5078438be67293d044) publish new versions ([#221](https://github.com/tauri-apps/wry/pull/221)) on 2021-05-09
  - [0cf0089](https://github.com/tauri-apps/wry/commit/0cf0089b6d49aa9e1a8c791ec8883fce48a0dfd1) Update tao to v0.2.6 ([#271](https://github.com/tauri-apps/wry/pull/271)) on 2021-05-18
  - [a76206c](https://github.com/tauri-apps/wry/commit/a76206c11fa0a4ba1d041aa0f25452dd80941ee9) publish new versions ([#272](https://github.com/tauri-apps/wry/pull/272)) on 2021-05-18
  - [3c4f8b8](https://github.com/tauri-apps/wry/commit/3c4f8b8b2bd42e7634b889aa5317d909bfce593c) Update tao to v0.5 ([#365](https://github.com/tauri-apps/wry/pull/365)) on 2021-08-09
  - [44aa1dc](https://github.com/tauri-apps/wry/commit/44aa1dc8fcc20cc5826697d69f763118d45f724a) publish new versions ([#351](https://github.com/tauri-apps/wry/pull/351)) on 2021-08-09
  - [935cc5f](https://github.com/tauri-apps/wry/commit/935cc5fe8b73055279dc107e71a10f2701ea8b3d) Update tao to 0.13 ([#642](https://github.com/tauri-apps/wry/pull/642)) on 2022-07-27
  - [657888a](https://github.com/tauri-apps/wry/commit/657888aac13830d97d2970bdf1c87319dadb2ffa) Publish New Versions ([#632](https://github.com/tauri-apps/wry/pull/632)) on 2022-07-27
  - [3a91376](https://github.com/tauri-apps/wry/commit/3a91376fa2c04783a32804e6f123722749ad595e) chore(deps): update tao to 0.14 ([#691](https://github.com/tauri-apps/wry/pull/691)) on 2022-09-13
- Allow setting the webview background color.
  - [eb1b723](https://github.com/tauri-apps/wry/commit/eb1b7234f731759b5e091f7c88ac18ce4b507017) feat: allow setting webview bg color, closes [#197](https://github.com/tauri-apps/wry/pull/197) ([#682](https://github.com/tauri-apps/wry/pull/682)) on 2022-09-05
- Added the `RustWebView` class on Android.
  - [b1e8560](https://github.com/tauri-apps/wry/commit/b1e8560c3f13f2674528f6ca440ba476ddbef7c2) feat(android): define WebView class in kotlin ([#672](https://github.com/tauri-apps/wry/pull/672)) on 2022-08-24
- Update the `windows` crate to the latest 0.39.0 release and `webview2-com` to 0.19.1 to match.
  - [c7d7e1f](https://github.com/tauri-apps/wry/commit/c7d7e1f9c85a5db9c98aa5ded1e0eaf7fe697817) Update windows to 0.39.0 and webview2-com to 0.19.1 to match ([#679](https://github.com/tauri-apps/wry/pull/679)) on 2022-08-31
- On Windows, automatically focus the webview when the window gains focus to match other platforms.
  - [f338df7](https://github.com/tauri-apps/wry/commit/f338df7a2716cbbde357b81d9baa108ce679eaa5) feat(windows): auto-focus the webview ([#676](https://github.com/tauri-apps/wry/pull/676)) on 2022-08-27

## \[0.20.2]

- Implement custom protocol on Android.
  - [dc68289](https://github.com/tauri-apps/wry/commit/dc68289169196419b8c9cda73c73b139ea1301f9) feat(android): implement custom protocol ([#656](https://github.com/tauri-apps/wry/pull/656)) on 2022-08-13
- Implement `WebView::eval` on Android.
  - [690fd26](https://github.com/tauri-apps/wry/commit/690fd26a3b9bd47f9d7b1b5d2aa3dcb1c018a771) feat(android): implement eval ([#658](https://github.com/tauri-apps/wry/pull/658)) on 2022-08-13
- On iOS, add webview as subview instead of replacing original view.
  - [74391e0](https://github.com/tauri-apps/wry/commit/74391e0769d0e0f4be015147ddfa39bf25c90928) fix(ios): addSubview instead of setContentView ([#655](https://github.com/tauri-apps/wry/pull/655)) on 2022-08-13
- Move WebView logic from tao to wry.
  - [aba1ae5](https://github.com/tauri-apps/wry/commit/aba1ae5afcf96c88b1215ef66f38a5a635ecf7c3) refactor(android): move WebView logic from tao to wry ([#659](https://github.com/tauri-apps/wry/pull/659)) on 2022-08-14

## \[0.20.1]

- Add android support
  - [3218091](https://github.com/tauri-apps/wry/commit/3218091aa393dca9451840d3baa44bc9371f2e1d) Add real android support [#577](https://github.com/tauri-apps/wry/pull/577)
- Enable private picture-in-picture on macos.
  - [3cfd8c9](https://github.com/tauri-apps/wry/commit/3cfd8c9e7a43f6c35a1ea61358521bd62fc70633) fix: add feature flag to enable private picture-in-picture flag on macos ([#645](https://github.com/tauri-apps/wry/pull/645)) on 2022-08-05
- On macOS, fix devtool warning
  - [2eba8c9](https://github.com/tauri-apps/wry/commit/2eba8c9c26ff5f9512b0039ac04bc7fd27a5256f) fix: devtool warning by adding parent view

## \[0.20.0]

- Add `WebViewBuilder::with_clipboard`.
  - [c798700](https://github.com/tauri-apps/wry/commit/c7987004eaaf5cb7da830d574d81bd96dace0112) fix: Add `WebViewBuilder::with_clipboard`([#631](https://github.com/tauri-apps/wry/pull/631)) on 2022-07-05
- Fix typos in several files.
  - [4466250](https://github.com/tauri-apps/wry/commit/44662506ab01846c7e8767eb2f13bf0bbca7fe9a) Fix typos ([#635](https://github.com/tauri-apps/wry/pull/635)) on 2022-07-11
- Set webview2 language to match the OS language. This makes i18n functions like `new Date().toLocaleStrin()` behave correctly.
  - [e9f04d7](https://github.com/tauri-apps/wry/commit/e9f04d7e7bea576d0283d97e25faf7b356c5e959) fix: set system language to webview on windows, closes [#442](https://github.com/tauri-apps/wry/pull/442) ([#640](https://github.com/tauri-apps/wry/pull/640)) on 2022-07-26
- Update tao to 0.13.0.
  - [935cc5f](https://github.com/tauri-apps/wry/commit/935cc5fe8b73055279dc107e71a10f2701ea8b3d) Update tao to 0.13 ([#642](https://github.com/tauri-apps/wry/pull/642)) on 2022-07-27

## \[0.19.0]

- - Automatically resize the webview on Windows to align with other platforms.
- **Breaking change**: Removed `WebView::resize`
- [d7c9097](https://github.com/tauri-apps/wry/commit/d7c9097256d76de7400032cf27acd7a1874da5cd) feat: auto resize webview on Windows ([#628](https://github.com/tauri-apps/wry/pull/628)) on 2022-06-27
- Implement new window requested handler
  - [fa5456c](https://github.com/tauri-apps/wry/commit/fa5456c6abe16be17073e75f4a0205966be266b2) feat: Implement new window requested event, closes [#527](https://github.com/tauri-apps/wry/pull/527) ([#526](https://github.com/tauri-apps/wry/pull/526)) on 2022-06-19
- Re-export `url::Url`.
  - [0cb6961](https://github.com/tauri-apps/wry/commit/0cb696119b5e25292af9595fd89856116520c049) fix: re-export `url::Url` ([#612](https://github.com/tauri-apps/wry/pull/612)) on 2022-06-17
- Update tao to 0.12
  - [448837e](https://github.com/tauri-apps/wry/commit/448837e795a8f7f8dc4ac5f34b27063b108fc1f2) Update tao to 0.12 ([#629](https://github.com/tauri-apps/wry/pull/629)) on 2022-06-28

## \[0.18.3]

- Update tao to 0.11
  - [f4b42fb](https://github.com/tauri-apps/wry/commit/f4b42fb412fa557188f20b72ef6c4314d1d6bb91) Update tao to v0.12 ([#609](https://github.com/tauri-apps/wry/pull/609)) on 2022-06-15

## \[0.18.2]

- Fix NSString can not be released.
  - [95ca52f](https://github.com/tauri-apps/wry/commit/95ca52f5d8ca86b64f8587a0f96cf0fb7dc22125) fix: NSString isn't released ([#604](https://github.com/tauri-apps/wry/pull/604)) on 2022-06-07

## \[0.18.1]

- Remove unused tray from doc features.
  - [5eecb00](https://github.com/tauri-apps/wry/commit/5eecb0074397efa40351b3caa8fd4a6d972c4c85) Remove unused tray from doc features ([#602](https://github.com/tauri-apps/wry/pull/602)) on 2022-05-31

## \[0.18.0]

- Remove trivial tray features.
  - [a3fea48](https://github.com/tauri-apps/wry/commit/a3fea48d2d78ebe4fa3f08b40d2c3c8c8135bb12) Remove trivial tray features ([#599](https://github.com/tauri-apps/wry/pull/599)) on 2022-05-31

## \[0.17.0]

- Add option to enable/disable zoom shortcuts for WebView2, disabled by default.
  - [494a110](https://github.com/tauri-apps/wry/commit/494a11057f9ddd2bf4bcecdc96b43ed95c5bd08e) WebView2: Enable/disable platform default zooming shortcuts, closes [#569](https://github.com/tauri-apps/wry/pull/569) ([#574](https://github.com/tauri-apps/wry/pull/574)) on 2022-05-15
- Prevent memory leak on macOS.
  - [16d1924](https://github.com/tauri-apps/wry/commit/16d192450ed639f94cf8b7137fa5fea1a319f8b5) fix: prevent memory leak on macOS, closes [#536](https://github.com/tauri-apps/wry/pull/536) ([#587](https://github.com/tauri-apps/wry/pull/587)) on 2022-05-20
- Update the `windows` crate to the latest 0.37.0 release and `webview2-com` to 0.16.0 to match.

The `#[implement]` macro in `windows-implement` and the `implement` feature in `windows` depend on some `const` generic features which stabilized in `rustc` 1.61. The MSRV on Windows targets is effectively 1.61, but other targets do not require these features.

The `webview2-com` crate specifies `rust-version = "1.61"`, so `wry` will inherit that MSRV and developers on Windows should get a clear error message telling them to update their toolchain when building `wry` or anything that depends on `wry`. Developers targeting other platforms should be able to continue using whatever toolchain they were using before.

- [9d9d9d8](https://github.com/tauri-apps/wry/commit/9d9d9d8f3d37a283bbb707d39c3aac090325a63e) Update windows-rs to 0.37.0 and webview2-com to 0.16.0 to match ([#592](https://github.com/tauri-apps/wry/pull/592)) on 2022-05-23

## \[0.16.2]

- Fixed build on macos.
  - [17ab12d](https://github.com/tauri-apps/wry/commit/17ab12ded27949474f687640faebb5cc376327c5) fix: fix build on macos, closes [#580](https://github.com/tauri-apps/wry/pull/580) ([#581](https://github.com/tauri-apps/wry/pull/581)) on 2022-05-10

## \[0.16.1]

- Fixes a crash on macOS below Big Sur due to `titlebarSeparatorStyle` (11+ API) usage.
  - [eb2dddb](https://github.com/tauri-apps/wry/commit/eb2dddb611f7fadf35bf7d7c32cb6d054da9fe9e) fix(macos): only use APIs when supported on 2022-05-08
- Only run `WebView::print` on macOS on v11+. This prevents a crash on older versions.
  - [eb2dddb](https://github.com/tauri-apps/wry/commit/eb2dddb611f7fadf35bf7d7c32cb6d054da9fe9e) fix(macos): only use APIs when supported on 2022-05-08

## \[0.16.0]

- Fixes a typo in the `WebviewExtMacOS` conditional compilation.
  - [10d7f03](https://github.com/tauri-apps/wry/commit/10d7f03f403e9c373fe80897308393e0bb67a06d) fix(macos): typo in the WebviewExtMacOS conditional compilation ([#568](https://github.com/tauri-apps/wry/pull/568)) on 2022-05-02
- Fixes a crash when the custom protocol response is empty on macOS.
  - [67809f4](https://github.com/tauri-apps/wry/commit/67809f4d8abe1a042b2cdb616b03f6a2c50652b8) fix(macos): crash when custom protocol response is empty ([#567](https://github.com/tauri-apps/wry/pull/567)) on 2022-05-01
- Add `WebView::zoom` method.
  - [34b6cbc](https://github.com/tauri-apps/wry/commit/34b6cbca76811966cedf8050ae0d0fa18c84aa34) feat: add feature to zoom webview contents, closes [#388](https://github.com/tauri-apps/wry/pull/388) ([#564](https://github.com/tauri-apps/wry/pull/564)) on 2022-05-02
- Set the titlebar separator style in macOS to `none`.
  - [9776fc4](https://github.com/tauri-apps/wry/commit/9776fc466b5f3a6ef47956ec5c9cdd9c5164046a) fix(macos): set titlebar style to `none` ([#566](https://github.com/tauri-apps/wry/pull/566)) on 2022-05-01
- Disable webview2 mini menu
  - [ed0b223](https://github.com/tauri-apps/wry/commit/ed0b2230c285991b7a4588c8045111f04a67a16f) fix: disable WebView2 mini menu ("OOUI"), closes [#535](https://github.com/tauri-apps/wry/pull/535) ([#559](https://github.com/tauri-apps/wry/pull/559)) on 2022-04-29

## \[0.15.1]

- Update how android handles url
  - [427cf92](https://github.com/tauri-apps/wry/commit/427cf9222d7152f911aa70eb778eb7aa90c83fac) Unify custom protocol across Android/iOS ([#546](https://github.com/tauri-apps/wry/pull/546)) on 2022-04-11
- Add devtools support on Android/iOS.
  - [1c5d77a](https://github.com/tauri-apps/wry/commit/1c5d77a8ce79e75705a71c659af86541d50c5007) Add devtools support on Android/iOS ([#548](https://github.com/tauri-apps/wry/pull/548)) on 2022-04-11
- Fix to reset process on MacOS when webview is closed, closes #536.
  - [fd1dcc3](https://github.com/tauri-apps/wry/commit/fd1dcc3cc5a290bfe4ae8de04064074109902432) fix: reset background process when webview is closed, closes [#536](https://github.com/tauri-apps/wry/pull/536) ([#556](https://github.com/tauri-apps/wry/pull/556)) on 2022-04-24

## \[0.15.0]

- On Windows and Linux, disable resizing maximized borderless windows.
  - [313eaea](https://github.com/tauri-apps/wry/commit/313eaea0ff123bddbc8b5c337ded05d464d3dfaa) fix(win,linux): disable resizing maximized borderless windows ([#533](https://github.com/tauri-apps/wry/pull/533)) on 2022-03-30
- Fixes a memory leak on the custom protocol response body on macOS.
  - [36b985e](https://github.com/tauri-apps/wry/commit/36b985e939f4769f9835b4865ee1013229ec7539) fix(macos): custom protocol memory leak ([#539](https://github.com/tauri-apps/wry/pull/539)) on 2022-04-03
- Update tao to v0.8.0.
  - [1c540b0](https://github.com/tauri-apps/wry/commit/1c540b01fa08e84c199b8ded726b6ec77b40f015) feat: update tao to 0.8, refactor tray features ([#541](https://github.com/tauri-apps/wry/pull/541)) on 2022-04-07
- The `tray` and `ayatana-tray` Cargo features are not enabled by default.
  - [1c540b0](https://github.com/tauri-apps/wry/commit/1c540b01fa08e84c199b8ded726b6ec77b40f015) feat: update tao to 0.8, refactor tray features ([#541](https://github.com/tauri-apps/wry/pull/541)) on 2022-04-07
- **Breaking change:** Renamed the `ayatana` Cargo feature to `ayatana-tray` and added the `gtk-tray` feature. The default tray on Linux is now `libayatana-appindicator`.
  - [1c540b0](https://github.com/tauri-apps/wry/commit/1c540b01fa08e84c199b8ded726b6ec77b40f015) feat: update tao to 0.8, refactor tray features ([#541](https://github.com/tauri-apps/wry/pull/541)) on 2022-04-07

## \[0.14.0]

- Added `close_devtools` function to `Webview`.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28
- Hide the devtool functions behind the `any(debug_assertions, feature = "devtools")` flag.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28
- **Breaking change:** Renamed the `devtool` function to `open_devtools`.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28
- Enable tab navigation on macOS.
  - [28ebedc](https://github.com/tauri-apps/wry/commit/28ebedc41f9017fed3fe1dc3a6d021c69f88ef5d) fix(macOS): enable tab navigation on all elements, fixes [#406](https://github.com/tauri-apps/wry/pull/406) ([#512](https://github.com/tauri-apps/wry/pull/512)) on 2022-03-03
- Added `is_devtools_open` function to `Webview`.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28
- - Expose methods to access the underlying native handles of the webview.
- **Breaking change**: `WebviewExtWindows::controller` now returns the controller directly and not wrapped in an `Option`
- [e54afec](https://github.com/tauri-apps/wry/commit/e54afec43b767ffdb43debbd526d249c3c5b5490) feat: expose webview native handles, closes [#495](https://github.com/tauri-apps/wry/pull/495) ([#513](https://github.com/tauri-apps/wry/pull/513)) on 2022-03-03
- Add navigation handler to decide if an url is allowed to navigate.
  - [aa8af02](https://github.com/tauri-apps/wry/commit/aa8af020ab9d88ad762f2facbfa368effb04f570) feat: Implement navigation event and cancellation, closes [#456](https://github.com/tauri-apps/wry/pull/456) ([#519](https://github.com/tauri-apps/wry/pull/519)) on 2022-03-18
- **Breaking change**: Renamed the `devtool` feature to `devtools`.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28
- **Breaking change:** Renamed the `with_dev_tool` function to `with_devtools`.
  - [bf3b710](https://github.com/tauri-apps/wry/commit/bf3b7107631f14567b0b5ff1947c2bff1ffa2603) feat: add function to close the devtool and check if it is opened ([#529](https://github.com/tauri-apps/wry/pull/529)) on 2022-03-28

## \[0.13.3]

- Fix rustdoc generation of Windows and Mac on docs.rs.
  - [327a019](https://github.com/tauri-apps/wry/commit/327a019a07fd10ca3a42ebfb8d9d626e3b91fd05) Fix rustdoc generation of Windows and Mac on docs.rs, fix [#503](https://github.com/tauri-apps/wry/pull/503) ([#507](https://github.com/tauri-apps/wry/pull/507)) on 2022-02-27

## \[0.13.2]

- Fix cross compilation from `macOS`.
  - [c97499f](https://github.com/tauri-apps/wry/commit/c97499fb078c7c65508bf2fa3502ef95c8114ef4) fix: cross compilation from macOS ([#498](https://github.com/tauri-apps/wry/pull/498)) on 2022-02-15
- Update `webview2-com` to 0.13.0, which bumps the WebView2 SDK to 1.0.1108.44 and improves cross-compilation support.

Targeting \*-pc-windows-gnu works now, but it has some [limitations](https://github.com/wravery/webview2-rs#cross-compilation).

- [24a443c](https://github.com/tauri-apps/wry/commit/24a443ca1d90ef091eaceb0ec61bcc648499b743) Add /.changes/webview2-com-0.13.0.md on 2022-02-14

## \[0.13.1]

- Add `devtool` feature flag and configuration option.
  - [d0f307b](https://github.com/tauri-apps/wry/commit/d0f307b218c3913520efbb378e9c01a526137fdd) feat: implement `devtools` API, closes [#287](https://github.com/tauri-apps/wry/pull/287) ([#486](https://github.com/tauri-apps/wry/pull/486)) on 2022-02-07

- Update the `webview2-com` crate 0.11.0:

- Fix silent build script errors related to unconfigured nuget in https://github.com/wravery/webview2-rs/pull/4

- Update the WebView2 SDK (not the runtime, just the API bindings) to the latest 1.0.1072.54 version

- [7d4eeb7](https://github.com/tauri-apps/wry/commit/7d4eeb744bf008e43c034e865b383ee4a330e77a) Update webview2-com to 0.11.0 ([#488](https://github.com/tauri-apps/wry/pull/488)) on 2022-02-06

## \[0.13.0]

- Update gtk to 0.15
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Add clipboard field in WebViewAttributes.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Ignore transparency on Windows 7 to prevent application crash.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Remove clipboard property for consistency across platforms.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Enable cookie persistence on Linux if the `data_directory` is provided.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Enable objc's exception features so they can be treated as panic message.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Add inner size method for webview. This can reflect correct size of webview on macOS.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Add "transparent" and "fullscreen" feature flags on macOS to toggle private API.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Implement WebContextImpl on mac to extend several callback lifetimes.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- The only thing that private mod shared does is re-export http mod to public,
  we can just pub mod http.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- - Fix hovering over an edge of undecorated window on Linux won't change cursor.
- Undecorated window can be resized using touch on Linux.
- [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Update webkit2gtk to 0.15
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Add `with_user_agent(&str)` to `WebViewBuilder`.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Replace all of the `winapi` crate references with the `windows` crate, and replace `webview2` and `webview2-sys` with `webview2-com` and `webview2-com-sys` built with the `windows` crate. The replacement bindings are in the `webview2-com-sys` crate, with `pub use` in the `webview2-com` crate. They can be shared with TAO.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Fix null pointer crash on `get_content` of web resource request. This is a temporary fix.
  We will switch it back once upstream is updated.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Update the `windows` crate to 0.25.0, which comes with pre-built libraries. WRY and Tao can both reference the same types directly from the `windows` crate instead of sharing bindings in `webview2-com-sys`.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Update the `windows` crate to 0.29.0 and `webview2-com` to 0.9.0.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05
- Update the `windows` crate to 0.30.0 and `webview2-com` to 0.10.0.
  - [219d20c](https://github.com/tauri-apps/wry/commit/219d20ce66a6bdf6c3e1af6156c9f2a74f2eed29) Merge next back to dev branch ([#477](https://github.com/tauri-apps/wry/pull/477)) on 2022-02-05

## \[0.12.2]

- Fixed a Linux multi-window issue where the internal url loader didn't unlock when flushed while empty
  - [5377821](https://github.com/tauri-apps/wry/commit/5377821f43c0e7556ec46f0aaf4d6b0637512493) Fix async multiwindow deadlock ([#382](https://github.com/tauri-apps/wry/pull/382)) on 2021-08-16

- The custom protocol now returns a `Request` and expects a `Response`.

- This allows us to get the complete request from the Webview. (Method, GET, POST, PUT etc..)
  Read the complete header.

- And allow us to be more flexible in the future without bringing breaking changes.

- [d202573](https://github.com/tauri-apps/wry/commit/d202573c2c68a2ff0411c1aa797ecc10f727e93b) refactor: Custom protocol request/response ([#387](https://github.com/tauri-apps/wry/pull/387)) on 2021-08-22

- On Linux, automation callbacks now use the first created webview as the return value
  - [f9d7049](https://github.com/tauri-apps/wry/commit/f9d7049978bbad389c99d7a7cce9903a528d871d) Use the first created webview for webkit2gtk automation callbacks ([#383](https://github.com/tauri-apps/wry/pull/383)) on 2021-08-16

## \[0.12.1]

- Add html attributes as another method to load the page. This can provide some other origin header and make CORS request
  possible.
  - [02ad372](https://github.com/tauri-apps/wry/commit/02ad37219a1f6e5e6ed8e4da61e6a5ac021d410e) feat: html string attributes ([#368](https://github.com/tauri-apps/wry/pull/368)) on 2021-08-12
- Shorter protocol name on Windows. This can make origin be shorter too.
  - [2d9f5c9](https://github.com/tauri-apps/wry/commit/2d9f5c95e3805911d12803122fd1e83be758a769) Shorter protocol name on Windows ([#367](https://github.com/tauri-apps/wry/pull/367)) on 2021-08-12

## \[0.12.0]

- Custom Protocol handlers no longer take a `&Window` parameter.
  - [0e2574c](https://github.com/tauri-apps/wry/commit/0e2574c420f778c59bafc164ddee2bc0b7705ee9) Remove `&Window` parameter from Custom Protocol handlers ([#361](https://github.com/tauri-apps/wry/pull/361)) on 2021-07-28
- Update gtk to version 0.14. This also remove requirement of `clang`.
  - [251a80b](https://github.com/tauri-apps/wry/commit/251a80bab49d42f742a3ae6b3ca2cbfc97de98bb) Update gtk to version 0.14 ([#364](https://github.com/tauri-apps/wry/pull/364)) on 2021-08-06
- Update tao to v0.5. Please see release notes on tao for more information.
  - [483bad0](https://github.com/tauri-apps/wry/commit/483bad0fc7e7564500f7183547c15604fa387258) feat: tao as window dependency ([#230](https://github.com/tauri-apps/wry/pull/230)) on 2021-05-03
  - [51430e9](https://github.com/tauri-apps/wry/commit/51430e97dfb6589c5ff71e5078438be67293d044) publish new versions ([#221](https://github.com/tauri-apps/wry/pull/221)) on 2021-05-09
  - [0cf0089](https://github.com/tauri-apps/wry/commit/0cf0089b6d49aa9e1a8c791ec8883fce48a0dfd1) Update tao to v0.2.6 ([#271](https://github.com/tauri-apps/wry/pull/271)) on 2021-05-18
  - [a76206c](https://github.com/tauri-apps/wry/commit/a76206c11fa0a4ba1d041aa0f25452dd80941ee9) publish new versions ([#272](https://github.com/tauri-apps/wry/pull/272)) on 2021-05-18
  - [3c4f8b8](https://github.com/tauri-apps/wry/commit/3c4f8b8b2bd42e7634b889aa5317d909bfce593c) Update tao to v0.5 ([#365](https://github.com/tauri-apps/wry/pull/365)) on 2021-08-09
- Add flags to support all other possible unix systems.
  - [c0d0a78](https://github.com/tauri-apps/wry/commit/c0d0a78b893eecdc45c6cda71264020d6ae17bda) Add flags to support all other unix systems. ([#352](https://github.com/tauri-apps/wry/pull/352)) on 2021-07-21
- Support having multiple webkit2gtk `WebView`s on a single `WebContext`.
  - [3f03d6b](https://github.com/tauri-apps/wry/commit/3f03d6b5ea4e9ba81950245de156f09e72ab40a1) Support multiple webviews on a single WebContext (webkit2gtk) ([#359](https://github.com/tauri-apps/wry/pull/359)) on 2021-07-28
- On Windows, Fix cursor flickering when Tao window is without decorations
  - [e28bcce](https://github.com/tauri-apps/wry/commit/e28bcce0884937365013fda3098f64f9956d569f) fix(windows): fix mouse style flicker when `decorations: false` ([#350](https://github.com/tauri-apps/wry/pull/350)) on 2021-07-20
- Remove winrt support since it's outdated for a long time. We will reimplement it again once `windws-rs` is stable!
  - [c37973e](https://github.com/tauri-apps/wry/commit/c37973e47318e9cff2712eb4a394c07734f58d54) chore(windows): remove winrt support ([#356](https://github.com/tauri-apps/wry/pull/356)) on 2021-07-24

## \[0.11.0]

- Allow resizing of borderless window on Windows
  - [bd10b8e](https://github.com/tauri-apps/wry/commit/bd10b8e5fe517edd6234ed03170741f1a51768bf) feat(Windows): resize borderless window ([#333](https://github.com/tauri-apps/wry/pull/333)) on 2021-07-15
- Mark enums as `#[non_exhaustive]` to prevent breaking changes on enum update.
  - [f07ae14](https://github.com/tauri-apps/wry/commit/f07ae144197933c28f8302105b313c2a2afc62af) refactor: add `#[non_exhaustive]` attributes to enums ([#304](https://github.com/tauri-apps/wry/pull/304)) on 2021-07-08
- Bump tao to `0.4`. Please refer to `tao` changelog for more details.
  - [6eb10d4](https://github.com/tauri-apps/wry/commit/6eb10d4e10ce86c8403c80fb41ba5e37072dc61e) bump `tao` to 0.4 and fix examples ([#329](https://github.com/tauri-apps/wry/pull/329)) on 2021-07-14
- - Add `focus` method to `Webview`
- Add `WebviewExtWindows` trait with `controller` method
- [621ed1f](https://github.com/tauri-apps/wry/commit/621ed1fff35d9389d88664d8084e1a678dfbfc36) feat: add `.focus()` to `Webview` ([#325](https://github.com/tauri-apps/wry/pull/325)) on 2021-07-05
- [96b7b94](https://github.com/tauri-apps/wry/commit/96b7b943da34ab81872553e65d2f2cd138531a62) Add controller method instead ([#326](https://github.com/tauri-apps/wry/pull/326)) on 2021-07-07
- macOS: Remove handler in the webview as it should be handled with the menu.
  - [5a9df15](https://github.com/tauri-apps/wry/commit/5a9df156f04789d4c89fdb8edf72b301667df127) fix(macos): Remove keypress handler in the webview for copy/paste/cut ([#328](https://github.com/tauri-apps/wry/pull/328)) on 2021-07-07
- Fixes multiple custom protocols registration on Windows.
  - [923d346](https://github.com/tauri-apps/wry/commit/923d3461ce93846af8dd548d4e43ebd0fd6111a3) fix(windows): multiple custom protocols, closes [#323](https://github.com/tauri-apps/wry/pull/323) ([#324](https://github.com/tauri-apps/wry/pull/324)) on 2021-07-02

## \[0.10.3]

- [#315](https://github.com/tauri-apps/wry/pull/315) fixed Webview2 runtime performance issues.
  - [d3c9b16](https://github.com/tauri-apps/wry/commit/d3c9b169d81fd8b79e6695d91b3a1d0e8042a81f) Fix Webview2 runtime performance issues ([#316](https://github.com/tauri-apps/wry/pull/316)) on 2021-06-29

## \[0.10.2]

- Fix file explorer getting blocked by automation.
  - [0c5cdd8](https://github.com/tauri-apps/wry/commit/0c5cdd8f2a6f4d07d87c6c4d1c51540ff9abfd97) Fix file explorer getting blocked by automation ([#310](https://github.com/tauri-apps/wry/pull/310)) on 2021-06-23

## \[0.10.1]

- `WebContext::set_allows_automation` is now available to specify if the context should allow automation (e.g. WebDriver).
  It is only enforced on Linux, but may expand platforms in the future.
  - [4ad0bf1](https://github.com/tauri-apps/wry/commit/4ad0bf12d186b3c313131060316aef371f45d455) move set_allows_automation to WebContext method ([#302](https://github.com/tauri-apps/wry/pull/302)) on 2021-06-21

## \[0.10.0]

- Add WebViewAttributes
  - [81f3218](https://github.com/tauri-apps/wry/commit/81f3218d9ac55a987b050f574774afcaa0b5c2f7) Add WebViewAttributes ([#286](https://github.com/tauri-apps/wry/pull/286)) on 2021-06-04
- Add `with_web_context` method that can work well with builder pattern.
  - [48f53a3](https://github.com/tauri-apps/wry/commit/48f53a3393b0c016a972a72dec45691959ac9e3b) Add `with_web_context` method ([#292](https://github.com/tauri-apps/wry/pull/292)) on 2021-06-13
- Change the custom protocol handler on macOS so it returns a response on error and a status code on success.
  - [6b869b1](https://github.com/tauri-apps/wry/commit/6b869b1ad5de9c8e9f36c1fc1b7040e10b033b52) fix(macos): custom protocol response with status code + error response ([#279](https://github.com/tauri-apps/wry/pull/279)) on 2021-05-20
- Update signature of custom protocol closure. It should return a mime type string now.
  - [cc9fc4b](https://github.com/tauri-apps/wry/commit/cc9fc4b43df79834c1b8f2c1347accba50356604) Add mimetype to return type of custom protocol ([#296](https://github.com/tauri-apps/wry/pull/296)) on 2021-06-13
- Fix webview creation when using new_any_thread of event loop.
  - [4d62cf5](https://github.com/tauri-apps/wry/commit/4d62cf5a3ddcbed06afb93d9503424a9b8110d57) Fix webview creation when using new_any_thread on Windows ([#298](https://github.com/tauri-apps/wry/pull/298)) on 2021-06-18
- Remove `Dispatcher`, `dispatch_script` and `dispatcher` in the `webview` module and add a `js` parameter to `evaluate_script`.
  - [de4a5fa](https://github.com/tauri-apps/wry/commit/de4a5fa820b1938532223677913e73720885cb54) refactor: remove `Dispatcher` and related methods, closes [#290](https://github.com/tauri-apps/wry/pull/290) ([#291](https://github.com/tauri-apps/wry/pull/291)) on 2021-06-09
- Removes the `image` dependency.
  - [1d5cc59](https://github.com/tauri-apps/wry/commit/1d5cc590856e1be1428f8516595ace6d8099f41f) chore(deps): remove `image` dependency ([#274](https://github.com/tauri-apps/wry/pull/274)) on 2021-05-19
- Bump tao to `0.3` and add more examples.

*For more details, please refer to `tao` changelog.*

- [cd4697e](https://github.com/tauri-apps/wry/commit/cd4697ebdb8eb955f0ed2be4aefea82d2c263a52) bump `tao` to 0.3 with examples ([#294](https://github.com/tauri-apps/wry/pull/294)) on 2021-06-21
- Add `wry::webview::WebContext`. It's now a required argument on `WebViewBuilder::build`.
  - [761b2b5](https://github.com/tauri-apps/wry/commit/761b2b59fe0434b3458d99ed599394af0e1e3962) webdriver support ([#281](https://github.com/tauri-apps/wry/pull/281)) on 2021-06-08

## \[0.9.4]

- Update tao to v0.2.6
  - [483bad0](https://github.com/tauri-apps/wry/commit/483bad0fc7e7564500f7183547c15604fa387258) feat: tao as window dependency ([#230](https://github.com/tauri-apps/wry/pull/230)) on 2021-05-03
  - [51430e9](https://github.com/tauri-apps/wry/commit/51430e97dfb6589c5ff71e5078438be67293d044) publish new versions ([#221](https://github.com/tauri-apps/wry/pull/221)) on 2021-05-09
  - [0cf0089](https://github.com/tauri-apps/wry/commit/0cf0089b6d49aa9e1a8c791ec8883fce48a0dfd1) Update tao to v0.2.6 ([#271](https://github.com/tauri-apps/wry/pull/271)) on 2021-05-18

## \[0.9.3]

- Expose `webview_version` function in the `webview` module.
  - [4df310e](https://github.com/tauri-apps/wry/commit/4df310e6bb508854ffc17ec915b3d0ab7c11f03d) feat: get webview version ([#259](https://github.com/tauri-apps/wry/pull/259)) on 2021-05-12
- Add print method on Linux and Windows.
  - [54c5ec7](https://github.com/tauri-apps/wry/commit/54c5ec7ae6166da5ce670ccd2ceaa108233bb845) Implement print method on Linux and Windows ([#264](https://github.com/tauri-apps/wry/pull/264)) on 2021-05-17
- Disable smooth scrolling on Linux to match behaviour on browsers.
  - [3e786bb](https://github.com/tauri-apps/wry/commit/3e786bb28793e939c00ebf0c6758d4f6cf4d3b28) Disable smooth scrolling on Linux ([#268](https://github.com/tauri-apps/wry/pull/268)) on 2021-05-17

## \[0.9.2]

- Add `tray` feature flag from tao.
  - [093c25e](https://github.com/tauri-apps/wry/commit/093c25ee68d51849b95a1a3b9341e5ad6021cecf) feat: expose tray feature flag ([#256](https://github.com/tauri-apps/wry/pull/256)) on 2021-05-10

## \[0.9.1]

- Correctly set visibility when building `Window` on gtk-backend
  - [4395ad1](https://github.com/tauri-apps/wry/commit/4395ad147b799e67f9802c499346d0ad53554317) fix: only call `show_all` when needed ([#227](https://github.com/tauri-apps/wry/pull/227)) on 2021-05-02
- Fix `macOS` cursors and other minors UI glitch.
  - [d550b2f](https://github.com/tauri-apps/wry/commit/d550b2f0a1c708747537e3a5e6d880fea00e651d) fix(macOS): Window layers ([#220](https://github.com/tauri-apps/wry/pull/220)) on 2021-04-28
- Expose `print()` function to the webview. Work only on macOS for now.
  - [5206db6](https://github.com/tauri-apps/wry/commit/5206db6ca599fe0e146d72b04c908330e3045838) fix(macOS): Printing ([#235](https://github.com/tauri-apps/wry/pull/235)) ([#236](https://github.com/tauri-apps/wry/pull/236)) on 2021-05-06
- Fix macOS windows order for tray (statusbar) applications.
  - [229275f](https://github.com/tauri-apps/wry/commit/229275f106371d79800e0ca1cbc7b6c1827bc2ac) fix: macOS windows order ([#242](https://github.com/tauri-apps/wry/pull/242)) on 2021-05-07
- Add `request_redraw` method of `Window` on Linux
  - [03abfa0](https://github.com/tauri-apps/wry/commit/03abfa06019a78a182c7cd29dc63bf3d9df10e44) Add request_redraw method on Linux ([#222](https://github.com/tauri-apps/wry/pull/222)) on 2021-04-30
- Add tao as window dependency.
  - [483bad0](https://github.com/tauri-apps/wry/commit/483bad0fc7e7564500f7183547c15604fa387258) feat: tao as window dependency ([#230](https://github.com/tauri-apps/wry/pull/230)) on 2021-05-03
- Close the window when the instance is dropped on Linux and Windows.
  - [3f2cc28](https://github.com/tauri-apps/wry/commit/3f2cc28b4fbfcf54c97000a6541e9356440838e8) fix: close window when the instance is dropped ([#228](https://github.com/tauri-apps/wry/pull/228)) on 2021-05-02
- Remove winit dependency on Linux
  - [fa15076](https://github.com/tauri-apps/wry/commit/fa15076207d9e678db4149210aba929044d0ff45) feat: winit interface for gtk ([#163](https://github.com/tauri-apps/wry/pull/163)) on 2021-04-19
  - [39d6f59](https://github.com/tauri-apps/wry/commit/39d6f595d81c857e92aef31cc2559b402e64edd3) publish new versions ([#166](https://github.com/tauri-apps/wry/pull/166)) on 2021-04-29
  - [4ef8330](https://github.com/tauri-apps/wry/commit/4ef8330d856e07d34bf86d1f2903c82c37042556) Remove winit dependency on Linux ([#226](https://github.com/tauri-apps/wry/pull/226)) on 2021-04-30

## \[0.9.0]

- Refactor signatures of most closure types
  - [b8823fe](https://github.com/tauri-apps/wry/commit/b8823fe14ee5f95d07cd2cb1f9f673b964c9dc83) refactor: signature of closure types ([#167](https://github.com/tauri-apps/wry/pull/167)) on 2021-04-19
- Drop handler closures properly on macOS.
  - [f905503](https://github.com/tauri-apps/wry/commit/f905503c4a010ed4219c6ad36d14c0dbf0b6e122) fix: [#160](https://github.com/tauri-apps/wry/pull/160) drop handler closures properly ([#211](https://github.com/tauri-apps/wry/pull/211)) on 2021-04-27
- Fix `history.pushState` in webview2.
  - [dd0fa46](https://github.com/tauri-apps/wry/commit/dd0fa46494c1ab8536bcc7ea1dd16341b12856b4) Use http instead of file for windows custom protocol workaround ([#173](https://github.com/tauri-apps/wry/pull/173)) on 2021-04-20
- The `data_directory` field now affects the IndexedDB and LocalStorage directories on Linux.
  - [1a6c821](https://github.com/tauri-apps/wry/commit/1a6c8216ee6865ca14025c229b37342496b38f26) feat(linux): implement custom user data path ([#188](https://github.com/tauri-apps/wry/pull/188)) on 2021-04-22
- Fix runtime panic on macOS, when no file handler are defined.
  - [22a4991](https://github.com/tauri-apps/wry/commit/22a4991aa8ca7c75aa52150a90379c40bcc34d07) bug(macOS): Runtime panic when no file_drop_handler ([#177](https://github.com/tauri-apps/wry/pull/177)) on 2021-04-20
- Add position field on WindowAttribute
  - [2b3be7a](https://github.com/tauri-apps/wry/commit/2b3be7a4db2cbc1612c7105cb698c1f21a05da77) Add position field on WindowAttribute ([#219](https://github.com/tauri-apps/wry/pull/219)) on 2021-04-28
- Fix panic on multiple custom protocols registration.
  - [01647a2](https://github.com/tauri-apps/wry/commit/01647a2a5b769bc192754c2d3806a55112d58d33) Fix custom protocol registry on mac ([#205](https://github.com/tauri-apps/wry/pull/205)) on 2021-04-26
- Fix SVG render with the custom protocol.
  - [890cfe5](https://github.com/tauri-apps/wry/commit/890cfe527996c181d643c9f8e5fc3e79ff0841a0) fix(custom-protocol): SVG mime type - close [#168](https://github.com/tauri-apps/wry/pull/168) ([#169](https://github.com/tauri-apps/wry/pull/169)) on 2021-04-19
- Initial custom WindowExtWindows trait.
  - [1ef1f58](https://github.com/tauri-apps/wry/commit/1ef1f58efb6afa6c6b9eda3a43ee83fc79c3b78e) feat: custom WindowExtWindow trait ([#191](https://github.com/tauri-apps/wry/pull/191)) on 2021-04-23
- Fix transparency on Windows
  - [e278556](https://github.com/tauri-apps/wry/commit/e2785566c69d43f003896b7b5da79b29d2966c13) fix: transparency on Windows  ([#217](https://github.com/tauri-apps/wry/pull/217)) on 2021-04-28
- Add platform module and WindowExtUnix trait on Linux
  - [004e298](https://github.com/tauri-apps/wry/commit/004e298e0198e6576a11e6e84fdf6b7c2f66b6ae) feat: WindowExtUnix trait ([#192](https://github.com/tauri-apps/wry/pull/192)) on 2021-04-23
- Make sure custom protocol on Windows is over HTTPS.
  - [c36db35](https://github.com/tauri-apps/wry/commit/c36db35b2b8704eb36bc341cd99abac01abfab87) fix(custom-protocol): Make sure custom protocol on Windows is over HTTPS. ([#179](https://github.com/tauri-apps/wry/pull/179)) on 2021-04-20
- Initial winit interface for gtk backend
  - [fa15076](https://github.com/tauri-apps/wry/commit/fa15076207d9e678db4149210aba929044d0ff45) feat: winit interface for gtk ([#163](https://github.com/tauri-apps/wry/pull/163)) on 2021-04-19

## \[0.8.0]

- Wry now accepts multiple custom protocol registrations.
  - [db64fc6](https://github.com/tauri-apps/wry/commit/db64fc69c48a728184fcef001688b94f0294edab) feat/licenses ([#155](https://github.com/tauri-apps/wry/pull/155)) on 2021-04-14
- Apply license header for SPDX compliance.
  - [05e0218](https://github.com/tauri-apps/wry/commit/05e02180c9fe929d3e691185df44257654546935) feat: multiple custom protocols ([#151](https://github.com/tauri-apps/wry/pull/151)) on 2021-04-11
  - [db64fc6](https://github.com/tauri-apps/wry/commit/db64fc69c48a728184fcef001688b94f0294edab) feat/licenses ([#155](https://github.com/tauri-apps/wry/pull/155)) on 2021-04-14
- Remove bindings crate and use windows-webview2 as dependency instead.
  - [c2156a4](https://github.com/tauri-apps/wry/commit/c2156a45d7fbfead956b6d03b2594962e3455e6d) Move to windows-webview2 as dependency for winrt impl ([#144](https://github.com/tauri-apps/wry/pull/144)) on 2021-04-03

## \[0.7.0]

- Add old win32 implementation on windows as default feature flag.
  - [1a88cd2](https://github.com/tauri-apps/wry/commit/1a88cd267f2a29c1dd35d7197250972718081847) refactor: Add win32 implementation and feature flag for both backends ([#139](https://github.com/tauri-apps/wry/pull/139)) on 2021-04-02
- Adds a `WindowProxy` to the file drop handler closure - `WindowFileDropHandler`.
  - [20cb051](https://github.com/tauri-apps/wry/commit/20cb051aba28009c70dad838b2a9b1575cb5363a) feat: add WindowProxy to file drop handler closure ([#140](https://github.com/tauri-apps/wry/pull/140)) on 2021-04-01

## \[0.6.2]

- Add pipe back to version check for covector config. This prevents the CI failure on publish if it exists already. The issue was patched in covector (and tests in place so it doesn't break in the future).
  - [a32829c](https://github.com/tauri-apps/wry/commit/a32829c527f02b228fa1da45e9710941c5415bfc) chore: add pipe for publish check back in ([#131](https://github.com/tauri-apps/wry/pull/131)) on 2021-03-28
- Fix messages to the webview from the backend being delayed on Linux/GTK when the user is not actively engaged with the UI.
  - [d2a2a9f](https://github.com/tauri-apps/wry/commit/d2a2a9f473d2588b27a95bf627d125caea1b979d) fix: spawn async event loop on gtk to prevent delayed messages ([#135](https://github.com/tauri-apps/wry/pull/135)) on 2021-03-31
- Add draggable regions, just add `drag-region` class to the html element.
  - [b2a0bfc](https://github.com/tauri-apps/wry/commit/b2a0bfc289786d0a23dac0c8d9543771e70e3427) feat/ draggable-region ([#92](https://github.com/tauri-apps/wry/pull/92)) on 2021-03-25
- Add event listener in application proxy
  - [c49846c](https://github.com/tauri-apps/wry/commit/c49846cfc41bb548a685edeac5f8036501f7dcec) feat: event listener ([#129](https://github.com/tauri-apps/wry/pull/129)) on 2021-03-26
- Better result error handling
  - [485035f](https://github.com/tauri-apps/wry/commit/485035f17d28560966b07b512935821814f0e951) chore: better result error handling ([#124](https://github.com/tauri-apps/wry/pull/124)) on 2021-03-21
- Fix visibility on webview2 when window was invisible previously and then shown.
  - [6d31706](https://github.com/tauri-apps/wry/commit/6d31706a6bff43e9b28100675cf8fc12f29db248) Fix visibility on webview2 when window was invisible previously ([#128](https://github.com/tauri-apps/wry/pull/128)) on 2021-03-24

## \[0.6.1]

- Add attribute option to allow WebView on Windows use user_data folder
  - [8dd58ee](https://github.com/tauri-apps/wry/commit/8dd58eec77d4c89491b1af427d06c4ee6cfa8e58) feat/ allow webview2 (windows) to use optional user_data folder provided by the attributes ([#120](https://github.com/tauri-apps/wry/pull/120)) on 2021-03-21

## \[0.6.0]

- Initialize covector!
  - [33b64ed](https://github.com/tauri-apps/wry/commit/33b64ed5c208b778d03dbb5f3f2808bb417c9f52) chore: covector init ([#55](https://github.com/tauri-apps/wry/pull/55)) on 2021-02-21
- Support Windows 7, 8, and 10
  - [fbf0d17](https://github.com/tauri-apps/wry/commit/fbf0d17164da455400aaa44104c3925eded09393) Adopt Webview2 on Windows ([#48](https://github.com/tauri-apps/wry/pull/48)) on 2021-02-20
- Dev tools are enabled on debug build
- Add skip task bar option
  - [395b6fb](https://github.com/tauri-apps/wry/commit/395b6fbcd66f6cbd0457cb609bea4afe734fadd4) feat: `skip_taskbar` for windows ([#49](https://github.com/tauri-apps/wry/pull/49)) on 2021-02-20
- Add custom protocol option
  - [a492806](https://github.com/tauri-apps/wry/commit/7a492806d716a30abe15a2104b64152c1ca370bb) Add custom protocol ([#65](https://github.com/tauri-apps/wry/pull/65)) on 2021-02-23
- Add transparent option to mac and linux
- Error type has Send/Sync traits
  - [3536b83](https://github.com/tauri-apps/wry/commit/3536b831ec30ee7436616ba4b262bbdd1e6279c8) Add .changes file in prepare of v0.6 on 2021-02-24
- Replace Callback with RPC handler
  - [e215157](https://github.com/tauri-apps/wry/commit/e215157146f0eab8ee6beab0628b036c68eea108) Implement draft RPC API ([#95](https://github.com/tauri-apps/wry/pull/95)) on 2021-03-04
- Add File drop handlers
  - [fed0ee7](https://github.com/tauri-apps/wry/commit/fed0ee772100ad19a344a85266618c7bcf7cb649) File drop handlers ([#96](https://github.com/tauri-apps/wry/pull/96)) on 2021-03-09
