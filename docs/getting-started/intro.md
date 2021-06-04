---
title: Introduction
---

import OSList from '@theme/OSList'

Welcome to Tauri! This guide will help you create your first Tauri app. It should only take about 10 minutes, although it could take longer if you have a slower internet connection.

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
