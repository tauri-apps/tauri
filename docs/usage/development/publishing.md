---
title: App Publishing
sidebar_label: 'App Publishing (4/4)'
---

import Alert from '@theme/Alert'
import Command from '@theme/Command'

### 1. Build Your Web App

Now that you are ready to package your project, you will need to run your framework's or bundler's build command (assuming you're using one, of course).

<Alert title="Note">
Every framework has its own publishing tooling. It is outside of the scope of this document to treat them all or keep them up to date.
</Alert>

### 2. Bundle your application with Tauri

<Command name="build" />

This command will embed your web assets into a single binary with your Rust code. The binary itself will be located in `src-tauri/target/release/[app name]`, and installers will be located in `src-tauri/target/release/bundle/`.

Like the `tauri dev` command, the first time you run this, it will take some time to collect the Rust crates and build everything - but on subsequent runs it will only need to rebuild your code, which is much quicker.
