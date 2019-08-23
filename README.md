# quasar-tauri [WIP]
## A fresh take on creating cross-platform apps.
[![official icon](https://img.shields.io/badge/Quasar%201.0-Official-blue.svg)](https://quasar.dev)
[![status](https://img.shields.io/badge/Status-Internal%20Review-yellow.svg)](https://github.com/quasarframework/quasar/tree/tauri)
[![version](https://img.shields.io/badge/Version-unreleased-yellow.svg)](https://github.com/quasarframework/quasar/tree/tauri) <img align="right" src="https://cdn.quasar.dev/logo/tauri/tauri-logo-240x240.png">

[![Join the chat at https://chat.quasar.dev](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://chat.quasar.dev)
<a href="https://forum.quasar.dev" target="_blank"><img src="https://img.shields.io/badge/community-forum-brightgreen.svg"></a>
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)

**Tauri** brings a mode to build Quasar Apps that creates tiny, blazing 
fast binaries for all major desktop platforms. In Quasar's 
[neverending quest](https://quasar.dev/introduction-to-quasar#Why-Quasar%3F) 
for performance and security, the core team is proud to offer an
alternative to Electron.

Whether you are just starting out making apps for your meetup or 
regularly crunch terabyte datasets, we are absolutely confident that 
you will love using Tauri as much as we love making and maintaining it.

## Who Tauri is For
Anyone who can create a Quasar app can use Tauri, as it is *merely* a new 
build target. All components and plugins (suitable for Native Desktop) can
be used. For the User Interface, nothing has changed, except you will 
probably notice that everything seems much faster.

Because of the way Tauri has been built and can be extended, developers 
are able to interface not only with the entire Rust ecosystem, but also 
with many other programming languages. Being freed of the heaviest thing 
in the universe and the many shortcomings of server-side Javascript 
suddenly opens up whole new avenues for high-performance, security-focused
applications that need the purebred power, agility and community 
acceptance of a low-level language.

We expect to witness an entire new class of applications being built with 
Quasar Tauri. From a simple calender to locally crunching massive realtime 
feeds at particle colliders or even mesh-network based distributed message-
passing ecosystems - the bar has been raised and gauntlet thrown. 

What will you make?

## 5 Reasons to consider Tauri
- **BUNDLE SIZE** of a vanilla Tauri app is less than 3 MB - about 140 MB smaller than what you get with Electron.
- **MEMORY FOOTPRINT** is less than half of the size of an Electron app built from the same codebase. 
- **SECURITY** is Tauri's biggest priority and we take it so seriously that we innovate to keep hackers out of your apps. 
- **RELIABILITY** of the underlying code base is why critical libraries have been forked and will be perpetually maintained.
- **FLOSS** licensing is regretfully impossible with downstream Chromium consumers, like Electron.

## Technical Details
The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS, 
gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows. 
**Tauri** is based on the MIT licensed prior work known as 
[webview](https://github.com/zserge/webview).

The default binding to the underlying webview library currently uses Rust,
but other languages like Golang or Python (and many others) are possible 
(and only a PR away).

> Rust is blazingly fast and memory-efficient: with no runtime or garbage 
collector, it can power performance-critical services, run on embedded 
devices, and easily integrate with other languages. Rust’s rich type system
and ownership model guarantee memory-safety and thread-safety — and enable
you to eliminate many classes of bugs at compile-time. Rust has great 
documentation, a friendly compiler with useful error messages, and top-notch
tooling — an integrated package manager and build tool, smart multi-editor
support with auto-completion and type inspections, an auto-formatter, and 
more. - [https://www.rust-lang.org/](https://www.rust-lang.org/)

This combination of power, safety and usability are why we chose Rust to be
the default binding for Tauri. It is our intention to provide the most safe
and performant native app experience (for devs and app consumers), out of 
the box. 

To this end, we have spent a great deal of time creating an especially secure 
localhost-free backend for the security conscious application-artisans. This 
means that your app does not use a localhost server, as is generally the case with 
cordova apps. This also has the positive side effect, that less code is needed
and the final binaries are smaller.

> Less code doesn't always mean something is safer, but it does mean that
> there is less surface area for attackers to barnacle themselves.  - Denjell

### Current Status
We are in the process of vetting this new mode. It is not yet available to
use without jumping through some development hurdles. If you don't care,
please reach out to the team at https://chat.quasar.dev and we'll guide
you through the process. Here is a bit of a status report.

#### App Bundles
- [x] App Icons and integration with Icon-Genie
- [x] Build on MacOS (.app, .dmg coming soon)
- [x] Build on Linux (.deb, AppImage coming soon)
- [x] Build on Windows (.exe, .msi coming soon)
- [ ] App Signing
- [x] Self Updater (WIP)
- [ ] Frameless Mode
- [ ] Transparent Mode
- [ ] Multiwindow Mode
- [ ] Tray (coming soon)
- [x] Copy Buffer

#### API 
- [ ] answer - enable rust to direct the UI
- [ ] bridge - enable Quasar Bridge
- [x] event - enable binding to message
- [x] execute - STDOUT Passthrough with Command Invocation
- [x] listFiles - list files in a directory 
- [x] open - open link in a browser
- [x] readBinaryFile - read binary file from local filesystem
- [x] readTextFile - read text file from local filesystem
- [x] setTitle - set the window title
- [x] writeFile - write file to local filesystem
- [x] API Spec
- [x] Inter Process Communication (IPC)
- [x] Documentation (WIP)
- [x] Message Bus

### Security Features
- [x] localhost-free mode (:fire:)
- [x] Secure Cryptographic Enclave (devland implementation)
- [x] Dynamic ahead of Time Compilation (dAoT) with functional tree-shaking
- [x] functional Address Space Layout Randomization
- [x] OTP salting of function names and messages
- [x] CSP Injection
- [ ] Frida-based harness for Post-Binary Analysis

### Comparison between Tauri 1 and Electron 5

|  | Tauri | Electron |
|--|--------|----------|
| Binary Size MacOS | 2.6 MB | 147.7 MB |
| Memory Consumption MacOS | 13 MB | 34.1 MB |
| Benchmark FPS | TODO | TODO |
| Interface Service Provider | Varies | Chromium |
| Quasar UI | VueJS | VueJS |
| Backend Binding | Rust | Node.js (ECMAScript) |
| Underlying Engine | C/C++ | V8 (C/C++) |
| FLOSS | Yes | No |
| Multithreading | Yes | No |
| Bytecode Delivery | Yes | No |
| Can Render PDF | Yes | No |
| Multiple Windows | Soon | Yes |
| GPU Access | Yes | Yes |
| Auto Updater | Yes | Yes (1) |
| Inter Process Communication (IPC) | Yes | Yes |
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

## Relation to Upstream Origins
We have made the decision to fork, enhance and maintain several upstream
projects here in this repository, in order to guarantee the security of the
code and our ability to enhance it with features that may not be needed for
other consumers.

We hope that this code is useful, but make no claims to suitability or 
guarantees that it will work outside of the Quasar ecosystem.

This has been done with our best attempt at due diligence and in
respect of the original authors. Thankyou - this project would never have
been possible without your amazing contribution to open-source and we are
honoured to carry the torch further. Of special note:
- [zserge](https://github.com/zserge) for the original webview approach and 
go bindings
- [Boscop](https://github.com/Boscop) for the Rust Bindings
- [Burtonago](https://github.com/burtonageo) for the Cargo Bundle prototype

## Documentation
Head over to the Quasar Framework official website: 
[https://quasar.dev](https://quasar.dev)

## Stay in Touch
For latest releases and announcements, follow on Twitter: 
[@quasarframework](https://twitter.com/quasarframework)

## Chat Support
Get realtime help at the official community Discord server: 
[https://chat.quasar.dev](https://chat.quasar.dev)

## Community Forum
Ask complicated questions at the official community forum: 
[https://forum.quasar.dev](https://forum.quasar.dev)

## Contributing
Please make sure to read the [Contributing Guide](./.github/CONTRIBUTING.md) 
before making a pull request. If you have a Quasar-related 
project/component/tool, add it with a pull request to 
[this curated list](https://github.com/quasarframework/quasar-awesome)!

Thank you to all the people who already <a href="https://github.com/quasarframework/tauri/graphs/contributors">contributed to Tauri</a>!

Special thanks to the development team at Volentix Labs for the encouragement and support in the early phases of Tauri, notably Rhys Parry and Gregory Luneau.

## Semver
quasarframework/tauri is following [Semantic Versioning 2.0](https://semver.org/).

## Licenses
Code: (c) 2015 - 2019 - Daniel Thompson-Yvetot, Razvan Stoenescu, Lucas Nogueira, Tensor, Boscop, Serge Zaitsev, George Burton and all the other amazing contributors.

MIT or MIT/Apache where applicable.

Logo: CC-BY-NC-ND
- Original Tauri Logo Design by [Daniel Thompson-Yvetot](https://github.com/nothingismagick)
- Based on the prior work by [Emanuele Bertoldi](https://github.com/zuck)

Name: The proper name of this project is "Quasar Tauri", and is to be used in all citations.
