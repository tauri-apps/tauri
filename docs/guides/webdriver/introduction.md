---
title: Introduction
---
import Alert from '@theme/Alert'

<Alert title="Currently in pre-alpha" type="info" icon="info-alt">

Webdriver support for Tauri is still in pre-alpha. Tooling that is dedicated to it such as [tauri-driver] is still in
active development and may change as necessary over time. Additionally, only Windows and Linux are currently supported.
</Alert>

[WebDriver] is a standardized interface to interact with web documents that is primarily intended for automated testing.
Tauri supports the [WebDriver] interface by leveraging the native platform's [WebDriver] server underneath a
cross-platform wrapper [`tauri-driver`].

## System Dependencies

Install the latest [`tauri-driver`] or update an existing installation by running:

```sh
cargo install tauri-driver
```

Because we currently utilize the platform's native [WebDriver] server, there are some requirements for running
[`tauri-driver`] on supported platforms. Platform support is currently limited to Linux and Windows.

### Linux

We use `WebKitWebDriver` on linux platforms. Check if this binary exists already (command `which WebKitWebDriver`) as
some distributions bundle it with the regular webkit package. Other platforms may have a separate package for them such
as `webkit2gtk-driver` on Debian based distributions.

### Windows

Make sure to grab the version of [Microsoft Edge Driver] that matches your Windows' Edge version that the application is
being built and tested on. On up-to-date Window installs, this should almost always be the latest stable version. If the
two versions do not match, you may experience your WebDriver testing suite hanging while trying to connect.

The download contains a binary called `msedgedriver.exe`. [`tauri-driver`] looks for that binary in the `$PATH` so make
sure it's either available on the path or use the `--native-driver` option on [`tauri-driver`]. On Windows CI machines,
you may want to download this automatically as part of the CI setup process to ensure the Edge and Edge Driver versions
stay in sync. A guide on how to do this may be added at a later date.

## Example Application

The [next section](example/setup) of the guide will show step-by-step how to create a minimal example application that
is tested with WebDriver.

If you prefer to just see the result of the guide and look over a finished minimal codebase that utilizes it then you
can look at https://github.com/chippers/hello_tauri. That example also comes with a CI script to test with GitHub
actions, but you may still be interested in the [WebDriver CI](ci) guide as it explains the concept a bit more.

[WebDriver]: https://www.w3.org/TR/webdriver/
[`tauri-driver`]: https://crates.io/crates/tauri-driver
[tauri-driver]: https://crates.io/crates/tauri-driver
[Microsoft Edge Driver]: https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
