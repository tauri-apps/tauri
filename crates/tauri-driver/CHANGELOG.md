# Changelog

## \[2.0.1]

### What's Changed

- [`0ab2b3306`](https://www.github.com/tauri-apps/tauri/commit/0ab2b330644b6419f6cee1d5377bfb5cdda2ccf9) ([#11205](https://www.github.com/tauri-apps/tauri/pull/11205) by [@lucasfernog](https://www.github.com/tauri-apps/tauri/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

### What's Changed

- [`382ed482b`](https://www.github.com/tauri-apps/tauri/commit/382ed482bd08157c39e62f9a0aaad8802f1092cb) Bump MSRV to 1.78.
- [`637285790`](https://www.github.com/tauri-apps/tauri/commit/6372857905ae9c0aedb7f482ddf6cf9f9836c9f2) Promote to v2 stable!

## \[0.1.5]

### Bug Fixes

- [`5f64ed2b7`](https://www.github.com/tauri-apps/tauri/commit/5f64ed2b78201b7e379b6234f7a799d9695b11d7) ([#10738](https://www.github.com/tauri-apps/tauri/pull/10738) by [@chippers](https://www.github.com/tauri-apps/tauri/../../chippers)) support both 1.x and 2.x automation env vars in `tauri-driver`

### What's Changed

- [`f4d5241b3`](https://www.github.com/tauri-apps/tauri/commit/f4d5241b377d0f7a1b58100ee19f7843384634ac) ([#10731](https://www.github.com/tauri-apps/tauri/pull/10731) by [@amrbashir](https://www.github.com/tauri-apps/tauri/../../amrbashir)) Update documentation icon path.

## \[0.1.4]

### New Features

- [`435d7513`](https://www.github.com/tauri-apps/tauri/commit/435d7513e45eab8b512e9a7e695a1adef8a98a46)([#8609](https://www.github.com/tauri-apps/tauri/pull/8609)) Added `webviewOptions` object to the `tauri:options` capability to configure the [Edge webview options](https://learn.microsoft.com/en-us/microsoft-edge/webdriver-chromium/capabilities-edge-options#webviewoptions-object) on Windows.

## \[0.1.3]

### What's Changed

- [`9edebbba`](https://www.github.com/tauri-apps/tauri/commit/9edebbba4ec472772b2f6307232e8d256f62c8ba)([#7475](https://www.github.com/tauri-apps/tauri/pull/7475)) Update locked dependencies to fix a Windows build issue when using them with a recent Rust compiler.
- [`9edebbba`](https://www.github.com/tauri-apps/tauri/commit/9edebbba4ec472772b2f6307232e8d256f62c8ba)([#7475](https://www.github.com/tauri-apps/tauri/pull/7475)) Bump minimum Rust version to `1.60` to be in line with the rest of the Tauri project.

## \[0.1.2]

- Expose `native-host` option in tauri-driver and set default to `127.0.0.1`.
  - [cd9dfc7b](https://www.github.com/tauri-apps/tauri/commit/cd9dfc7b9a3fe0e04e40d9b0f9be674aefd0d725) fix(driver): expose native-host option and set default to 127.0.0.1 ([#3816](https://www.github.com/tauri-apps/tauri/pull/3816)) on 2022-03-31

## \[0.1.1]

- The minimum Rust version is now `1.56`.
  - [a9dfc015](https://www.github.com/tauri-apps/tauri/commit/a9dfc015505afe91281c2027954ffcc588b1a59c) feat: update to edition 2021 and set minimum rust to 1.56 ([#2789](https://www.github.com/tauri-apps/tauri/pull/2789)) on 2021-10-22
- Add `args` field (array of application CLI arguments) to the `tauri:options` capabilities.
  - [d0970e34](https://www.github.com/tauri-apps/tauri/commit/d0970e3499297a6c102a36f2dc479d3d657bfaf3) feat(driver): add `args` to `tauri:options` ([#3154](https://www.github.com/tauri-apps/tauri/pull/3154)) on 2022-01-03

## \[0.1.0]

- Initial release including Linux and Windows support.
  - [be76fb1d](https://www.github.com/tauri-apps/tauri/commit/be76fb1dfe73a1605cc2ad246418579f4c2e1999) WebDriver support ([#1972](https://www.github.com/tauri-apps/tauri/pull/1972)) on 2021-06-23
  - [c22e5a7c](https://www.github.com/tauri-apps/tauri/commit/c22e5a7c2ebede41657973b80eff6b68106817fc) fix(covector): keep `tauri-driver` version as alpha on 2021-06-23
  - [b4426eda](https://www.github.com/tauri-apps/tauri/commit/b4426eda9e64fcdd25a2d72e548b8b0fbfa09619) Revert "WebDriver support ([#1972](https://www.github.com/tauri-apps/tauri/pull/1972))" on 2021-06-23
  - [4b2aa356](https://www.github.com/tauri-apps/tauri/commit/4b2aa35684632ed2afd7dec4ad848df5704868e4) Add back WebDriver support ([#2324](https://www.github.com/tauri-apps/tauri/pull/2324)) on 2021-08-01
