---
title: App Development
sidebar_label: 'App Development (2/4)'
---

import Alert from '@theme/Alert'
import Command from '@theme/Command'

### 1. Start Your Devserver

Now that you have everything setup, you should start your application development server provided by your UI framework or bundler (assuming you're using one, of course).

<Alert title="Note">
Every framework has its own development tooling. It is outside of the scope of this document to treat them all or keep them up to date.
</Alert>

### 2. Start Tauri Development Window

<Command name="dev" />

The first time you run this command, it will take several minutes for the Rust package manager to download and build all the required packages. Since they are cached, subsequent builds will be much faster, as only your code will need rebuilding.

Once Rust has finished building, the webview will open and it should display your web app. You can make changes to your web app, and if your tooling enables it, the webview should update automatically just like a browser. When you make changes to your Rust files, they will be rebuilt automatically and your app will restart.

<Alert title="A note about Cargo.toml and Source Control" icon="info-alt">
  In your project repository, you SHOULD commit the "src-tauri/Cargo.toml" to git because you want it to be deterministic. You SHOULD NOT commit the "src-tauri/target" folder or any of its contents.
</Alert>
