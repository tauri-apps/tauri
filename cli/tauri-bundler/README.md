# Tauri Bundler

Wrap Rust executables in OS-specific app bundles.

## About

This is a fork of the awesome [cargo-bundle](https://github.com/burtonageo/cargo-bundle), turned into a library used by the [Tauri CLI](../core).

## Configuration

Tauri automatically loads configurations from the `tauri.conf.json > tauri > bundle` object, but this library doesn't rely on it and can be used by non-Tauri apps.

### General settings

These settings apply to bundles for all (or most) OSes.

 * `name`: The name of the built application. If this is not present, then it will use the `name` value from
           your `Cargo.toml` file.
 * `identifier`: [REQUIRED] A string that uniquely identifies your application,
   in reverse-DNS form (for example, `"com.example.appname"` or
   `"io.github.username.project"`).  For OS X and iOS, this is used as the
   bundle's `CFBundleIdentifier` value; for Windows, this is hashed to create
   an application GUID.
 * `icon`: [OPTIONAL] The icons used for your application.  This should be an array of file paths or globs (with images
           in various sizes/formats); `tauri-bundler` will automatically convert between image formats as necessary for
           different platforms.  Supported formats include ICNS, ICO, PNG, and anything else that can be decoded by the
           [`image`](https://crates.io/crates/image) crate.  Icons intended for high-resolution (e.g. Retina) displays
           should have a filename with `@2x` just before the extension (see example below).
 * `version`: [OPTIONAL] The version of the application. If this is not present, then it will use the `version`
              value from your `Cargo.toml` file.
 * `resources`: [OPTIONAL] List of files or directories which will be copied to the resources section of the
                bundle. Globs are supported.
 * `copyright`: [OPTIONAL] This contains a copyright string associated with your application.
 * `category`: [OPTIONAL] What kind of application this is.  This can
   be a human-readable string (e.g. `"Puzzle game"`), or a Mac OS X
   LSApplicationCategoryType value
   (e.g. `"public.app-category.puzzle-games"`), or a GNOME desktop
   file category name (e.g. `"LogicGame"`), and `tauri-bundler` will
   automatically convert as needed for different platforms.
 * `short_description`: [OPTIONAL] A short, one-line description of the application. If this is not present, then it
                        will use the `description` value from your `Cargo.toml` file.
 * `long_description`: [OPTIONAL] A longer, multi-line description of the application.

### Debian-specific settings

These settings are used only when bundling `deb` packages.

* `depends`: A list of strings indicating other packages (e.g. shared
  libraries) that this package depends on to be installed.  If present, this
  forms the `Depends:` field of the `deb` package control file.
* `use_bootstrapper`: Enables the bootstrapper script, which allows access to the environment variables.

### Mac OS X-specific settings

These settings are used only when bundling `app` and `dmg` packages.

* `frameworks`: A list of strings indicating any Mac OS X frameworks that
  need to be bundled with the app.  Each string can either be the name of a
  framework (without the `.framework` extension, e.g. `"SDL2"`), in which case
  `tauri-bundler` will search for that framework in the standard install
  locations (`~/Library/Frameworks/`, `/Library/Frameworks/`, and
  `/Network/Library/Frameworks/`), or a path to a specific framework bundle
  (e.g. `./data/frameworks/SDL2.framework`).  Note that this setting just makes
  `tauri-bundler` copy the specified frameworks into the OS X app bundle (under
  `Foobar.app/Contents/Frameworks/`); you are still responsible for (1)
  arranging for the compiled binary to link against those frameworks (e.g. by
  emitting lines like `cargo:rustc-link-lib=framework=SDL2` from your
  `build.rs` script), and (2) embedding the correct rpath in your binary
  (e.g. by running `install_name_tool -add_rpath
  "@executable_path/../Frameworks" path/to/binary` after compiling).
* `minimum_system_version`: A version string indicating the minimum Mac OS
  X version that the bundled app supports (e.g. `"10.11"`).  If you are using
  this config field, you may also want have your `build.rs` script emit
  `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11` (or whatever version number
  you want) to ensure that the compiled binary has the same minimum version.
* `license`: Path to the license file for the DMG bundle.
* `exception_domain`: The exception domain to use on the macOS .app bundle. Allows communication to the outside world e.g. a web server you're shipping.
* `use_bootstrapper`: Enables the bootstrapper script, which allows access to the environment variables.

### Example `tauri.conf.json`:

```json
{
  "package": {
    "productName": "Your Awesome App",
    "version": "0.1.0"
  },
  "tauri": {
    "bundle": {
      "active": true,
      "identifier": "com.my.app",
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
        "depends": ["debian-dependency1", "debian-dependency2"],
        "useBootstrapper": true
      },
      "macOS": {
        "frameworks": [],
        "minimumSystemVersion": "10.11",
        "license": "./LICENSE",
        "useBootstrapper": true
      },
      "externalBin": ["./sidecar-app"]
    }
  }
}
```

## License
(c) 2017 - present, George Burton, Tauri-Apps Organization

This program is licensed either under the terms of the
[Apache Software License](http://www.apache.org/licenses/LICENSE-2.0), or the
[MIT License](https://opensource.org/licenses/MIT).

-> note, for bundle_dmg we have included a BSD 3 licenced binary `seticon`.
https://github.com/sveinbjornt/osxiconutils/blob/master/seticon.m
`tools/rust/cargo-tauri-bundle/src/bundle/templates/seticon`
