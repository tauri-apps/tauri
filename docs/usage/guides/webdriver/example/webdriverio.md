---
title: WebdriverIO
---
import Alert from '@theme/Alert'
import Tabs from '@theme/Tabs'
import TabItem from '@theme/TabItem'

<Alert title="Example Application" type="info" icon="info-alt">

This [WebdriverIO] guide expects you to have already gone through the [example Application setup] in order to follow
step-by-step. The general information may still be useful otherwise.
</Alert>

This WebDriver testing example will use [WebdriverIO] and its testing suite. It is expected to already have Node.js
installed, along with `npm` or `yarn` although the [finished example project] uses `yarn`.

## Create a Directory for the Tests

Let's start off by creating a space in our project to write these tests. We are going to be using a nested directory for
this example project as we will later also go over other frameworks, but typically you will only need to use one. Create
the directory we will use with `mkdir -p webdriver/webdriverio`. The rest of this guide will assume you are inside the
`webdriver/webdriverio` directory.

## Initializing a WebdriverIO Project

We will be using a pre-existing `package.json` to bootstrap this test suite because we have already chosen specific
[WebdriverIO] config options and want to showcase a simple working solution. The bottom of this section has a collapsed
guide on how to set it up from scratch.

`package.json`:
```json
{
  "name": "webdriverio",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test": "wdio run wdio.conf.js"
  },
  "dependencies": {
    "@wdio/cli": "^7.9.1"
  },
  "devDependencies": {
    "@wdio/local-runner": "^7.9.1",
    "@wdio/mocha-framework": "^7.9.1",
    "@wdio/spec-reporter": "^7.9.0"
  }
}
```

We have a script which runs a [WebdriverIO] config as a test suite exposed as the `test` command. We also have various
dependencies that were added by the `@wdio/cli` command when we first set it up. In short, these dependencies are for
the most simple setup using a local WebDriver runner, [Mocha] as the test framework, and a simple Spec Reporter.

<details><summary>Click me if you want to see how to set a project up from scratch</summary>

The CLI is interactive, and you may choose the tools to work with yourself. Note that you will likely diverge from
the rest of the guide, and need to set up the differences yourself.

Let's add the [WebdriverIO] CLI to this npm project.

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npm install @wdio/cli
```

</TabItem>

<TabItem value="yarn">

```sh
yarn add @wdio/cli
```

</TabItem>
</Tabs>

To then run the interactive config command to set up a [WebdriverIO] test suite, you can then run:

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npx wdio config
```

</TabItem>

<TabItem value="yarn">

```sh
yarn wdio config
```

</TabItem>
</Tabs>

</details>

## Config

You may have noticed that the `test` script in our `package.json` mentions a file `wdio.conf.js`. That's the [WebdriverIO]
config file which controls most aspects of our testing suite.

`wdio.conf.js`:
```js
const os = require("os");
const path = require("path");
const { spawn, spawnSync } = require("child_process");

// keep track of the `tauri-driver` child process
let tauriDriver;

exports.config = {
  specs: ["./test/specs/**/*.js"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: "../../target/release/hello-tauri-webdriver",
        args: ["app", "cli", "args"]
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  // ensure the rust project is built since we expect this binary to exist for the webdriver sessions
  onPrepare: () => spawnSync("cargo", ["build", "--release"]),

  // ensure we are running `tauri-driver` before the session starts so that we can proxy the webdriver requests
  beforeSession: () =>
    (tauriDriver = spawn(
      path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver"),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    )),

  // clean up the `tauri-driver` process we spawned at the start of the session
  afterSession: () => tauriDriver.kill(),
};
```

If you are interested in the properties on `exports.config` object, then I [suggest reading the documentation] for it.
For non-WDIO specific items, there are comments explaining why we are running commands in `onPrepare`, `beforeSession`,
and `afterSession`. We also have our specs set to `"./test/specs/**/*.js"`, so let's create a spec now.

## Spec

A spec contains the code that is testing your actual application. The test runner will load these specs and automatically
run them as it sees fit. Let's create our spec now in the directory we specified.

`test/specs/example.e2e.js`:
```js
// calculates the luma from a hex color `#abcdef`
function luma(hex) {
  if (hex.startsWith("#")) {
    hex = hex.substring(1);
  }

  const rgb = parseInt(hex, 16);
  const r = (rgb >> 16) & 0xff;
  const g = (rgb >> 8) & 0xff;
  const b = (rgb >> 0) & 0xff;
  return  0.2126 * r + 0.7152 * g + 0.0722 * b;
}

describe("Hello Tauri", () => {
  it("should be cordial", async () => {
    const header = await $("body > h1");
    const text = await header.getText();
    expect(text).toMatch(/^[hH]ello/);
  });

  it("should be excited", async () => {
    const header = await $("body > h1");
    const text = await header.getText();
    expect(text).toMatch(/!$/);
  });

  it("should be easy on the eyes", async () => {
    const body = await $("body");
    const backgroundColor = await body.getCSSProperty("background-color");
    expect(luma(backgroundColor.parsed.hex)).toBeLessThan(100);
  });
});
```

The `luma` function on top is just a helper function for one of our tests and is not related to the actual testing of
the application. If you are familiar with other testing frameworks, you may notice similar functions being exposed that
are used such as `describe`, `it`, and `expect`. The other APIs, such as items like `$` and the methods it exposes is
covered by the [WebdriverIO API docs](https://webdriver.io/docs/api).

## Running the Test Suite

Now that we are all set up with a config and a spec, let's run it!

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npm test
```

</TabItem>

<TabItem value="yarn">

```sh
yarn test
```

</TabItem>
</Tabs>

We should see output the following output:

```text
➜  webdriverio git:(main) ✗ yarn test
yarn run v1.22.11
$ wdio run wdio.conf.js

Execution of 1 workers started at 2021-08-17T08:06:10.279Z

[0-0] RUNNING in undefined - /test/specs/example.e2e.js
[0-0] PASSED in undefined - /test/specs/example.e2e.js

 "spec" Reporter:
------------------------------------------------------------------
[wry 0.12.1 linux #0-0] Running: wry (v0.12.1) on linux
[wry 0.12.1 linux #0-0] Session ID: 81e0107b-4d38-4eed-9b10-ee80ca47bb83
[wry 0.12.1 linux #0-0]
[wry 0.12.1 linux #0-0] » /test/specs/example.e2e.js
[wry 0.12.1 linux #0-0] Hello Tauri
[wry 0.12.1 linux #0-0]    ✓ should be cordial
[wry 0.12.1 linux #0-0]    ✓ should be excited
[wry 0.12.1 linux #0-0]    ✓ should be easy on the eyes
[wry 0.12.1 linux #0-0]
[wry 0.12.1 linux #0-0] 3 passing (244ms)


Spec Files:	 1 passed, 1 total (100% completed) in 00:00:01

Done in 1.98s.
```

We see the Spec Reporter tell us that all 3 tests from the `test/specs/example.e2e.js` file, along with the final report
`Spec Files:	 1 passed, 1 total (100% completed) in 00:00:01`.

Using the [WebdriverIO] test suite, we just easily enabled e2e testing for our Tauri application from just a few lines
of configuration and a single command to run it! Even better, we didn't have to modify the application at all.


[WebdriverIO]: https://webdriver.io/
[finished example project]: https://github.com/chippers/hello_tauri
[example Application setup]: setup
[Mocha]: https://mochajs.org/
[suggest reading the documentation]: (https://webdriver.io/docs/configurationfile)
