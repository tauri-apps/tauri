---
title: Setup for macOS
---

import Alert from '@theme/Alert'
import { Intro } from '@theme/SetupDocs'
import Icon from '@theme/Icon'

<Intro />

## 1. System Dependencies&nbsp;<Icon title="alert" color="danger"/>


You will need to have <a href="https://brew.sh/" target="_blank">Homebrew</a> installed to run the following command.

```sh
$ brew install gcc
```

You will also need to make sure `xcode` is installed.

```sh
$ xcode-select --install
```

## 2. Node.js Runtime and Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

### Node.js (npm included)

We recommend using nvm to manage your Node.js runtime. It allows you to easily switch versions and update Node.js.

```sh
$ curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh | bash
```

<Alert title="Note">
We have audited this bash script, and it does what it says it is supposed to do. Nevertheless, before blindly curl-bashing a script, it is always wise to look at it first. Here is the file as a mere <a href="https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.2/install.sh" target="_blank">download link</a>.
</Alert>

Once nvm is installed, close and reopen your terminal, then install the latest version of Node.js and npm:

```sh
$ nvm install node --latest-npm
$ nvm use node
```

If you have any problems with nvm, please consult their <a href="https://github.com/nvm-sh/nvm">project readme</a>.

### Optional Node.js Package Manager

You may want to use an alternative to npm:

- <a href="https://yarnpkg.com/getting-started" target="_blank">Yarn</a>, is preferred by Tauri's team
- <a href="https://pnpm.js.org/en/installation" target="_blank">pnpm</a>

## 3. Rustc and Cargo Package Manager&nbsp;<Icon title="control-skip-forward" color="warning"/>

The following command will install <a href="https://rustup.rs/" target="_blank">rustup</a>, the official installer for <a href="https://www.rust-lang.org/" target="_blank">Rust</a>.

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

<Alert title="Note">
We have audited this bash script, and it does what it says it is supposed to do. Nevertheless, before blindly curl-bashing a script, it is always wise to look at it first. Here is the file as a mere <a href="https://sh.rustup.rs" target="_blank">download link</a>.
</Alert>

To make sure that Rust has been installed successfully, run the following command:

```sh
$ rustc --version
latest update on 2019-12-19, rust version 1.40.0
```

You may need to restart your terminal if the command does not work.

## Continue

Now that you have set up the macOS-specific dependencies for Tauri, learn how to [add Tauri to your project](/docs/usage/development/integration).
