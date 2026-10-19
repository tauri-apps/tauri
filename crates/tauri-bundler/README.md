# Tauri Bundler

Wrap Rust executables in OS-specific app bundles.

## About

This is a fork of the awesome [cargo-bundle](https://github.com/burtonageo/cargo-bundle), turned into a library used by the [Tauri CLI](../tauri-cli).

### Stability

As it's intended to be used primarily in `tauri-cli`, the public API does not strictly adhere to SemVer. For example, minor releases may add new struct fields and change or remove Error enum variants.

## Configuration

The Tauri CLI maps configuration from `tauri.conf.json` into this library's settings, but the
library does not rely on that file and can be used by non-Tauri apps.

### General settings

These settings apply to bundles for all (or most) OSes.

- `product_name`: The name of the built application. The Tauri CLI maps the `productName` configuration
  value to this field and falls back to the `name` value from your `Cargo.toml` file.
- `identifier`: [REQUIRED] A string that uniquely identifies your application,
  in reverse-DNS form (for example, `"com.example.appname"` or
  `"io.github.username.project"`). For OS X and iOS, this is used as the
  bundle's `CFBundleIdentifier` value; for Windows, this is hashed to create
  an application GUID.
- `icon`: [OPTIONAL] The icons used for your application. This should be an array of file paths or globs (with images
  in various sizes/formats); `tauri-bundler` will automatically convert between image formats as necessary for
  different platforms. Supported formats include ICNS, ICO, PNG, and anything else that can be decoded by the
  [`image`](https://crates.io/crates/image) crate. Icons intended for high-resolution (e.g. Retina) displays
  should have a filename with `@2x` just before the extension (see example below).
- `version`: [OPTIONAL] The version of the application. If this is not present, then it will use the `version`
  value from your `Cargo.toml` file.
- `resources`: [OPTIONAL] List of files or directories which will be copied to the resources section of the
  bundle. Globs are supported.
- `copyright`: [OPTIONAL] This contains a copyright string associated with your application.
- `category`: [OPTIONAL] What kind of application this is. This can
  be a human-readable string (e.g. `"Puzzle game"`), or a Mac OS X
  LSApplicationCategoryType value
  (e.g. `"public.app-category.puzzle-games"`), or a GNOME desktop
  file category name (e.g. `"LogicGame"`), and `tauri-bundler` will
  automatically convert as needed for different platforms.
- `short_description`: [OPTIONAL] A short, one-line description of the application. If this is not present, then it
  will use the `description` value from your `Cargo.toml` file.
- `long_description`: [OPTIONAL] A longer, multi-line description of the application.

### Debian-specific settings

These settings are used only when bundling `deb` packages.

- `depends`: A list of strings indicating other packages (e.g. shared
  libraries) that this package depends on to be installed. If present, this
  forms the `Depends:` field of the `deb` package control file.

### Mac OS X-specific settings

These settings are used only when bundling `app` and `dmg` packages.

- `frameworks`: A list of strings indicating any Mac OS X frameworks that
  need to be bundled with the app. Each string can either be the name of a
  framework (without the `.framework` extension, e.g. `"SDL2"`), in which case
  `tauri-bundler` will search for that framework in the standard install
  locations (`~/Library/Frameworks/`, `/Library/Frameworks/`, and
  `/Network/Library/Frameworks/`), or a path to a specific framework bundle
  (e.g. `./data/frameworks/SDL2.framework`). Note that this setting just makes
  `tauri-bundler` copy the specified frameworks into the OS X app bundle (under
  `Foobar.app/Contents/Frameworks/`); you are still responsible for (1)
  arranging for the compiled binary to link against those frameworks (e.g. by
  emitting lines like `cargo:rustc-link-lib=framework=SDL2` from your
  `build.rs` script), and (2) embedding the correct rpath in your binary
  (e.g. by running `install_name_tool -add_rpath
"@executable_path/../Frameworks" path/to/binary` after compiling).
- `minimum_system_version`: A version string indicating the minimum Mac OS
  X version that the bundled app supports (e.g. `"10.11"`). If you are using
  this config field, you may also want have your `build.rs` script emit
  `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11` (or whatever version number
  you want) to ensure that the compiled binary has the same minimum version.
- `license`: Path to the license file for the DMG bundle.
- `exception_domain`: The exception domain to use on the macOS .app bundle. Allows communication to the outside world e.g. a web server you're shipping.
- `provider_short_name`: If your Apple ID is connected to multiple teams, you have to specify the provider short name of the team you want to use to notarize your app. See [Customizing the notarization workflow](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution/customizing_the_notarization_workflow) and search for `--list-providers` for more information how to obtain your provider short name.

### Example `tauri.conf.json`:

```json
{
  "productName": "Your Awesome App",
  "version": "0.1.0",
  "identifier": "com.my.app",
  "app": {},
  "bundle": {
    "active": true,
    "shortDescription": "",
    "longDescription": "",
    "copyright": "Copyright (c) You 2021. All rights reserved.",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": ["./assets/**/*.png"],
    "deb": {
      "depends": ["debian-dependency1", "debian-dependency2"]
    },
    "macOS": {
      "frameworks": [],
      "minimumSystemVersion": "10.11",
      "license": "./LICENSE"
    },
    "externalBin": ["./sidecar-app"]
  }
}
```

## Runtime paths

`tauri-bundler` creates the platform package and copies configured resources into it. It does not
provide a runtime context or automatically resolve paths for the packaged application.

### Bundled resources

Applications that do not use the Tauri runtime can use
[`tauri_utils::platform::resource_dir`](https://docs.rs/tauri-utils/latest/tauri_utils/platform/fn.resource_dir.html)
to locate the resource root. Pass the same product name used in
[`PackageSettings::product_name`](https://docs.rs/tauri-bundler/latest/tauri_bundler/bundle/struct.PackageSettings.html#structfield.product_name)
(the `productName` value in a Tauri configuration), because Linux uses it as the resource directory
name.

Add the path helpers as direct dependencies:

```toml
[dependencies]
dirs = "6"
tauri-utils = "2"
```

Then resolve resources at runtime and append the target path from `resources` or `resources_map`:

```rust
use std::path::{Path, PathBuf};
use tauri_utils::{platform::resource_dir, Env, PackageInfo};

fn bundled_resource(path: impl AsRef<Path>) -> tauri_utils::Result<PathBuf> {
  let package_info = PackageInfo {
    // Must match the product name passed to tauri-bundler.
    name: "Your Awesome App".into(),
    version: env!("CARGO_PKG_VERSION").parse().unwrap(),
    authors: env!("CARGO_PKG_AUTHORS"),
    description: env!("CARGO_PKG_DESCRIPTION"),
    crate_name: env!("CARGO_PKG_NAME"),
  };

  resource_dir(&package_info, &Env::default()).map(|dir| dir.join(path))
}
```

The helper currently resolves the desktop bundle layouts as follows:

| Platform               | Resource root                                     |
| ---------------------- | ------------------------------------------------- |
| Windows                | The directory containing the installed executable |
| Linux (`deb` or `rpm`) | `/usr/lib/<product-name>`                         |
| Linux (AppImage)       | `${APPDIR}/usr/lib/<product-name>`                |
| macOS                  | `<app>.app/Contents/Resources`                    |

These locations describe the current package layout, not a path contract. Use the helper instead of
hardcoding them.

### Other base directories

The bundle identifier does not determine the resource root. `tauri-bundler` uses it for
platform-specific package metadata where applicable, but the application is responsible for
choosing and creating its runtime data directories.

Tauri's generic desktop base directories are provided by the
[`dirs`](https://docs.rs/dirs/latest/dirs/) crate. Its `config_dir`, `data_dir`,
`data_local_dir`, and `cache_dir` functions return OS-level user directories and do not append an
application name or identifier.

Tauri's app-specific desktop directories append the bundle identifier to those generic bases:

| Purpose                      | Equivalent path without the Tauri runtime                 |
| ---------------------------- | --------------------------------------------------------- |
| App config                   | `dirs::config_dir()?.join(identifier)`                    |
| App data                     | `dirs::data_dir()?.join(identifier)`                      |
| App local data               | `dirs::data_local_dir()?.join(identifier)`                |
| App cache                    | `dirs::cache_dir()?.join(identifier)`                     |
| App logs (Linux and Windows) | `dirs::data_local_dir()?.join(identifier).join("logs")`   |
| App logs (macOS)             | `dirs::home_dir()?.join("Library/Logs").join(identifier)` |

## License

(c) 2017 - present, George Burton, Tauri-Apps Organization

This program is licensed either under the terms of the
[Apache Software License](http://www.apache.org/licenses/LICENSE-2.0), or the
[MIT License](https://opensource.org/licenses/MIT).

-> note, for bundle_dmg we have included a BSD 3 licensed binary `seticon`.
https://github.com/sveinbjornt/osxiconutils/blob/master/seticon.m
`tools/rust/cargo-tauri-bundle/src/bundle/templates/seticon`
