# apple_codesign

This directory contains code vendored from the [`apple-codesign`](https://crates.io/crates/apple-codesign)
crate, version 0.29.0, part of [apple-platform-rs](https://github.com/indygreg/apple-platform-rs).
Copyright Gregory Szorc and contributors.

Only the self-signed code signing certificate generation used by `tauri-macos-sign` was kept,
the rest of the crate was removed. The upstream crate has not been released since 0.29.0 and
depending on it blocked dependency updates elsewhere in Tauri.

## License

The files vendored from upstream (`certificate.rs` and `error.rs`) are licensed under the
[Mozilla Public License 2.0](https://mozilla.org/MPL/2.0/), as in the original crate, and each
keeps its original MPL-2.0 notice. Modifications made to those files are also MPL-2.0.

`mod.rs` and the rest of `tauri-macos-sign` are licensed under Apache-2.0 OR MIT. MPL-2.0
allows its files to be combined with code under other licenses in a larger work, so the crate
as a whole is `(Apache-2.0 OR MIT) AND MPL-2.0`.
