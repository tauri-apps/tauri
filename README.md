# tauri
 <img align="right" src="app-icon.png" height="128" width="128">

## A fresh take on creating cross-platform apps.
[![status](https://img.shields.io/badge/Status-Alpha-yellow.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![Chat Server](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![devto](https://img.shields.io/badge/blog-dev.to-black.svg)](https://dev.to/tauri)

![](https://img.shields.io/github/workflow/status/tauri-apps/tauri/test%20library?label=test%20library
)
[![devto](https://img.shields.io/badge/documentation-wiki-purple.svg)](https://github.com/tauri-apps/tauri/wiki)

[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Opencollective-blue.svg)](https://opencollective.com/tauri)


Tauri is a tool for building tiny, blazing fast binaries for all major desktop platforms. You can use any front-end framework that compiles to HTML,JS and CSS for building your interface.

| Component | Version | Lin | Win | Mac |
|-----------|---------|-----|-----|-----|
| tauri.js CLI | ![](https://img.shields.io/npm/v/tauri.svg)         |✅|✅|✅|
| tauri core    | ![](https://img.shields.io/crates/v/tauri.svg)      |✅|✅|✅|
| tauri bundler | ![](https://img.shields.io/crates/v/tauri-bundler.svg)  |✅|✅|✅ |

## Who Tauri is For
Because of the way Tauri has been built and can be extended, developers
are able to interface not only with the entire Rust ecosystem, but also
with many other programming languages. Being freed of the heaviest thing
in the universe and the many shortcomings of server-side Javascript
suddenly opens up whole new avenues for high-performance, security-focused
applications that need the purebred power, agility and community
acceptance of a low-level language.

We expect to witness an entire new class of applications being built with
Tauri. From a simple calender to locally crunching massive realtime
feeds at particle colliders or even mesh-network based distributed message-
passing ecosystems - the bar has been raised and gauntlet thrown.

What will you make?

## 4 Reasons to consider Tauri
- **BUNDLE SIZE** of a vanilla Tauri app is less than 3 MB - about 140 MB smaller than what you get with Electron.
- **MEMORY FOOTPRINT** is less than half of the size of an Electron app built from the same codebase.
- **SECURITY** is Tauri's biggest priority and we are constantly innovating.
- **FLOSS** licensing is regretfully impossible with downstream Chromium consumers, like Electron. Sources: [0](https://lists.gnu.org/archive/html/libreplanet-discuss/2017-01/msg00056.html) [1](https://lists.gnu.org/archive/html/directory-discuss/2017-12/msg00008.html) [2](https://lists.gnu.org/archive/html/libreplanet-discuss/2019-02/msg00001.html)

## Technical Details
Tauri has five major components:
- [Node.js CLI](https://github.com/tauri-apps/tauri/tree/dev/cli/tauri.js) for creating, developing and building apps
- [Rust Core](https://github.com/tauri-apps/tauri/tree/dev/tauri) for binding to the low level WEBVIEW and providing a tree-shakeable API
- [Rust Bundler](https://github.com/tauri-apps/tauri/tree/dev/cli/tauri-bundler) for manufacturing the final binaries
- [Rust Bindings](https://github.com/Boscop/web-view) for Webviews
- [Webview](https://github.com/Boscop/web-view/tree/master/webview-sys)
Low level library for creating and interfacing with OS "native" webviews

The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS,
gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows.
**Tauri** is based on the MIT licensed prior work known as
[webview](https://github.com/zserge/webview).

The default binding to the underlying webview library currently uses Rust,
but other languages like Golang or Python (and many others) are possible
(and only a PR away).

The combination of power, safety and usability are why we chose Rust to be
the default binding for Tauri. It is our intention to provide the most safe
and performant native app experience (for devs and app consumers), out of
the box.

#### App Bundles
- [x] App Icons
- [x] Build on MacOS (.app, .dmg coming soon)
- [x] Build on Linux (.deb, AppImage coming soon)
- [x] Build on Windows (.exe, .msi coming soon)
- [ ] App Signing
- [ ] Self Updater (WIP)
- [ ] Frameless Mode
- [ ] Transparent Mode
- [ ] Multiwindow Mode
- [ ] Tray (coming soon)
- [x] Copy Buffer

#### API
- [x] bridge - enable fast bridge
- [x] event - enable binding to message
- [x] execute - STDOUT Passthrough with Command Invocation
- [x] listFiles - list files in a directory
- [x] open - open link in a browser
- [x] readBinaryFile - read binary file from local filesystem
- [x] readTextFile - read text file from local filesystem
- [x] setTitle - set the window title
- [x] writeFile - write file to local filesystem
- [x] API Spec
- [x] Documentation (WIP)

### Security Features
- [x] localhost-free mode (:fire:)
- [x] Dynamic ahead of Time Compilation (dAoT) with functional tree-shaking
- [x] functional Address Space Layout Randomization
- [x] OTP salting of function names and messages at runtime
- [x] CSP Injection
- [ ] Frida-based harness for Post-Binary Analysis

### Comparison between Tauri and Electron

|  | Tauri | Electron |
|--|--------|----------|
| Binary Size MacOS | 0.6 MB | 47.7 MB |
| Memory Consumption MacOS | 13 MB | 34.1 MB |
| Interface Service Provider | Varies | Chromium |
| Backend Binding | Rust | Node.js (ECMAScript) |
| Underlying Engine | C/C++ | V8 (C/C++) |
| FLOSS | Yes | No |
| Multithreading | Yes | No |
| Bytecode Delivery | Yes | No |
| Can Render PDF | Yes | No |
| Multiple Windows | Soon | Yes |
| GPU Access | Yes | Yes |
| Auto Updater | Soon | Yes (1) |
| Cross Platform | Yes | Yes |
| Custom App Icon | Yes | Yes |
| Windows Binary | Yes | Yes |
| MacOS Binary | Yes | Yes |
| Linux Binary | Yes | Yes |
| iOS Binary | Soon | No |
| Android Binary | Soon | No |
| Localhost Server | Yes | Yes |
| No localhost option | Yes | No |
| Desktop Tray | Soon | No |

#### Notes
1) Electron has no native auto updater on Linux, but is offered by electron-packager

## Organization
Tauri aims to be a sustainable collective based on principles that guide [sustainable
free and open software communities](https://sfosc.org). You can get involved in many ways.

This has been done with our best attempt at due diligence and in
respect of the original authors. Thankyou - this project would never have
been possible without your amazing contribution to open-source and we are
honoured to carry the torch further. Of special note:
- [zserge](https://github.com/zserge) for the original webview approach and
go bindings
- [Boscop](https://github.com/Boscop) for the Rust Bindings
- [Burtonago](https://github.com/burtonageo) for the Cargo Bundle prototype

## Contributing
Please make sure to read the [Contributing Guide](./.github/CONTRIBUTING.md)
before making a pull request.

Thank you to all the people who already contributed to Tauri!

Special thanks to the development team at Volentix Labs for the encouragement and support in the early phases of Tauri, notably Rhys Parry and Gregory Luneau. Also a warm thanks to the incubation period at the Quasar Framework and specifically Razvan Stoenescu for believing in Tauri from the beginning.

## Semver
**tauri** is following [Semantic Versioning 2.0](https://semver.org/).

## Licenses
Code: (c) 2015 - present - Daniel Thompson-Yvetot, Lucas Nogueira, Tensor, Boscop, Serge Zaitsev, George Burton and all the other amazing contributors.

MIT or MIT/Apache where applicable.

Logo: CC-BY-NC-ND
- Original Tauri Logo Design by [Daniel Thompson-Yvetot](https://github.com/nothingismagick)
