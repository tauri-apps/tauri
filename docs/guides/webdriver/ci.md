---
title: Continuous Integration
---

Utilizing Linux and some programs to create a fake display, it is possible to run [WebDriver] tests with
[`tauri-driver`] on your CI. The following example will use the [WebdriverIO] example we [previously built together] and
GitHub Actions.

This means the following assumptions:

1. The Tauri application is in the repository root and the binary builds when running `cargo build --release`.
2. The [WebDriverIO] test runner is in the `webdriver/webdriverio` directory and runs when `yarn test` is used in that
   directory.

The following is a commented GitHub Actions workflow file at `.github/workflows/webdriver.yml`

```yaml
# run this action when the repository is pushed to
on: [ push ]

# the name of our workflow
name: WebDriver

jobs:
  # a single job named test
  test:
    # the display name the test job
    name: WebDriverIO Test Runner

    # we want to run on the latest linux environment
    runs-on: ubuntu-latest

    # the steps our job runs **in order**
    steps:
      # checkout the code on the workflow runner
      - uses: actions/checkout@v2

      # install system dependencies that Tauri needs to compile on Linux.
      # note the extra dependencies for `tauri-driver` to run which are `webkit2gtk-driver` and  `xvfb`
      - name: Tauri dependencies
        run: >-
          sudo apt-get update &&
          sudo apt-get install -y
          libgtk-3-dev
          libgtksourceview-3.0-dev
          webkit2gtk-4.0
          libappindicator3-dev
          webkit2gtk-driver
          xvfb

      # install the latest Rust stable
      - name: Rust stable
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      # we run our rust tests before the webdriver tests to avoid testing a broken application
      - name: Cargo test
        uses: actions-rs/cargo@v1
        with:
          command: test

      # build a release build of our application to be used during our WebdriverIO tests
      - name: Cargo build
        uses: actions-rs/cargo@v1
        with:
          command: build
          args: --release

      # install the latest stable node version at the time of writing
      - name: Node v16
        uses: actions/setup-node@v2
        with:
          node-version: 16.x

      # install our Node.js dependencies with Yarn
      - name: Yarn install
        run: yarn install
        working-directory: webdriver/webdriverio

      # install the latest version of `tauri-driver`.
      # note: the tauri-driver version is independent of any other Tauri versions
      - name: Install tauri-driver
        uses: actions-rs/cargo@v1
        with:
          command: install
          args: tauri-driver

      # run the WebdriverIO test suite.
      # we run it through `xvfb-run` (the dependency we installed earlier) to have a fake
      # display server which allows our application to run headless without any changes to the code
      - name: WebdriverIO
        run: xvfb-run yarn test
        working-directory: webdriver/webdriverio
```

[WebDriver]: https://www.w3.org/TR/webdriver/
[`tauri-driver`]: https://crates.io/crates/tauri-driver
[WebdriverIO]: https://webdriver.io/
[previously built together]: example/webdriverio
