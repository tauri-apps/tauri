<p align="center"><img height="100" src="https://raw.githubusercontent.com/tauri-apps/wry/refs/heads/dev/.github/splash.png" alt="WRY Webview Rendering library" /></p>

[![](https://img.shields.io/crates/v/wry?style=flat-square)](https://crates.io/crates/wry) [![](https://img.shields.io/docsrs/wry?style=flat-square)](https://docs.rs/wry/)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

Wry is a cross-platform WebView rendering library.

The webview requires a running event loop and a window type that implements [`HasWindowHandle`],
or a gtk container widget if you need to support X11 and Wayland.
You can use a windowing library like [`tao`] or [`winit`].

### Examples

This example leverages the [`HasWindowHandle`] and supports Windows, macOS, iOS, Android and Linux (X11 Only).
See the following example using [`winit`]:

```rust
#[derive(Default)]
struct App {
  window: Option<Window>,
  webview: Option<wry::WebView>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window = event_loop.create_window(Window::default_attributes()).unwrap();
    let webview = WebViewBuilder::new()
      .with_url("https://tauri.app")
      .build(&window)
      .unwrap();

    self.window = Some(window);
    self.webview = Some(webview);
  }

  fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {}
}

let event_loop = EventLoop::new().unwrap();
let mut app = App::default();
event_loop.run_app(&mut app).unwrap();
```

If you also want to support Wayland too, then we recommend you use [`WebViewBuilderExtUnix::new_gtk`] on Linux.
See the following example using [`tao`]:

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new().build(&event_loop).unwrap();

let builder = WebViewBuilder::new().with_url("https://tauri.app");

#[cfg(not(target_os = "linux"))]
let webview = builder.build(&window).unwrap();
#[cfg(target_os = "linux")]
let webview = builder.build_gtk(window.gtk_window()).unwrap();
```

### Child webviews

You can use [`WebViewBuilder::build_as_child`] to create the webview as a child inside another window. This is supported on
macOS, Windows and Linux (X11 Only).

```rust
#[derive(Default)]
struct App {
  window: Option<Window>,
  webview: Option<wry::WebView>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window = event_loop.create_window(Window::default_attributes()).unwrap();
    let webview = WebViewBuilder::new()
      .with_url("https://tauri.app")
      .with_bounds(Rect {
        position: LogicalPosition::new(100, 100).into(),
        size: LogicalSize::new(200, 200).into(),
      })
      .build_as_child(&window)
      .unwrap();

    self.window = Some(window);
    self.webview = Some(webview);
  }

  fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {}
}

let event_loop = EventLoop::new().unwrap();
let mut app = App::default();
event_loop.run_app(&mut app).unwrap();
```

If you want to support X11 and Wayland at the same time, we recommend using
[`WebViewExtUnix::new_gtk`] or [`WebViewBuilderExtUnix::new_gtk`] with [`gtk::Fixed`].

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new().build(&event_loop).unwrap();

let builder = WebViewBuilder::new()
  .with_url("https://tauri.app")
  .with_bounds(Rect {
    position: LogicalPosition::new(100, 100).into(),
    size: LogicalSize::new(200, 200).into(),
  });

#[cfg(not(target_os = "linux"))]
let webview = builder.build_as_child(&window).unwrap();
#[cfg(target_os = "linux")]
let webview = {
  # use gtk::prelude::*;
  let vbox = window.default_vbox().unwrap(); // tao adds a gtk::Box by default
  let fixed = gtk::Fixed::new();
  vbox.append(&fixed);
  builder.build_gtk(&fixed).unwrap()
};
```

### Platform Considerations

Here is the underlying web engine each platform uses, and some dependencies you might need to install.

#### Linux

[WebKitGTK](https://webkitgtk.org/) is used to provide webviews on Linux which requires GTK,
so if the windowing library doesn't support GTK (as in [`winit`])
you'll need to call [`gtk::init`] before creating the webview and then iterate the default GLib main context alongside
your windowing library event loop.

```rust
#[derive(Default)]
struct App {
  webview_window: Option<(Window, WebView)>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window = event_loop.create_window(Window::default_attributes()).unwrap();
    let webview = WebViewBuilder::new()
      .with_url("https://tauri.app")
      .build(&window)
      .unwrap();

    self.webview_window = Some((window, webview));
  }

  fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {}

  // Advance GTK event loop <!----- IMPORTANT
  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    #[cfg(target_os = "linux")]
    while gtk::glib::MainContext::default().pending() {
      gtk::glib::MainContext::default().iteration(false);
    }
  }
}

let event_loop = EventLoop::new().unwrap();
let mut app = App::default();
event_loop.run_app(&mut app).unwrap();
```

##### Linux Dependencies

###### Arch Linux / Manjaro:

```bash
sudo pacman -S webkitgtk-6.0
```

###### Debian / Ubuntu:

```bash
sudo apt install libwebkitgtk-6.0-dev
```

###### Fedora

```bash
sudo dnf install gtk4-devel webkitgtk6.0-devel
```

###### Nix & NixOS

```nix
# shell.nix

let
   # Unstable Channel | Rolling Release
   pkgs = import (fetchTarball("channel:nixpkgs-unstable")) { };
   packages = with pkgs; [
     pkg-config
     webkitgtk_6_0
   ];
 in
 pkgs.mkShell {
   buildInputs = packages;
 }
```

```sh
nix-shell shell.nix
```

###### GUIX

```scheme
;; manifest.scm

(specifications->manifest
  '("pkg-config"                ; Helper tool used when compiling
    "webkitgtk"                 ; Web content engine fot GTK+
 ))
```

```bash
guix shell -m manifest.scm
```

#### macOS

WebKit is native on macOS so everything should be fine.

If you are cross-compiling for macOS using [osxcross](https://github.com/tpoechtrager/osxcross) and encounter a runtime panic like `Class with name WKWebViewConfiguration could not be found` it's possible that `WebKit.framework` has not been linked correctly, to fix this set the `RUSTFLAGS` environment variable:

```bash
RUSTFLAGS="-l framework=WebKit" cargo build --target=x86_64-apple-darwin --release
```

#### Windows

WebView2 provided by Microsoft Edge Chromium is used. So wry supports Windows 7, 8, 10 and 11.

#### Android

In order for `wry` to be able to create webviews on Android, there are a few requirements that your application needs to uphold:

1. You need to set a few environment variables that will be used to generate the necessary kotlin
   files that you need to include in your Android application for wry to function properly.
   - `WRY_ANDROID_PACKAGE`: which is the reversed domain name of your android project and the app name in snake_case, for example, `com.wry.example.wry_app`
   - `WRY_ANDROID_LIBRARY`: for example, if your cargo project has a lib name `wry_app`, it will generate `libwry_app.so` so you set this env var to `wry_app`
   - `WRY_ANDROID_KOTLIN_FILES_OUT_DIR`: for example, `path/to/app/src/main/kotlin/com/wry/example`
2. Your main Android Activity needs to inherit `AppCompatActivity`, preferably it should use the generated `WryActivity` or inherit it.
3. Your Rust app needs to call `wry::android_setup` function to setup the necessary logic to be able to create webviews later on.
4. Your Rust app needs to call `wry::android_binding!` macro to setup the JNI functions that will be called by `WryActivity` and various other places.

It is recommended to use the [`tao`](https://docs.rs/tao/latest/tao/) crate as it provides maximum compatibility with `wry`.

```rust
#[cfg(target_os = "android")]
{
  tao::android_binding!(
      com_example,
      wry_app,
      WryActivity,
      wry::android_setup, // pass the wry::android_setup function to tao which will be invoked when the event loop is created
      _start_app
  );
  wry::android_binding!(com_example, ttt);
}
```

If this feels overwhelming, you can just use the preconfigured template from [`cargo-mobile2`](https://github.com/tauri-apps/cargo-mobile2).

For more information, check out [MOBILE.md](https://github.com/tauri-apps/wry/blob/dev/MOBILE.md).

### Feature flags

Wry uses a set of feature flags to toggle several advanced features.

- `os-webview` (default): Enables the default WebView framework on the platform. This must be enabled
  for the crate to work. This feature was added in preparation of other ports like cef and servo.
- `protocol` (default): Enables [`WebViewBuilder::with_custom_protocol`] to define custom URL scheme for handling tasks like
  loading assets.
- `drag-drop` (default): Enables [`WebViewBuilder::with_drag_drop_handler`] to control the behavior when there are files
  interacting with the window.
- `devtools`: Enables devtools on release builds. Devtools are always enabled in debug builds.
  On **macOS**, enabling devtools, requires calling private APIs so you should not enable this flag in release
  build if your app needs to publish to App Store.
- `transparent`: Transparent background on **macOS** requires calling private functions.
  Avoid this in release build if your app needs to publish to App Store.
- `fullscreen`: Fullscreen video and other media on **macOS** requires calling private functions.
  Avoid this in release build if your app needs to publish to App Store.
- `linux-body`: Enables body support of custom protocol request on Linux. Requires
  WebKit2GTK v2.40 or above.
- `tracing`: enables [`tracing`] for `evaluate_script`, `ipc_handler`, and `custom_protocols`.

### Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src=".github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

### License

Apache-2.0/MIT

[`tao`]: https://docs.rs/tao
[`winit`]: https://docs.rs/winit
[`tracing`]: https://docs.rs/tracing
