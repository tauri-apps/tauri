<img src=".github/splash.png" alt="Tauri" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test core](https://img.shields.io/github/actions/workflow/status/tauri-apps/tauri/test-core.yml?label=test%20core&logo=github)](https://github.com/tauri-apps/tauri/actions/workflows/test-core.yml)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## Current Releases

### Core

| Component                                                                                    | Description                               | Version                                                                                                  | Lin | Win | Mac |
| -------------------------------------------------------------------------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**tauri**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri)                         | runtime core                              | [![](https://img.shields.io/crates/v/tauri.svg)](https://crates.io/crates/tauri)                         | ✅  | ✅  | ✅  |
| [**tauri-build**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-build)             | applies macros at build-time              | [![](https://img.shields.io/crates/v/tauri-build.svg)](https://crates.io/crates/tauri-build)             | ✅  | ✅  | ✅  |
| [**tauri-codegen**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-codegen)         | handles assets, parses tauri.conf.json    | [![](https://img.shields.io/crates/v/tauri-codegen.svg)](https://crates.io/crates/tauri-codegen)         | ✅  | ✅  | ✅  |
| [**tauri-macros**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-macros)           | creates macros using tauri-codegen        | [![](https://img.shields.io/crates/v/tauri-macros.svg)](https://crates.io/crates/tauri-macros)           | ✅  | ✅  | ✅  |
| [**tauri-runtime**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime)         | layer between Tauri and webview libraries | [![](https://img.shields.io/crates/v/tauri-runtime.svg)](https://crates.io/crates/tauri-runtime)         | ✅  | ✅  | ✅  |
| [**tauri-runtime-wry**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-runtime-wry) | enables system-level interaction via WRY  | [![](https://img.shields.io/crates/v/tauri-runtime-wry.svg)](https://crates.io/crates/tauri-runtime-wry) | ✅  | ✅  | ✅  |
| [**tauri-utils**](https://github.com/tauri-apps/tauri/tree/dev/core/tauri-utils)             | common code used across the tauri crates  | [![](https://img.shields.io/crates/v/tauri-utils.svg)](https://crates.io/crates/tauri-utils)             | ✅  | ✅  | ✅  |

### Tooling

| Component                                                                            | Description                              | Version                                                                                                | Lin | Win | Mac |
| ------------------------------------------------------------------------------------ | ---------------------------------------- | ------------------------------------------------------------------------------------------------------ | --- | --- | --- |
| [**bundler**](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler)          | manufacture the final binaries           | [![](https://img.shields.io/crates/v/tauri-bundler.svg)](https://crates.io/crates/tauri-bundler)       | ✅  | ✅  | ✅  |
| [**tauri-cli**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli)            | create, develop and build apps           | [![](https://img.shields.io/crates/v/tauri-cli.svg)](https://crates.io/crates/tauri-cli)               | ✅  | ✅  | ✅  |
| [**@tauri-apps/cli**](https://github.com/tauri-apps/tauri/tree/dev/tooling/cli/node) | Node.js CLI wrapper for `tauri-cli`      | [![](https://img.shields.io/npm/v/@tauri-apps/cli.svg)](https://www.npmjs.com/package/@tauri-apps/cli) | ✅  | ✅  | ✅  |
| [**@tauri-apps/api**](https://github.com/tauri-apps/tauri/tree/dev/tooling/api)      | JS API for interaction with Rust backend | [![](https://img.shields.io/npm/v/@tauri-apps/api.svg)](https://www.npmjs.com/package/@tauri-apps/api) | ✅  | ✅  | ✅  |

### Utilities and Plugins

| Component                                                                       | Description                           | Version                                                                                                          | Lin | Win | Mac |
| ------------------------------------------------------------------------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | --- | --- | --- |
| [**create-tauri-app**](https://github.com/tauri-apps/create-tauri-app)          | Get started with your first Tauri app | [![](https://img.shields.io/npm/v/create-tauri-app.svg)](https://www.npmjs.com/package/create-tauri-app)         | ✅  | ✅  | ✅  |
| [**vue-cli-plugin-tauri**](https://github.com/tauri-apps/vue-cli-plugin-tauri/) | Vue CLI plugin for Tauri              | [![](https://img.shields.io/npm/v/vue-cli-plugin-tauri.svg)](https://www.npmjs.com/package/vue-cli-plugin-tauri) | ✅  | ✅  | ✅  |

## Introduction

Tauri is a framework for building tiny, blazingly fast binaries for all major desktop platforms. Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface. The backend of the application is a rust-sourced binary with an API that the front-end can interact with.

The user interface in Tauri apps currently leverages [`tao`](https://docs.rs/tao) as a window handling library on macOS and Windows, and [`gtk`](https://gtk-rs.org/docs/gtk/) on Linux via the **Tauri-team** incubated and maintained [WRY](https://github.com/tauri-apps/wry), which creates a unified interface to the system webview (and other goodies like Menu and Taskbar), leveraging WebKit on macOS, WebView2 on Windows and WebKitGTK on Linux.

To learn more about the details of how all of these pieces fit together, please consult this [ARCHITECTURE.md](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md) document.

## Get Started

If you are interested in making a tauri app, please visit the [documentation website](https://tauri.app). This README is directed towards those who are interested in contributing to the core library. But if you just want a quick overview about where `tauri` is at in its development, here's a quick burndown:

### Platforms

Tauri currently supports development and distribution on the following platforms:

| Platform                 | Versions        |
| :----------------------- | :-------------- |
| Windows                  | 7 and above     |
| macOS                    | 10.15 and above |
| Linux                    | See below       |
| iOS/iPadOS (coming soon) |                 |
| Android (coming soon)    |                 |

**Linux Support**

For **developing** Tauri apps refer to the [Getting Started guide on tauri.app](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux).

For **running** Tauri apps we support the below configurations (these are automatically added as dependencies for .deb and are bundled for AppImage so that your users don't need to manually install them):

- Debian (Ubuntu 18.04 and above or equivalent) with the following packages installed:
  - `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`<sup>1</sup>
- Arch with the following packages installed:
  - `webkit2gtk`, `gtk3`, `libayatana-appindicator`<sup>1</sup>
- Fedora (latest 2 versions) with the following packages installed:
  - `webkit2gtk3`, `gtk3`, `libappindicator-gtk3`<sup>1</sup>
- Void with the following packages installed:
  - `webkit2gtk`, `gtk+3`, `libappindicator`<sup>1</sup>
- <sup>1</sup> `appindicator` is only required if system trays are used

### Features

- [x] Desktop Bundler (.app, .dmg, .deb, AppImage, .msi)
- [x] Self Updater
- [x] App Signing
- [x] Native Notifications (toast)
- [x] App Tray
- [x] Core Plugin System
- [x] Scoped Filesystem
- [x] Sidecar

### Security Features

- [x] localhost-free (:fire:)
- [x] custom protocol for secure mode
- [x] Dynamic ahead of Time Compilation (dAoT) with functional tree-shaking
- [x] functional Address Space Layout Randomization
- [x] OTP salting of function names and messages at runtime
- [x] CSP Injection

### Utilities

- [x] Rust-based CLI
- [x] GH Action for creating binaries for all platforms
- [x] VS Code Extension

## Development

Tauri is a system composed of a number of moving pieces:

### Infrastructure

- Git for code management
- GitHub for project management
- GitHub actions for CI and CD
- Discord for discussions
- Netlify-hosted documentation website
- DigitalOcean Meilisearch instance

### Operating systems

Tauri core can be developed on Mac, Linux and Windows, but you are encouraged to use the latest possible operating systems and build tools for your OS.

### Contributing

Before you start working on something, it's best to check if there is an existing issue first. It's also a good idea to stop by the Discord server and confirm with the team if it makes sense or if someone else is already working on it.

Please make sure to read the [Contributing Guide](./.github/CONTRIBUTING.md) before making a pull request.

Thank you to everyone contributing to Tauri!

### Documentation

Documentation in a polyglot system is a tricky proposition. To this end, we prefer to use inline documentation of Rust code and at JSDoc in typescript / javascript code. We autocollect these and publish them using Docusaurus v2 and netlify. Here is the hosting repository for the documentation site: https://github.com/tauri-apps/tauri-docs

### Testing & Linting

Test all the things! We have a number of test suites, but are always looking to improve our coverage:

- Rust (`cargo test`) => sourced via inline `#[cfg(test)]` declarations
- Typescript (`jest`) => via spec files
- Smoke Tests (run on merges to latest)
- eslint, clippy

### CI/CD

We recommend you read this article to understand better how we run our pipelines: https://www.jacobbolda.com/setting-up-ci-and-cd-for-tauri/

## Organization

Tauri aims to be a sustainable collective based on principles that guide [sustainable free and open software communities](https://sfosc.org). To this end it has become a Programme within the [Commons Conservancy](https://commonsconservancy.org/), and you can contribute financially via [Open Collective](https://opencollective.com/tauri).

## Semver

**tauri** is following [Semantic Versioning 2.0](https://semver.org/).

## Licenses

Code: (c) 2015 - 2021 - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.

Logo: CC-BY-NC-ND

- Original Tauri Logo Designs by [Alve Larsson](https://alve.io/), [Daniel Thompson-Yvetot](https://github.com/nothingismagick) and [Guillaume Chau](https://github.com/akryum)

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_large)
