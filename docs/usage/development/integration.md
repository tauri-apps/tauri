---
title: Tauri Integration
sidebar_label: 'Tauri Integration (1/4)'
---

import Alert from '@theme/Alert'
import Command from '@theme/Command'
import Link from '@docusaurus/Link'

<Alert title="Please note" type="warning" icon="alert">
  You must have completed all the steps required for setting up the development environment on your machine. If you haven't done this yet, please see the <a href="/docs/getting-started/intro#setting-up-your-environment"> setup page for your operating system</a>.
</Alert>

### 1. Install Tauri JS Package as a Dependency:

```bash
cd project-folder

# Not required if you already have a package.json:
# yarn init
# OR
# npm init

yarn add @tauri-apps/cli
# OR
npm install @tauri-apps/cli
```

<Alert title="Note">
  You can install Tauri as both a local and a global dependency, but we recommend installing it locally.
</Alert>

If you decide to use Tauri as a local package with npm (not yarn), you will have to define a custom script to your package.json:

```js title=package.json
{
  // This content is just a sample
  "scripts": {
    "tauri": "tauri"
  }
}
```

### 2. Initialize Tauri in Your App

<Command name="init" />

This command will place a new folder in your current working directory, `src-tauri`.

```sh
└── src-tauri
    ├── .gitignore
    ├── Cargo.toml
    ├── rustfmt.toml
    ├── tauri.conf.json
    ├── icons
    │   ├── 128x128.png
    │   ├── 128x128@2x.png
    │   ├── 32x32.png
    │   ├── Square107x107Logo.png
    │   ├── Square142x142Logo.png
    │   ├── Square150x150Logo.png
    │   ├── Square284x284Logo.png
    │   ├── Square30x30Logo.png
    │   ├── Square310x310Logo.png
    │   ├── Square44x44Logo.png
    │   ├── Square71x71Logo.png
    │   ├── Square89x89Logo.png
    │   ├── StoreLogo.png
    │   ├── icon.icns
    │   ├── icon.ico
    │   └── icon.png
    └── src
        ├── build.rs
        ├── cmd.rs
        └── main.rs
```

### 3. Check `tauri info` to Make Sure Everything Is Set up Properly:

<Command name="info" />

Which should return something like:

```
Operating System - Darwin(16.7.0) - darwin/x64

Node.js environment
  Node.js - 12.14.0
  tauri.js - 0.2.1

Rust environment
  rustc - 1.40.0
  cargo - 1.40.0
  tauri-cli - 0.1.2

Global packages
  NPM - 6.13.4
  yarn - 1.21.1

App directory structure
/.quasar
/build-tauri
/dist
/node_modules
/src

App
  tauri - 0.3.0
  build-type - bundle
  CSP - default-src data: filesystem: ws: http: https: 'unsafe-eval' 'unsafe-inline'
  Windows - Edge
  distDir - ../dist/spa
  devPath - http://localhost:7334
```

This information can be very helpful when triaging problems.

### Patterns

We've also defined prebuilt configurations called "Patterns". They may help you to customize Tauri to fit your needs.
[See more about patterns](/docs/usage/patterns/about-patterns).

## Vue CLI Plugin Tauri

If you are using Vue CLI, it is recommended to use the official [CLI plugin](https://github.com/tauri-apps/vue-cli-plugin-tauri).
