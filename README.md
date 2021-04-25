<h1 style="font-family:monospace, courier; font-size:3em"><img align="left" src="app-icon.png" height="52" width="52">&nbsp;TAURI</h2>

[![status](https://img.shields.io/badge/Status-Beta-green.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test library](https://img.shields.io/github/workflow/status/tauri-apps/tauri/test%20library?label=test%20library)](https://github.com/tauri-apps/tauri/actions?query=workflow%3A%22test+library%22)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)

[![Chat Server](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![devto](https://img.shields.io/badge/blog-dev.to-black.svg)](https://dev.to/tauri)
[![devto](https://img.shields.io/badge/documentation-tauri.studio-purple.svg)](https://tauri.studio/docs/getting-started/intro)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-open%20collective-blue.svg)](https://opencollective.com/tauri)

```yml
Tauri Apps
  footprint:   minuscule
  performance: ludicrous
  flexibility: gymnastic
  security:    hardened
```

## Current Releases

| Component                                                                   | Description                    | Version                                                                                          | Lin | Win | Mac |
| --------------------------------------------------------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------ | --- | --- | --- |
| [**cli.rs**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli.rs)   | create, develop and build apps | [![](https://img.shields.io/crates/v/tauri-cli.svg)](https://crates.io/crates/tauri-cli)         | ✅   | ✅   | ✅   |
| [**cli.js**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli.js)   | Node.js CLI wrapper for cli.rs | [![](https://img.shields.io/npm/v/@tauri-apps/cli.svg)](https://www.npmjs.com/package/@tauri-apps/cli)               | ✅   | ✅   | ✅   |
| [**api.js**](https://github.com/tauri-apps/tauri/tree/dev/tooling/api)      | JS API for interaction with Rust backend | [![](https://img.shields.io/npm/v/@tauri-apps/api.svg)](https://www.npmjs.com/package/@tauri-apps/api)               | ✅   | ✅   | ✅   |
| [**core**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri)         | runtime core                   | [![](https://img.shields.io/crates/v/tauri.svg)](https://crates.io/crates/tauri)                 | ✅   | ✅   | ✅   |
| [**bundler**](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler) | manufacture the final binaries | [![](https://img.shields.io/crates/v/tauri-bundler.svg)](https://crates.io/crates/tauri-bundler) | ✅   | ✅   | ✅   |
| [**utils**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-utils)  | common tools for tauri         | [![](https://img.shields.io/crates/v/tauri-utils.svg)](https://crates.io/crates/tauri-utils)     | ✅   | ✅   | ✅   |


## Introduction
Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms. Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface. The backend of the application is a rust-sourced binary with an API that the front-end can interact with.

The user interface in Tauri apps currently leverages [`winit`](https://docs.rs/winit) as a WebView abstraction for all platforms via the **Tauri-team** incubated and maintained [wry](https://github.com/tauri-apps/wry), which creates a unified interface to these system webviews using gtk on linux and leveraging wkwebkit (macOS), webview2 (windows) webkitgtk (linux).


## Get Started
If you are interested in making a tauri-app, please visit the [documentation website](https://tauri.studio). This README is directed towards those who are interested in contributing to the core library. But if you just want a quick overview about where `tauri` is at in its development, here's a quick burndown:

#### App Bundles
- [x] App Icons
- [x] Build on MacOS (.app, .dmg)
- [x] Build on Linux (.deb, AppImage)
- [x] Build on Windows (.exe, .msi)
- [x] Copy Buffer
- [x] Device Notifications (toast)
- [x] Self Updater
- [x] App Signing (coming soon)
- [x] Frameless Mode (coming soon)
- [x] Transparent Mode (coming soon)
- [x] Multiwindow Mode (coming soon)
- [ ] deeplink RPC (in progress)
- [ ] One-Time commands (coming soon)
- [ ] Tray (coming soon)

#### API
- [x] setTitle - set the window title
- [x] command - make custom API interfaces
- [x] execute - STDOUT Passthrough with command invocation
- [x] open - open link in a browser
- [x] event - two part api consisting of `emit` and `listen`
- [x] httpRequest - command rust to make an http request
- [x] openDialog - native file chooser dialog
- [x] saveDialog - native file saver dialog
- [x] readDir - list files in a directory
- [x] createDir - create a directory
- [x] removeDir - remove a directory
- [x] removeFile - remove a file
- [x] renameFile - rename a file
- [x] copyFile - copy a file to a new destination
- [x] writeFile - write file to local filesystem
- [x] writeBinaryFile - write binary file to local filesystem
- [x] readBinaryFile - read binary file from local filesystem
- [x] readTextFile - read text file from local filesystem
- [ ] channel - stream constant data to the webview

### Security Features
- [x] localhost-free (:fire:)
- [x] custom protocol for secure mode
- [x] Dynamic ahead of Time Compilation (dAoT) with functional tree-shaking
- [x] functional Address Space Layout Randomization
- [x] OTP salting of function names and messages at runtime
- [x] CSP Injection

### Utilities
- [x] GH Action for creating binaries for all platforms
- [x] VS Code Extension
- [x] Tauri Core Plugins
- [x] Update core dependencies automatically from the command line
- [x] Rust-based CLI

### Comparison between Tauri and Electron

| Detail                     | Tauri  | Electron             |
| -------------------------- | ------ | -------------------- |
| Installer Size Linux       | 3.1 MB | 52.1 MB              |
| Memory Consumption Linux   | 180 MB | 462 MB               |
| Launch Time Linux          | 0.39s  | .80s                 |
| Interface Service Provider | Varies | Chromium             |
| Backend Binding            | Rust   | Node.js (ECMAScript) |
| Underlying Engine          | C/C++  | V8 (C/C++)           |
| FLOSS                      | Yes    | No                   |
| Multithreading             | Yes    | Yes                  |
| Bytecode Delivery          | Yes    | No                   |
| Multiple Windows           | Soon   | Yes                  |
| Auto Updater               | Soon   | Yes (1)              |
| Cross Platform             | Yes    | Yes                  |
| Custom App Icon            | Yes    | Yes                  |
| Windows Binary             | Yes    | Yes                  |
| MacOS Binary               | Yes    | Yes                  |
| Linux Binary               | Yes    | Yes                  |
| iOS Binary                 | Soon   | No                   |
| Android Binary             | Soon   | No                   |
| Desktop Tray               | Soon   | Yes                  |
| Sidecar Binaries           | Yes    | No                   |

#### Notes
1. Electron has no native auto updater on Linux, but is offered by electron-packager

## Development

Tauri is a system composed of a number of moving pieces:

### Infrastructure
- git for code management
- github for project management
- github actions for CI and CD
- discord for discussions
- netlify-hosted documentation website
- digital ocean meilisearch instance

### Major Runtimes
- node.js for running the CLI (deno and pure rust are on the roadmap)
- cargo for testing, running the dev service, building binaries and as the runtime harness for the webview

### Major Languages
- rust for the CLI
- ecmascript bindings to the RUST API, written in typescript
- rust for bindings, rust side of the API, harnesses
- rust plugins to tauri backend

### Operating systems
Tauri core can be developed on Mac, Linux and Windows, but you are encouraged to use the latest possible operating systems and build tools for your OS.

### Contribution Flow
Before you start working on something, it is best to check if there is an existing issue first. Also it is a good idea to stop by the Discord guild and confirm with the team if it makes sense or if someone is already working on it. If you want to read more about this, please see [this page](https://github.com/tauri-apps/tauri/blob/dev/.github/CONTRIBUTING.md).

### Documentation
Documentation in a polyglot system is a tricky proposition. To this end, we prefer to use inline documentation of Rust code and at JSDoc in typescript / javascript code. We autocollect these and publish them using Docusaurus v2 and netlify. Here is the hosting repository for the documentation site: https://github.com/tauri-apps/tauri-docs

### Testing & Linting
Test all the things! We have a number of test suites, but are always looking to improve our coverage:
- Rust (`cargo test`) => sourced via inline `#[cfg(test)]` declarations
- TS (`jest`) => via spec files
- Smoke Tests (run on merges to latest)
- eslint, clippy

### CI/CD
We recommend you read this article to understand better how we run our pipelines: https://www.jacobbolda.com/setting-up-ci-and-cd-for-tauri/

## Organization
Tauri aims to be a sustainable collective based on principles that guide [sustainable free and open software communities](https://sfosc.org). To this end it has become a Programme within the [Commons Conservancy](https://commonsconservancy.org/), and you can contribute financially via [Open Collective](https://opencollective.com/tauri).

## Contributing
Please make sure to read the [Contributing Guide](./.github/CONTRIBUTING.md)
before making a pull request.

Thank you to all the people who already contributed to Tauri!

## Semver
**tauri** is following [Semantic Versioning 2.0](https://semver.org/).

## Licenses
Code: (c) 2015 - 2021 - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.

Logo: CC-BY-NC-ND
- Original Tauri Logo Designs by [Daniel Thompson-Yvetot](https://github.com/nothingismagick) and [Guillaume Chau](https://github.com/akryum)


[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_large)
