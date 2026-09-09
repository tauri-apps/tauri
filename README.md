<img src=".github/splash.png" alt="Tauri" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/tauri-apps/tauri/tree/dev)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![test core](https://img.shields.io/github/actions/workflow/status/tauri-apps/tauri/test-core.yml?label=test%20core\&logo=github)](https://github.com/tauri-apps/tauri/actions/workflows/test-core.yml)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_shield)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.com/invite/tauri)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![Good Labs](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## Introduction

Tauri is a framework for building tiny, blazingly fast binaries for all major operating systems. Developers can use any front end framework that compiles to HTML, JS, and CSS to build their user interface. The backend of the application is a Rust based binary that the front end can interact with through an API.

The user interface in Tauri apps currently uses [`tao`](https://docs.rs/tao) as a window handling library on macOS, Windows, Linux, Android, and iOS.

To render your application, Tauri uses [WRY](https://github.com/tauri-apps/wry), a library that provides a unified interface to system webviews. It uses WKWebView on macOS and iOS, WebView2 on Windows, WebKitGTK on Linux, and Android System WebView on Android.

To learn more about how all of these pieces fit together, please see the [architecture document](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md).

## Getting Started

If you are interested in making a Tauri app, please visit the [documentation website](https://tauri.app).

The quickest way to get started is to install the [prerequisites](https://v2.tauri.app/start/prerequisites/) for your system and create a new project with [`create tauri app`](https://github.com/tauri-apps/create-tauri-app/#usage).

For example, with `npm`:

```sh
npm create tauri-app@latest
```

## Building from Source

If you are contributing to Tauri or want to use the latest unreleased version, you can build the CLI from source.

Clone the repository and build the CLI:

```sh
git clone https://github.com/tauri-apps/tauri.git
cd tauri
cargo build -p tauri-cli
```

The built binary is located at `target/debug/cargo-tauri` (`cargo-tauri.exe` on Windows).

You can run it directly or use it as a Cargo subcommand.

For example:

```sh
./target/debug/cargo-tauri init
./target/debug/cargo-tauri dev
./target/debug/cargo-tauri build
```

When initializing a new project, the `init` command automatically detects whether it is running in an interactive terminal. If no terminal is attached, such as in CI or scripts, it skips all prompts and uses default values.

You can also use the `--ci` flag to force non interactive mode, or use `--force` to overwrite an existing `src-tauri` folder.

Prompt values can be changed with command line options such as `--app-name`, `--frontend-dist`, or `--dev-url`.

For example:

```sh
cargo-tauri init \
  --force \
  --app-name myapp \
  --frontend-dist ../dist \
  --dev-url http://localhost:3000
```

This creates the `src-tauri` directory without prompting for input.

## Features

The list of Tauri features includes, but is not limited to:

- Built in app bundler for creating app bundles in formats such as `.app`, `.dmg`, `.deb`, `.rpm`, `.AppImage`, and Windows installers such as `.exe` through NSIS and `.msi` through WiX.
- Built in updater for desktop applications
- System tray icons
- Native notifications
- Native WebView protocol. Tauri does not create a localhost HTTP or HTTPS server to serve the WebView contents.
- GitHub Actions for streamlined CI
- VS Code extension

### Platforms

Tauri currently supports development and distribution on the following platforms:

| Platform   | Versions                                                                                                    |
| :--------- | :---------------------------------------------------------------------------------------------------------- |
| Windows    | 7 and above                                                                                                 |
| macOS      | 10.15 and above                                                                                             |
| Linux      | WebKitGTK 4.0 for Tauri v1, for example Ubuntu 18.04. WebKitGTK 4.1 for Tauri v2, for example Ubuntu 22.04. |
| iOS/iPadOS | 9 and above                                                                                                 |
| Android    | 7 and above, currently 8 and above                                                                          |

## Contributing

Before you start working on something, it is best to check if there is an existing issue first. It is also a good idea to stop by the Discord server and confirm with the team if it makes sense or if someone else is already working on it.

Please make sure to read the [Contributing Guide](./.github/CONTRIBUTING.md) before making a pull request.

Thank you to everyone contributing to Tauri!

### Documentation

Documentation in a polyglot system is a tricky proposition. To this end, we prefer to use inline documentation in the Rust and JS source code as much as possible.

Check out the [Tauri documentation repository](https://github.com/tauri-apps/tauri-docs) for further information.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank" rel="noopener noreferrer">
          <img src=".github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors, please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## Organization

Tauri aims to be a sustainable collective based on principles that guide sustainable free and open software communities.

To this end, it has become a Programme within the [Commons Conservancy](https://commonsconservancy.org/), and you can contribute financially through [Open Collective](https://opencollective.com/tauri).

## Licenses

Code: (c) 2015 to Present, The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.

Logo: CC BY NC ND

- Original Tauri Logo Designs by [Alve Larsson](https://alve.io/), [Daniel Thompson Yvetot](https://github.com/nothingismagick), and [Guillaume Chau](https://github.com/akryum).

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Ftauri-apps%2Ftauri?ref=badge_large)