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

- If your issue is resolved but still open, don't hesitate to close it. In case you found a solution by yourself, it could be helpful to explain how you fixed it.

- Most importantly, we beg your patience: the team must balance your request against many other responsibilities — fixing other bugs, answering other questions, new features, new documentation, etc. The issue list is not paid support and we cannot make guarantees about how fast your issue can be resolved.

## Pull Request Guidelines

- You have to [sign your commits](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).

- It's OK to have multiple small commits as you work on the PR - we will let GitHub automatically squash it before merging.

- If adding new feature:
  - Provide convincing reason to add this feature. Ideally you should open a suggestion issue first and have it greenlighted before working on it.

- If fixing a bug:
  - If you are resolving a special issue, add `(fix: #xxxx[,#xxx])` (#xxxx is the issue id) in your PR title for a better release log, e.g. `fix: update entities encoding/decoding (fix #3899)`.
  - Provide detailed description of the bug in the PR, or link to an issue that does.

- If the PR is meant to be released, follow the instructions in `.changes/readme.md` to log your changes. ie. [readme.md](https://github.com/tauri-apps/tauri/blob/dev/.changes/README.md)

## Development Guide

**NOTE: If you have any question don't hesitate to ask in our Discord server. We try to keep this guide to up guide, but if something doesn't work let us know.**

### General Setup

First, [join our Discord server](https://discord.gg/SpmNs4S) and let us know that you want to contribute. This way we can point you in the right direction and help ensure your contribution will be as helpful as possible.

To set up your machine for development, follow the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/) to get all the tools you need to develop Tauri apps. The only additional tool you may need is [PNPM](https://pnpm.io/), it is only required if you are developing the Node CLI or API packages (`packages/cli` and `packages/api`).

Next, [fork](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/fork-a-repo) and clone [this repository](https://github.com/tauri-apps).
The development process varies depending on what part of Tauri you are contributing to, see the guides below for per-package instructions.

Some Tauri packages will be automatically built when running one of the examples. Others, however, will need to be built beforehand. To initialize, execute these commands in the repository root:

```bash
pnpm install
pnpm build
```

### Overview

See [Architecture](../ARCHITECTURE.md#major-components) for an overview of the packages in this repository.

### Developing Tauri Core and Related Components (Rust API, Macros, Codegen, and Utils)

The code for the Rust crates, including the Core, Macros, Utils, WRY runtime, and a few more are located in the [main Tauri repository](https://github.com/tauri-apps/tauri/tree/dev/crates).

The easiest way to test your changes is to use the [helloworld](https://github.com/tauri-apps/tauri/tree/dev/examples/helloworld) example app. It automatically rebuilds and uses your local copy of the Tauri core packages. Just run `cargo run --example helloworld` after making changes to test them out.

To test local changes against your own application simply point the Tauri create to your local repository. In `src-tauri/Cargo.toml` file change:

`tauri = { version = "2.1.1" }`

to:

`tauri = { path = "path/to/local/tauri/crates/tauri" }`

If any other crates depend on Tauri you will have to point them to the local repo as well.

### Developing Tauri Bundler and Rust CLI

The code for the bundler is located in [crates/tauri-bundler](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri-bundler), and the code for the Rust CLI is located in [tauri-cli](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri-cli).
Running `cargo install --path .` in the Rust CLI directory will allow you to run `cargo tauri build` and `cargo tauri dev` anywhere, using the updated copy of the bundler and cli. You will have to run this command each time you make a change in either package.
You can use `cargo install --path . --debug` to speed up test builds.

### Developing The Node.js CLI (`@tauri-apps/cli`)

[`@tauri-apps/cli`](https://github.com/tauri-apps/tauri/tree/dev/packages/cli) is a small wrapper around `tauri-cli` so most changes should be happen in the Rust CLI (see above).

#### Building the documentation locally

You can build the Rust documentation locally running the following script:

```bash
$ cargo +nightly doc --all-features --open
```

### Developing the JS API

The JS API provides bindings between the developer's JS in the Webview and the built-in Tauri APIs, written in Rust. Its code is located in [/packages/api](https://github.com/tauri-apps/tauri/tree/dev/packages/api).
After making changes to the code, run `pnpm build` to build it. To test your changes, we recommend using the API example app, located in [/examples/api](https://github.com/tauri-apps/tauri/tree/dev/examples/api). It will automatically use your local copy of the JS API and provides a helpful UI to test the various commands.

## Financial Contribution

Tauri is an MIT-licensed open source project. Its ongoing development can be supported via [GitHub Sponsors](https://github.com/sponsors/tauri-apps) or [Open Collective](https://opencollective.com/tauri). We prefer GitHub Sponsors as donations made are doubled through the matching fund program.
