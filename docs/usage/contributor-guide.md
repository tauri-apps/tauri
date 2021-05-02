---
title: Contributor Guide
---

todo: make this friendlier and more complete

Tauri is a polyglot system that uses:

- git
- Node.js
- Rust
- GitHub actions

It can be developed on macOS, Linux and Windows.

## Contribution Flow

1. File an Issue
2. Fork the Repository
3. Make Your Changes
4. Make a PR

### A Note About Contributions to the Rust Libraries

When contributing to the Rust libraries `tauri`, `tauri-api`, and `tauri-updater`; you will want to setup an environment for RLS (the Rust Language Server). In the Tauri root directory, there is a `.scripts` folder that contains a set of scripts to automate adding a couple temporary environment variables to your shell/terminal. These environment variables point to directories in the test fixture which will prevent RLS from crashing on compile-time. This is a necessary step for setting up a development environment for Tauri's Rust libraries.

##### _Example Instructions_

1. Navigate to the Tauri Root directory.
2. Execute a script based on your Operating System from this folder: `.scripts/init_env.bat` for Windows Cmd, `.scripts/init_env.ps1` for Windows Powershell, `. .scripts/init_env.sh` for Linux/macOS bash (note the first `.` in this command).
3. Open your text editor/IDE from this shell/terminal.

## Hands On Example

Let's make a new example. That's a great way to learn. We are going to assume you are on a nixy type of environment like Linux or macOS and have all of your development dependencies like rust and node already sorted out.

```sh
git clone git@github.com:tauri-apps/tauri.git
cd tauri/cli/tauri.js
yarn
mkdir ../../examples/vanillajs && cd "$_"
```

```json
  "tauri:source": "node ../../../cli/tauri.js/bin/tauri",
```

```ini
  [dependencies.tauri]
  path = "../../../../core/tauri"
  features = [ "all-api" ]
```
