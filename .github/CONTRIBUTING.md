# Tauri Contributing Guide

Hi! We, the maintainers, are really excited that you are interested in contributing to Tauri. Before submitting your contribution though, please make sure to take a moment and read through the [Code of Conduct](CODE_OF_CONDUCT.md), as well as the appropriate section for the contribution you intend to make:

- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Development Guide](#development-guide)

## Issue Reporting Guidelines

- The issue list of this repo is **exclusively** for bug reports and feature requests. Non-conforming issues will be closed immediately.

- If you have a question, you can get quick answers from the [Tauri Discord chat](https://discord.gg/SpmNs4S).

- Try to search for your issue, it may have already been answered or even fixed in the development branch (`dev`).

- Check if the issue is reproducible with the latest stable version of Tauri. If you are using a pre-release, please indicate the specific version you are using.

- It is **required** that you clearly describe the steps necessary to reproduce the issue you are running into. Although we would love to help our users as much as possible, diagnosing issues without clear reproduction steps is extremely time-consuming and simply not sustainable.

- Use only the minimum amount of code necessary to reproduce the unexpected behavior. A good bug report should isolate specific methods that exhibit unexpected behavior and precisely define how expectations were violated. What did you expect the method or methods to do, and how did the observed behavior differ? The more precisely you isolate the issue, the faster we can investigate.

- Issues with no clear repro steps will not be triaged. If an issue labeled "need repro" receives no further input from the issue author for more than 5 days, it will be closed.

- If your issue is resolved but still open, don’t hesitate to close it. In case you found a solution by yourself, it could be helpful to explain how you fixed it.

- Most importantly, we beg your patience: the team must balance your request against many other responsibilities — fixing other bugs, answering other questions, new features, new documentation, etc. The issue list is not paid support and we cannot make guarantees about how fast your issue can be resolved.

## Pull Request Guidelines

- You have to [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

- It's OK to have multiple small commits as you work on the PR - we will let GitHub automatically squash it before merging.

- If adding new feature:

  - Provide convincing reason to add this feature. Ideally you should open a suggestion issue first and have it greenlighted before working on it.

- If fixing a bug:
  - If you are resolving a special issue, add `(fix: #xxxx[,#xxx])` (#xxxx is the issue id) in your PR title for a better release log, e.g. `fix: update entities encoding/decoding (fix #3899)`.
  - Provide detailed description of the bug in the PR, or link to an issue that does.

## Development Guide

**NOTE: Tauri is undergoing rapid development right now, and the docs match the latest published version of Tauri. They are horribly out of date when compared with the code in the dev branch. This contributor guide is up-to-date, but it doesn't cover all of Tauri's functions in depth. If you have any questions, don't hesitate to ask in our Discord server.**

### General Setup

First, [join our Discord server](https://discord.gg/SpmNs4S) and let us know that you want to contribute. This way we can point you in the right direction and help ensure your contribution will be as helpful as possible.

To set up your machine for development, follow the [Tauri setup guide](https://tauri.studio/guides/getting-started/prerequisites/) to get all the tools you need to develop Tauri apps. The only additional tool you may need is [Yarn](https://yarnpkg.com/), it is only required if you are developing the Node CLI or API packages (`tooling/cli/node` and `tooling/api`). Next, fork and clone this repo. It is structured as a monorepo, which means that all the various Tauri packages are under the same repository. The development process varies depending on what part of Tauri you are contributing to, see the guides below for per-package instructions.

Some Tauri packages will be automatically built when running one of the examples. Others, however, will need to be built beforehand. To build these automatically, run the `.scripts/setup.sh` (Linux and macOS) or `.scripts/setup.ps1` (Windows) script. This will install the Rust and Node.js CLI and build the JS API. After that, you should be able to run all the examples. Note that the setup script should be executed from the root folder of the respository in order to run correctly.

### Packages Overview

- The JS API (`/tooling/api`) contains JS bindings to the builtin Rust functions in the Rust API.
- cli.rs (`/tooling/cli`) is the primary CLI for creating and developing Tauri apps.
- cli.js (`/tooling/cli/node`) is a Node.js CLI wrapper for `cli.rs`.
- Tauri Bundler (`/tooling/bundler`) is used by the Rust CLI to package executables into installers.
- Tauri Core (`/core/tauri`) is the heart of Tauri. It contains the code that starts the app, configures communication between Rust and the Webview, and ties all the other packages together.
- The Macros (`/core/tauri-macros`) are used by Tauri Core for various functions.

### Developing The Node.js CLI (cli.js)

`cli.js` is a wrapper to `cli.rs` so most changes should be written on the Rust CLI. The `[Tauri repo root]/tooling/cli/node` folder contains only packaging scripts to properly publish the Rust CLI binaries to NPM.

### Developing Tauri Bundler and Rust CLI

The code for the bundler is located in `[Tauri repo root]/tooling/bundler`, and the code for the Rust CLI is located in `[Tauri repo root]/tooling/cli`. If you are using your local copy of cli.js (see above), any changes you make to the bundler and CLI will be automatically built and applied when running the build or dev command. Otherwise, running `cargo install --path .` in the Rust CLI directory will allow you to run `cargo tauri build` and `cargo tauri dev` anywhere, using the updated copy of the bundler and cli. You will have to run this command each time you make a change in either package.

### Developing Tauri Core and Related Components (Rust API, Macros, Codegen, and Utils)

The code for Tauri Core is located in `[Tauri repo root]/core/tauri`, and the Rust API, Macros, and Utils are in `[Tauri repo root]/core/tauri-(api/macros/utils)`. The easiest way to test your changes is to use the `[Tauri repo root]/examples/helloworld` app. It automatically rebuilds and uses your local copy of the Tauri core packages. Just run `yarn tauri build` or `yarn tauri dev` in the helloworld app directory after making changes to test them out. To use your local changes in another project, edit its `src-tauri/Cargo.toml` file so that the `tauri` key looks like `tauri = { path = "PATH", features = [ "api-all", "cli" ] }`, where `PATH` is the relative path to `[Tauri repo root]/core/tauri`. Then, your local copy of the Tauri core packages will be rebuilt and used whenever you build that project.

#### Building the documentation locally

You can build the Rust documentation locally running the following script:

```bash
$ RUSTDOCFLAGS="--cfg doc_cfg" cargo +nightly doc --all-features --open
```

### Developing the JS API

The JS API provides bindings between the developer's JS in the Webview and the builtin Tauri APIs, written in Rust. Its code is located in `[Tauri repo root]/tooling/api`. After making changes to the code, run `yarn build` to build it. To test your changes, we recommend using the API example app, located in `[Tauri repo root]/examples/api`. It will automatically use your local copy of the JS API and provides a helpful UI to test the various commands.

## Financial Contribution

Tauri is an MIT-licensed open source project. Its ongoing development can be supported via [Github Sponsors](https://github.com/sponsors/nothingismagick) or [Open Collective](https://opencollective.com/tauri). We prefer Github Sponsors as donations made are doubled through the matching fund program.
