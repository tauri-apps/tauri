# Tauri Contributing Guide

Hi! We, the maintainers, are really excited that you are interested in contributing to Tauri. Before submitting your contribution though, please make sure to take a moment and read through the following guidelines.

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Development Guide](#development-guide)
- [Project Structure](#project-structure)
- [Financial Contribution](#financial-contribution)

## Issue Reporting Guidelines

- The issue list of this repo is **exclusively** for bug reports and feature requests. Non-conforming issues will be closed immediately.

- For simple beginner questions, you can get quick answers from the [Tauri Discord chat](https://discord.gg/SpmNs4S).

- Try to search for your issue, it may have already been answered or even fixed in the development branch (`dev`).

- Check if the issue is reproducible with the latest stable version of Tauri. If you are using a pre-release, please indicate the specific version you are using.

- It is **required** that you clearly describe the steps necessary to reproduce the issue you are running into. Although we would love to help our users as much as possible, diagnosing issues without clear reproduction steps is extremely time-consuming and simply not sustainable.

- Use only the minimum amount of code necessary to reproduce the unexpected behavior. A good bug report should isolate specific methods that exhibit unexpected behavior and precisely define how expectations were violated. What did you expect the method or methods to do, and how did the observed behavior differ? The more precisely you isolate the issue, the faster we can investigate.

- Issues with no clear repro steps will not be triaged. If an issue labeled "need repro" receives no further input from the issue author for more than 5 days, it will be closed.

- If your issue is resolved but still open, don’t hesitate to close it. In case you found a solution by yourself, it could be helpful to explain how you fixed it.

- Most importantly, we beg your patience: the team must balance your request against many other responsibilities — fixing other bugs, answering other questions, new features, new documentation, etc. The issue list is not paid support and we cannot make guarantees about how fast your issue can be resolved.

## Pull Request Guidelines

- The `latest` branch is basically just a snapshot of the latest stable release. All development should be done in dedicated branches. **Do not submit PRs against the `latest` branch.**

- Checkout a topic branch from the relevant branch, e.g. `dev`, and merge back against that branch.

- **DO NOT** checkin `dist` in the commits.

- It's OK to have multiple small commits as you work on the PR - we will let GitHub automatically squash it before merging.

- If adding new feature:

  - Provide convincing reason to add this feature. Ideally you should open a suggestion issue first and have it greenlighted before working on it.

- If fixing a bug:
  - If you are resolving a special issue, add `(fix: #xxxx[,#xxx])` (#xxxx is the issue id) in your PR title for a better release log, e.g. `fix: update entities encoding/decoding (fix #3899)`.
  - Provide detailed description of the bug in the PR. Live demo preferred.

## Development Guide

### General Setup

First, [join our Discord server](https://discord.gg/SpmNs4S) and let us know that you want to contribute. This way we can point you in the right direction and help ensure your contribution will be as helpful as possible. We also recommend you read the [technical details page](https://tauri.studio/en/docs/getting-started/technical-details) to learn how Tauri works under the hood and familiarize yourself with the codebase.

To set up your machine for development, follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro#setting-up-your-environment) to get all the tools you need to develop Tauri apps. The only additional tool you may need is [Yarn](https://yarnpkg.com/), it is only required if you are developing the Node CLI/API (`tauri.js`). Next, clone the Tauri repo. It is structured as a monorepo, which means that all the various Tauri packages are under the same repository. The development process varies depending on what part of Tauri you are contributing to.

### Developing The CLI and API (`tauri.js`)

The code for `tauri.js` is located in `[Tauri repo root]/cli/tauri.js`. Open a terminal, `cd` into that directory, and install deps by running `yarn install`. The code for the API (ie notifications, filesystem, etc...) is in `api-src` (not `src/api`), and the code for the CLI (the build, dev, init, etc... commands) is in `src`. There are a few package scripts you should be aware of:

- `build` builds both the API and CLI
- `build:api` builds the API
- `build:webpack` builds the CLI
- `test` runs the unit and e2e test suite
- `lint` runs ESLint to catch linting errors
- `format` formats code with Prettier to match the style guide

To test your changes, we recommend using the `[Tauri repo root]/tauri/examples/communication` app. It automatically uses the local version of `tauri.js`. You will need to rebuild `tauri.js` after every change by running `yarn build` in the `tauri.js` directory.

If you want to use your local code in another app, we recommend using [Yarn link](https://classic.yarnpkg.com/en/docs/cli/link/). First, run `yarn link` in the `tauri.js` directory, then run `yarn link tauri` in your test project's directory. This will link the CLI and API for that project. To run CLI commands, use `yarn tauri [command name]`, ie `yarn tauri build`. You only need to link once, but will need to rebuild every time.

### Developing Tauri Bundler

The code for the bundler is located in `[Tauri repo root]/cli/tauri-bundler`. After making your changes to the code, run `cargo install --path .` in the bundler directory. This will update the global `tauri-bundler` Cargo install to use your local code. Now, all of your Tauri projects will use the local code when bundling. The Cargo install needs to be run after every change.

### Developing Tauri Core

The code for Tauri core is located in `[Tauri repo root]/tauri`. The easiest way to test your changes is to use the `[Tauri repo root]/tauri/examples/communication` app. It automatically rebuilds and uses your local codebase. Just run `yarn tauri build` or `yarn tauri dev` in the communication app directory after making changes to test them out. To use your local changes in another project, edit its `src-tauri/Cargo.toml` file so that the `tauri` key looks like `tauri = { path = "PATH", features = [ "all-api", "cli" ] }`, where `PATH` is the relative path to `[Tauri repo root]/tauri`.

## Financial Contribution

Tauri is an MIT-licensed open source project. Its ongoing development can be supported via [Github Sponsors](https://github.com/sponsors/nothingismagick) or [Open Collective](https://opencollective.com/tauri). We prefer Github Sponsors as donations made are doubled through the matching fund program.
