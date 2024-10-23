# Tauri CLI

 <img align="right" src="https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png" height="128" width="128">

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test cli](https://img.shields.io/github/actions/workflow/status/tauri-apps/tauri/test-cli-rs.yml?label=test%20cli&logo=github)](https://github.com/tauri-apps/tauri/actions/workflows/test-cli-rs.yml)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

| Component | Version                                                                                                |
| --------- | ------------------------------------------------------------------------------------------------------ |
| tauri-cli | [![](https://img.shields.io/crates/v/tauri-cli?style=flat-square)](https://crates.io/crates/tauri-cli) |

## About Tauri

Tauri is a polyglot and generic system that is very composable and allows engineers to make a wide variety of applications. It is used for building applications for Desktop Computers using a combination of Rust tools and HTML rendered in a Webview. Apps built with Tauri can ship with any number of pieces of an optional JS API / Rust API so that webviews can control the system via message passing. In fact, developers can extend the default API with their own functionality and bridge the Webview and Rust-based backend easily.

Tauri apps can have custom menus and have tray-type interfaces. They can be updated, and are managed by the user's operating system as expected. They are very small, because they use the system's webview. They do not ship a runtime, since the final binary is compiled from rust. This makes the reversing of Tauri apps not a trivial task.

## This module

This rust executable provides the full interface to all of the required activities for which the CLI is required. It will run on macOS, Windows, and Linux.

To learn more about the details of how all of these pieces fit together, please consult this [ARCHITECTURE.md](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md) document.

## Semver

**tauri** is following [Semantic Versioning 2.0](https://semver.org/).

## Licenses

Code: (c) 2015 - 2021 - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.

Logo: CC-BY-NC-ND

- Original Tauri Logo Designs by [Daniel Thompson-Yvetot](https://github.com/nothingismagick) and [Guillaume Chau](https://github.com/akryum)

## Licensing Errata:

Because of publishing issues upstream, we soft-forked (and patched) both [`console`](https://github.com/mitsuhiko/console/blob/278de9dc2bf0fa28db69adee351072f668beec8f/Cargo.toml#L7) and [`dialoguer`](https://github.com/mitsuhiko/dialoguer/blob/2c3fe6b64641cfb57eb0e1d428274f63976ec150/Cargo.toml#L12) crates because of untenable issues surrounding expected use on Windows.

This soft fork was introduced to the Tauri Codebase [here](https://github.com/tauri-apps/tauri/pull/1610).

`console`

```
license = "MIT"
authors = [
	"Armin Ronacher <armin.ronacher@active-4.com>"
]
```

`dialoguer`

```
license = "MIT"
authors = [
	"Armin Ronacher <armin.ronacher@active-4.com>",
	"Pavan Kumar Sunkara <pavan.sss1991@gmail.com>"
]
```
