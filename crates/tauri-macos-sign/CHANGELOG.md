# Changelog

## \[2.3.2]

### Dependencies

- [`514cf21e1`](https://www.github.com/tauri-apps/tauri/commit/514cf21e1417c7a78a0db494f891ba79d948b73d) ([#14591](https://www.github.com/tauri-apps/tauri/pull/14591) by [@Tunglies](https://www.github.com/tauri-apps/tauri/../../Tunglies)) Update num-bigint-dig from 0.8.4 to 0.8.6

## \[2.3.1]

### Performance Improvements

- [`ee3cc4a91`](https://www.github.com/tauri-apps/tauri/commit/ee3cc4a91bf1315ecaefe90f423ffd55ef6c40db) ([#14475](https://www.github.com/tauri-apps/tauri/pull/14475) by [@Tunglies](https://www.github.com/tauri-apps/tauri/../../Tunglies)) perf: remove needless clones in various files for improved performance. No user facing changes.

## \[2.3.0]

### Enhancements

- [`f59bf9d53`](https://www.github.com/tauri-apps/tauri/commit/f59bf9d5392ffd209e26ce5259c26d1acc31c4ba) ([#14337](https://www.github.com/tauri-apps/tauri/pull/14337) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) **Potentially breaking change:** Export custom Error enum instead of using anyhow. The changes happened in https://github.com/tauri-apps/tauri/pull/14126.

## \[2.2.0]

### New Features

- [`a9ec12843`](https://www.github.com/tauri-apps/tauri/commit/a9ec12843aa7d0eb774bd3a53e2e63da12cfa77b) ([#13521](https://www.github.com/tauri-apps/tauri/pull/13521) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Added a `--skip-stapling` option to make `tauri build|bundle` *not* wait for notarization to finish on macOS.

## \[2.1.0]

### Enhancements

- [`d6520a21c`](https://www.github.com/tauri-apps/tauri/commit/d6520a21ce02c3e2be2955999946c2cb7bdb07aa) ([#12541](https://www.github.com/tauri-apps/tauri/pull/12541) by [@FabianLars](https://www.github.com/tauri-apps/tauri/../../FabianLars)) Updated `wry` to 0.50, `windows` to 0.60, `webview2-com` to 0.36, and `objc2` to 0.6. This can be a **breaking change** if you use the `with_webview` API!

## \[2.0.1]

### What's Changed

- [`0ab2b3306`](https://www.github.com/tauri-apps/tauri/commit/0ab2b330644b6419f6cee1d5377bfb5cdda2ccf9) ([#11205](https://www.github.com/tauri-apps/tauri/pull/11205) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

### What's Changed

- [`382ed482b`](https://www.github.com/tauri-apps/tauri/commit/382ed482bd08157c39e62f9a0aaad8802f1092cb) Bump MSRV to 1.78.
- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

## \[0.1.1-rc.1]

### Enhancements

- [`f5d61822b`](https://www.github.com/tauri-apps/tauri/commit/f5d61822bf5988827776dd58bed75c19364e86bd) ([#11184](https://www.github.com/tauri-apps/tauri/pull/11184) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Added `Keychain::with_certificate_file` and `certificate::generate_self_signed`.

## \[0.1.1-rc.0]

### Bug Fixes

- [`1b0c447fc`](https://www.github.com/tauri-apps/tauri/commit/1b0c447fcbc424e08e4260277ec178df86f45d1d) ([#10654](https://www.github.com/tauri-apps/tauri/pull/10654) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Fixes output not visible when running on Node.js via NAPI.

## \[0.1.0-beta.0]

### New Features

- [`7c7fa0964`](https://www.github.com/tauri-apps/tauri/commit/7c7fa0964db3403037fdb9a34de2b877ddb8df1c) ([#9963](https://www.github.com/tauri-apps/tauri/pull/9963) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Initial release.
