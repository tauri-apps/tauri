---
title: Introduction
---

import OSList from '@theme/OSList'

Welcome to Tauri!

Tauri is a polyglot and generic system that is very composable and allows engineers to make a wide variety of applications. It is used for building applications for Desktop Computers using a combination of [Rust](https://www.rust-lang.org/) tools and HTML rendered in a Webview. Apps built with Tauri can ship with any number of pieces of an optional JS API / Rust API so that webviews can control the system via message passing.

Anything that can be displayed on a website, can be displayed in a Tauri webview app!

Developers are free to build the web front-end displayed in a Webview through Tauri with any web frameworks of their choice!
**Developers can even extend the default API** with their own functionality and bridge the Webview and Rust-based backend easily!

The Architecture is more fully described in [Architecture](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md).

This guide will help you create your first Tauri app. It should only take about 10 minutes, although it could take longer if you have a slower internet connection.

If you find an error or something unclear, or would like to propose an improvement, you have several options:

1. Open an issue on our [Github Repo](https://github.com/tauri-apps/tauri-docs)
2. Visit our [Discord server](https://discord.gg/tauri) and raise your concern
3. Request to join the education working group on Discord to gain access to its discussion channel

## Steps

1. Install and configure system prerequisites
2. Create a web app with your frontend framework of choice
3. Use the Tauri CLI to setup Tauri in your app
4. Write native Rust code to add functionality or improve performance (totally optional)
5. Use `tauri dev` to develop your app with features like hot module reloading and webview devtools
6. Use `tauri build` to package your app into a tiny installer

### Setting up Your Environment

Before creating an app, you'll have to install and configure some developer tools. This guide assumes that you know what the command line is, how to install packages on your operating system, and generally know your way around the development side of computing.

Follow the platform-specific guides to get started:

<OSList content={{
    linux: { title: 'Linux Setup', link: '/docs/getting-started/setup-linux'},
    macos: { title: 'macOS Setup', link: '/docs/getting-started/setup-macos'},
    windows: { title: 'Windows Setup', link: '/docs/getting-started/setup-windows'}
}} />

After that, you'll be ready to [add Tauri to your project!](/docs/usage/development/integration)
