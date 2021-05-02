---
title: Setup for Windows
---

import Alert from '@theme/Alert'
import Icon from '@theme/Icon'
import { Intro } from '@theme/SetupDocs'

<Alert title="Note">

For those using the Windows Subsystem for Linux (WSL) please refer to our [Linux specific instructions](/docs/getting-started/setup-linux) instead.
</Alert>

<Intro />

## 1. System Dependencies&nbsp;<Icon title="alert" color="danger"/>

You'll need to install Microsoft Visual Studio C++ build tools. <a href="https://visualstudio.microsoft.com/visual-cpp-build-tools/" target="_blank">Download the installer here</a>, and then run it. When it asks you what packages you would like to install, select C++ Build Tools.

<Alert title="Note">
This is a big download (over 1GB) and takes the most time, so go grab a coffee.
</Alert>

<Alert type="warning">
You may need to uninstall the 2017 version of the build tools if you have them. There are reports of Tauri not working with both the 2017 and 2019 versions installed.
</Alert>

## 2. Node.js Runtime and Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

### Node.js (npm included)

We recommend using <a href="https://github.com/coreybutler/nvm-windows#installation--upgrades" target="_blank">nvm-windows</a> to manage your Node.js runtime. It allows you to easily switch versions and update Node.js.

Then run the following from an Administrative PowerShell and press Y when prompted:

```powershell
# BE SURE YOU ARE IN AN ADMINISTRATIVE PowerShell!
nvm install latest
nvm use {{latest}} # Replace with your latest downloaded version
```

This will install the most recent version of Node.js with npm.

### Optional Node.js Package Manager

You may want to use an alternative to npm:

- <a href="https://yarnpkg.com/getting-started" target="_blank">Yarn</a>, is preferred by Tauri's team
- <a href="https://pnpm.js.org/en/installation" target="_blank">pnpm</a>

## 3. Rustc and Cargo Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

Now you will need to install <a href="https://www.rust-lang.org/" target="_blank">Rust</a>. The easiest way to do this is to use <a href="https://rustup.rs/" target="_blank">rustup</a>, the official installer.

- <a href="https://win.rustup.rs/x86_64" target="_blank">64-bit download link</a>
- <a href="https://win.rustup.rs/i686" target="_blank">32-bit download link</a>

Download and install the proper variant for your computer's architecture.

## 4. Install WebView2

Finally, you will need to install WebView2. The best way to do this is to download and run the Evergreen Bootstrapper from [this page](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section).

## Continue

Now that you have set up the Windows-specific dependencies for Tauri, learn how to [add Tauri to your project](/docs/usage/development/integration).
